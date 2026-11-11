use std::process::Command;

use super::task::{BackgroundTask, TaskPoll};

#[derive(Clone, Copy, PartialEq)]
enum AudioBackend {
    PipeWire,
    PulseAudio,
    AlsaMaster,
    AlsaPcm,
}

impl AudioBackend {
    fn label(self) -> &'static str {
        match self {
            Self::PipeWire => "PipeWire",
            Self::PulseAudio => "PulseAudio",
            Self::AlsaMaster | Self::AlsaPcm => "ALSA",
        }
    }

    fn alsa_control(self) -> Option<&'static str> {
        match self {
            Self::AlsaMaster => Some("Master"),
            Self::AlsaPcm => Some("PCM"),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
enum VolumeOperation {
    Increase,
    Decrease,
    ToggleMute,
}

struct VolumeSnapshot {
    level: u8,
    muted: bool,
    backend: AudioBackend,
}

#[derive(Default)]
pub struct VolumeService {
    level: Option<u8>,
    muted: bool,
    backend: Option<AudioBackend>,
    task: BackgroundTask<Result<VolumeSnapshot, String>>,
    busy_label: Option<String>,
    message: Option<(String, bool)>,
    initialized: bool,
}

impl VolumeService {
    pub fn level(&self) -> Option<u8> {
        self.level
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn backend_name(&self) -> Option<&'static str> {
        self.backend.map(AudioBackend::label)
    }

    pub fn is_busy(&self) -> bool {
        self.task.is_running()
    }

    pub fn busy_label(&self) -> Option<&str> {
        self.busy_label.as_deref()
    }

    pub fn message(&self) -> Option<(&str, bool)> {
        self.message
            .as_ref()
            .map(|(message, is_error)| (message.as_str(), *is_error))
    }

    pub fn ensure_initialized(&mut self) {
        if !self.initialized && !self.is_busy() {
            self.refresh();
        }
    }

    pub fn refresh(&mut self) {
        self.start_task("Reading volume…", None);
    }

    pub fn increase(&mut self) {
        self.start_task("Increasing volume…", Some(VolumeOperation::Increase));
    }

    pub fn decrease(&mut self) {
        self.start_task("Decreasing volume…", Some(VolumeOperation::Decrease));
    }

    pub fn toggle_mute(&mut self) {
        self.start_task("Changing mute state…", Some(VolumeOperation::ToggleMute));
    }

    pub fn poll(&mut self) {
        match self.task.poll() {
            TaskPoll::Ready(Ok(snapshot)) => {
                self.level = Some(snapshot.level);
                self.muted = snapshot.muted;
                self.backend = Some(snapshot.backend);
                self.message = None;
                self.finish_task();
            }
            TaskPoll::Ready(Err(error)) => {
                self.message = Some((error, true));
                self.finish_task();
            }
            TaskPoll::Disconnected => {
                self.message = Some(("Volume task ended unexpectedly.".to_owned(), true));
                self.finish_task();
            }
            TaskPoll::Pending => {}
        }
    }

    fn start_task(&mut self, label: &str, operation: Option<VolumeOperation>) {
        if self.is_busy() {
            return;
        }

        let preferred_backend = self.backend;
        self.busy_label = Some(label.to_owned());
        self.message = None;
        self.task
            .start(move || run_task(preferred_backend, operation));
    }

    fn finish_task(&mut self) {
        self.busy_label = None;
        self.initialized = true;
    }
}

fn run_task(
    preferred_backend: Option<AudioBackend>,
    operation: Option<VolumeOperation>,
) -> Result<VolumeSnapshot, String> {
    let snapshot = read_volume(preferred_backend)?;

    if let Some(operation) = operation {
        run_operation(snapshot.backend, operation)?;
        read_backend(snapshot.backend)
    } else {
        Ok(snapshot)
    }
}

fn read_volume(preferred_backend: Option<AudioBackend>) -> Result<VolumeSnapshot, String> {
    let mut backends = Vec::with_capacity(4);
    if let Some(backend) = preferred_backend {
        backends.push(backend);
    }
    for backend in [
        AudioBackend::PipeWire,
        AudioBackend::PulseAudio,
        AudioBackend::AlsaMaster,
        AudioBackend::AlsaPcm,
    ] {
        if !backends.contains(&backend) {
            backends.push(backend);
        }
    }

    let mut last_error = None;
    for backend in backends {
        match read_backend(backend) {
            Ok(snapshot) => return Ok(snapshot),
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| "No supported audio backend found.".to_owned()))
}

fn read_backend(backend: AudioBackend) -> Result<VolumeSnapshot, String> {
    match backend {
        AudioBackend::PipeWire => {
            let output = run("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"])?;
            parse_wpctl(&output)
        }
        AudioBackend::PulseAudio => {
            let volume = run("pactl", &["get-sink-volume", "@DEFAULT_SINK@"])?;
            let mute = run("pactl", &["get-sink-mute", "@DEFAULT_SINK@"])?;
            Ok(VolumeSnapshot {
                level: parse_percent(&volume)
                    .ok_or_else(|| "Could not parse PulseAudio volume.".to_owned())?,
                muted: mute.to_ascii_lowercase().contains("yes"),
                backend,
            })
        }
        AudioBackend::AlsaMaster | AudioBackend::AlsaPcm => {
            let control = backend.alsa_control().unwrap_or("Master");
            let output = run("amixer", &["get", control])?;
            Ok(VolumeSnapshot {
                level: parse_percent(&output)
                    .ok_or_else(|| "Could not parse ALSA volume.".to_owned())?,
                muted: output.contains("[off]"),
                backend,
            })
        }
    }
}

