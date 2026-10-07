use std::{
    fs,
    process::Command,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::{Duration, Instant},
};

const REFRESH_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Clone, Copy)]
struct CpuTimes {
    idle: u64,
    total: u64,
}

struct RawSystemInfo {
    cpu_times: Option<CpuTimes>,
    temperature_celsius: Option<f32>,
    memory_used_bytes: Option<u64>,
    memory_total_bytes: Option<u64>,
    disk_used_bytes: Option<u64>,
    disk_total_bytes: Option<u64>,
    uptime_seconds: Option<u64>,
    load_average: Option<f32>,
    hostname: String,
    ip_address: Option<String>,
}

#[derive(Clone, Default)]
pub struct SystemInfoSnapshot {
    pub cpu_percent: Option<f32>,
    pub temperature_celsius: Option<f32>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub disk_used_bytes: Option<u64>,
    pub disk_total_bytes: Option<u64>,
    pub uptime_seconds: Option<u64>,
    pub load_average: Option<f32>,
    pub hostname: String,
    pub ip_address: Option<String>,
}

#[derive(Default)]
pub struct SystemInfoService {
    snapshot: Option<SystemInfoSnapshot>,
    receiver: Option<Receiver<RawSystemInfo>>,
    previous_cpu_times: Option<CpuTimes>,
    last_refresh: Option<Instant>,
    error: Option<String>,
}

impl SystemInfoService {
    pub fn snapshot(&self) -> Option<&SystemInfoSnapshot> {
        self.snapshot.as_ref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn ensure_fresh(&mut self) {
        let refresh_due = self
            .last_refresh
            .is_none_or(|last_refresh| last_refresh.elapsed() >= REFRESH_INTERVAL);

        if self.receiver.is_none() && refresh_due {
            let (sender, receiver) = mpsc::channel();
            self.receiver = Some(receiver);
            thread::spawn(move || {
                let _ = sender.send(read_system_info());
            });
        }
    }

    pub fn poll(&mut self) {
        let result = self.receiver.as_ref().map(Receiver::try_recv);

        match result {
            Some(Ok(raw)) => {
                let cpu_percent = cpu_percent(self.previous_cpu_times, raw.cpu_times);
                self.previous_cpu_times = raw.cpu_times;
                self.snapshot = Some(SystemInfoSnapshot {
                    cpu_percent,
                    temperature_celsius: raw.temperature_celsius,
                    memory_used_bytes: raw.memory_used_bytes,
                    memory_total_bytes: raw.memory_total_bytes,
                    disk_used_bytes: raw.disk_used_bytes,
                    disk_total_bytes: raw.disk_total_bytes,
                    uptime_seconds: raw.uptime_seconds,
                    load_average: raw.load_average,
                    hostname: raw.hostname,
                    ip_address: raw.ip_address,
                });
                self.receiver = None;
                self.last_refresh = Some(Instant::now());
                self.error = None;
            }
            Some(Err(TryRecvError::Disconnected)) => {
                self.receiver = None;
                self.last_refresh = Some(Instant::now());
                self.error = Some("Could not refresh system information.".to_owned());
            }
            Some(Err(TryRecvError::Empty)) | None => {}
        }
    }
}

fn read_system_info() -> RawSystemInfo {
    let (memory_used_bytes, memory_total_bytes) = read_memory().unwrap_or((0, 0));
    let (disk_used_bytes, disk_total_bytes) = read_disk().unwrap_or((0, 0));

    RawSystemInfo {
        cpu_times: read_cpu_times(),
        temperature_celsius: read_temperature(),
        memory_used_bytes: (memory_total_bytes > 0).then_some(memory_used_bytes),
        memory_total_bytes: (memory_total_bytes > 0).then_some(memory_total_bytes),
        disk_used_bytes: (disk_total_bytes > 0).then_some(disk_used_bytes),
        disk_total_bytes: (disk_total_bytes > 0).then_some(disk_total_bytes),
        uptime_seconds: read_uptime(),
        load_average: read_load_average(),
        hostname: fs::read_to_string("/etc/hostname")
            .map(|hostname| hostname.trim().to_owned())
            .unwrap_or_else(|_| "Raspberry Pi".to_owned()),
        ip_address: read_ip_address(),
    }
}

fn read_cpu_times() -> Option<CpuTimes> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    let values: Vec<u64> = stat
        .lines()
        .next()?
        .split_whitespace()
        .skip(1)
        .filter_map(|value| value.parse().ok())
        .collect();

    if values.len() < 4 {
        return None;
    }

    Some(CpuTimes {
        idle: values[3] + values.get(4).copied().unwrap_or(0),
        total: values.iter().sum(),
    })
}

fn cpu_percent(previous: Option<CpuTimes>, current: Option<CpuTimes>) -> Option<f32> {
    let previous = previous?;
    let current = current?;
    let total_delta = current.total.saturating_sub(previous.total);
    let idle_delta = current.idle.saturating_sub(previous.idle);

    (total_delta > 0)
        .then(|| ((total_delta.saturating_sub(idle_delta)) as f32 / total_delta as f32) * 100.0)
}

fn read_temperature() -> Option<f32> {
    if let Ok(value) = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
        let value = value.trim().parse::<f32>().ok()?;
        return Some(if value > 1_000.0 {
            value / 1_000.0
        } else {
            value
        });
    }

    let output = Command::new("vcgencmd").arg("measure_temp").output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.split_once('=')?
        .1
        .trim()
        .trim_end_matches("'C")
        .parse()
        .ok()
}

fn read_memory() -> Option<(u64, u64)> {
    let memory = fs::read_to_string("/proc/meminfo").ok()?;
    let total = meminfo_value(&memory, "MemTotal")? * 1_024;
    let available = meminfo_value(&memory, "MemAvailable")? * 1_024;
    Some((total.saturating_sub(available), total))
}

fn meminfo_value(memory: &str, name: &str) -> Option<u64> {
    memory.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if key != name {
            return None;
        }
        value.split_whitespace().next()?.parse().ok()
    })
}

fn read_disk() -> Option<(u64, u64)> {
    let output = Command::new("df")
        .args(["-B1", "--output=size,used", "/"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let values: Vec<u64> = text
        .lines()
        .nth(1)?
        .split_whitespace()
        .filter_map(|value| value.parse().ok())
        .collect();
    Some((*values.get(1)?, *values.first()?))
}

fn read_uptime() -> Option<u64> {
    fs::read_to_string("/proc/uptime")
        .ok()?
        .split_whitespace()
        .next()?
        .parse::<f64>()
        .ok()
        .map(|seconds| seconds as u64)
}

fn read_load_average() -> Option<f32> {
    fs::read_to_string("/proc/loadavg")
        .ok()?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

fn read_ip_address() -> Option<String> {
    let output = Command::new("hostname").arg("-I").output().ok()?;
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_cpu_usage_from_two_samples() {
        let previous = CpuTimes {
            idle: 400,
            total: 1_000,
        };
        let current = CpuTimes {
            idle: 450,
            total: 1_100,
        };

        assert_eq!(cpu_percent(Some(previous), Some(current)), Some(50.0));
    }

    #[test]
    fn parses_memory_values() {
        let memory = "MemTotal:       1000 kB\nMemAvailable:    400 kB\n";
        assert_eq!(meminfo_value(memory, "MemTotal"), Some(1_000));
        assert_eq!(meminfo_value(memory, "MemAvailable"), Some(400));
    }
}