fn run_operation(backend: AudioBackend, operation: VolumeOperation) -> Result<(), String> {
    match (backend, operation) {
        (AudioBackend::PipeWire, VolumeOperation::Increase) => run(
            "wpctl",
            &["set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", "5%+"],
        ),
        (AudioBackend::PipeWire, VolumeOperation::Decrease) => {
            run("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", "5%-"])
        }
        (AudioBackend::PipeWire, VolumeOperation::ToggleMute) => {
            run("wpctl", &["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
        }
        (AudioBackend::PulseAudio, VolumeOperation::Increase) => {
            run("pactl", &["set-sink-volume", "@DEFAULT_SINK@", "+5%"])
        }
        (AudioBackend::PulseAudio, VolumeOperation::Decrease) => {
            run("pactl", &["set-sink-volume", "@DEFAULT_SINK@", "-5%"])
        }
        (AudioBackend::PulseAudio, VolumeOperation::ToggleMute) => {
            run("pactl", &["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
        }
        (AudioBackend::AlsaMaster | AudioBackend::AlsaPcm, operation) => {
            let control = backend.alsa_control().unwrap_or("Master");
            let value = match operation {
                VolumeOperation::Increase => concat!("5", "%+"),
                VolumeOperation::Decrease => concat!("5", "%-"),
                VolumeOperation::ToggleMute => "toggle",
            };
            run("amixer", &["set", control, value])
        }
    }
    .map(|_| ())
}

fn run(program: &str, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .map_err(|error| format!("Could not run {program}: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();

    if output.status.success() {
        Ok(stdout)
    } else if stderr.is_empty() {
        Err(format!("{program} command failed."))
    } else {
        Err(stderr)
    }
}

fn parse_wpctl(output: &str) -> Result<VolumeSnapshot, String> {
    let level = output
        .split_whitespace()
        .find_map(|value| value.parse::<f32>().ok())
        .map(|value| (value * 100.0).round().clamp(0.0, 100.0) as u8)
        .ok_or_else(|| "Could not parse PipeWire volume.".to_owned())?;

    Ok(VolumeSnapshot {
        level,
        muted: output.contains("[MUTED]"),
        backend: AudioBackend::PipeWire,
    })
}

fn parse_percent(output: &str) -> Option<u8> {
    let percent_index = output.find('%')?;
    let digits: String = output[..percent_index]
        .chars()
        .rev()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    digits.parse::<u8>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pipewire_volume() {
        let snapshot = parse_wpctl("Volume: 0.57 [MUTED]").unwrap();
        assert_eq!(snapshot.level, 57);
        assert!(snapshot.muted);
    }

    #[test]
    fn parses_alsa_and_pulse_percentages() {
        assert_eq!(parse_percent("Mono: Playback 48 [75%] [on]"), Some(75));
        assert_eq!(parse_percent("Volume: 65536 / 100% / 0.00 dB"), Some(100));
    }
}
