use std::process::Command;

use super::task::{BackgroundTask, TaskPoll};

const COMMAND_TIMEOUT_SECONDS: &str = "10";
const SCAN_TIMEOUT_SECONDS: &str = "8";
const PAIR_TIMEOUT_SECONDS: &str = "30";

#[derive(Clone, Debug)]
pub struct BluetoothDevice {
    pub address: String,
    pub name: String,
    pub paired: bool,
    pub trusted: bool,
    pub connected: bool,
}

#[derive(Clone, Copy)]
pub enum DeviceAction {
    Pair,
    Trust,
    Connect,
    Disconnect,
    Remove,
}

impl DeviceAction {
    pub fn progress_label(self) -> &'static str {
        match self {
            Self::Pair => "Pairing device…",
            Self::Trust => "Trusting device…",
            Self::Connect => "Connecting device…",
            Self::Disconnect => "Disconnecting device…",
            Self::Remove => "Removing device…",
        }
    }

    pub fn success_label(self) -> &'static str {
        match self {
            Self::Pair => "Device paired successfully.",
            Self::Trust => "Device trusted successfully.",
            Self::Connect => "Device connected successfully.",
            Self::Disconnect => "Device disconnected successfully.",
            Self::Remove => "Device removed successfully.",
        }
    }

    fn command(self) -> &'static str {
        match self {
            Self::Pair => "pair",
            Self::Trust => "trust",
            Self::Connect => "connect",
            Self::Disconnect => "disconnect",
            Self::Remove => "remove",
        }
    }
}

struct BluetoothTaskResult {
    update: BluetoothUpdate,
    message: Option<String>,
}

enum BluetoothUpdate {
    ReplaceAll(Vec<BluetoothDevice>),
    UpdateOne(BluetoothDevice),
    RemoveOne(String),
}

#[derive(Default)]
pub struct BluetoothService {
    devices: Vec<BluetoothDevice>,
    selected_address: Option<String>,
    task: BackgroundTask<Result<BluetoothTaskResult, String>>,
    busy_label: Option<String>,
    message: Option<(String, bool)>,
    initialized: bool,
}

impl BluetoothService {
    pub fn devices(&self) -> &[BluetoothDevice] {
        &self.devices
    }

    pub fn selected_device(&self) -> Option<&BluetoothDevice> {
        let address = self.selected_address.as_deref()?;
        self.devices.iter().find(|device| device.address == address)
    }

    pub fn select_device(&mut self, address: &str) -> bool {
        if self.devices.iter().any(|device| device.address == address) {
            self.selected_address = Some(address.to_owned());
            true
        } else {
            false
        }
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

    pub fn poll(&mut self) -> bool {
        match self.task.poll() {
            TaskPoll::Ready(Ok(result)) => {
                self.apply_update(result.update);
                self.message = result.message.map(|message| (message, false));
                self.finish_task();
                true
            }
            TaskPoll::Ready(Err(error)) => {
                self.message = Some((error, true));
                self.finish_task();
                true
            }
            TaskPoll::Disconnected => {
                self.message = Some(("Bluetooth task ended unexpectedly.".to_owned(), true));
                self.finish_task();
                true
            }
            TaskPoll::Pending => false,
        }
    }

    pub fn refresh(&mut self) {
        self.start_task("Refreshing devices…", || {
            Ok(BluetoothTaskResult {
                update: BluetoothUpdate::ReplaceAll(list_devices()?),
                message: None,
            })
        });
    }

    pub fn scan(&mut self) {
        self.start_task("Scanning for devices…", || {
            scan()?;
            Ok(BluetoothTaskResult {
                update: BluetoothUpdate::ReplaceAll(list_devices()?),
                message: Some("Scan complete.".to_owned()),
            })
        });
    }

    pub fn perform(&mut self, action: DeviceAction) {
        let Some(address) = self.selected_address.clone() else {
            return;
        };
        let fallback_name = self
            .selected_device()
            .map(|device| device.name.clone())
            .unwrap_or_else(|| address.clone());

        self.start_task(action.progress_label(), move || {
            perform_action(action, &address)?;
            let update = if matches!(action, DeviceAction::Remove) {
                BluetoothUpdate::RemoveOne(address)
            } else {
                BluetoothUpdate::UpdateOne(read_device(&address, fallback_name)?)
            };
            Ok(BluetoothTaskResult {
                update,
                message: Some(action.success_label().to_owned()),
            })
        });
    }

    fn apply_update(&mut self, update: BluetoothUpdate) {
        match update {
            BluetoothUpdate::ReplaceAll(devices) => self.devices = devices,
            BluetoothUpdate::UpdateOne(device) => {
                if let Some(existing) = self
                    .devices
                    .iter_mut()
                    .find(|existing| existing.address == device.address)
                {
                    *existing = device;
                } else {
                    self.devices.push(device);
                }
            }
            BluetoothUpdate::RemoveOne(address) => {
                self.devices.retain(|device| device.address != address);
                if self.selected_address.as_deref() == Some(address.as_str()) {
                    self.selected_address = None;
                }
            }
        }
        sort_devices(&mut self.devices);
    }

    fn finish_task(&mut self) {
        self.busy_label = None;
        self.initialized = true;
    }

    fn start_task<F>(&mut self, busy_label: &str, operation: F)
    where
        F: FnOnce() -> Result<BluetoothTaskResult, String> + Send + 'static,
    {
        if self.is_busy() {
            return;
        }

        self.busy_label = Some(busy_label.to_owned());
        self.message = None;
        self.task.start(operation);
    }
}

fn list_devices() -> Result<Vec<BluetoothDevice>, String> {
    let output = run(&["--timeout", COMMAND_TIMEOUT_SECONDS, "devices"])?;
    let mut devices = Vec::new();

    for line in output.lines() {
        let Some(device) = parse_device_line(line) else {
            continue;
        };

        devices.push(read_device(&device.address, device.name.clone()).unwrap_or(device));
    }

    sort_devices(&mut devices);
    Ok(devices)
}

fn read_device(address: &str, fallback_name: String) -> Result<BluetoothDevice, String> {
    let info = run(&["--timeout", COMMAND_TIMEOUT_SECONDS, "info", address])?;
    Ok(BluetoothDevice {
        address: address.to_owned(),
        name: property(&info, "Name")
            .or_else(|| property(&info, "Alias"))
            .unwrap_or(fallback_name),
        paired: yes_property(&info, "Paired"),
        trusted: yes_property(&info, "Trusted"),
        connected: yes_property(&info, "Connected"),
    })
}

fn sort_devices(devices: &mut [BluetoothDevice]) {
    devices.sort_by_key(|device| {
        (
            !device.connected,
            !device.paired,
            device.name.to_lowercase(),
        )
    });
}

fn scan() -> Result<(), String> {
    run(&["--timeout", SCAN_TIMEOUT_SECONDS, "scan", "on"]).map(|_| ())
}

fn perform_action(action: DeviceAction, address: &str) -> Result<(), String> {
    if !valid_address(address) {
        return Err("The selected Bluetooth address is invalid.".to_owned());
    }

    let timeout = if matches!(action, DeviceAction::Pair) {
        PAIR_TIMEOUT_SECONDS
    } else {
        COMMAND_TIMEOUT_SECONDS
    };

    let mut arguments = vec!["--timeout", timeout];
    if matches!(action, DeviceAction::Pair) {
        arguments.extend(["--agent", "NoInputNoOutput"]);
    }
    arguments.extend([action.command(), address]);

    run(&arguments).map(|_| ())
}

pub fn ensure_device_connected(address: &str) -> Result<(), String> {
    if !valid_address(address) {
        return Err("The default Bluetooth address is invalid.".to_owned());
    }

    let info = run(&["--timeout", COMMAND_TIMEOUT_SECONDS, "info", address])?;
    if yes_property(&info, "Connected") {
        return Ok(());
    }
    if !yes_property(&info, "Paired") {
        return Err(
            "The default Bluetooth device is not paired. Pair it on the Bluetooth screen first."
                .to_owned(),
        );
    }

    perform_action(DeviceAction::Connect, address)
}

fn run(arguments: &[&str]) -> Result<String, String> {
    let output = Command::new("bluetoothctl")
        .args(arguments)
        .output()
        .map_err(|error| format!("Could not run bluetoothctl: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();

    if output.status.success() {
        Ok(stdout)
    } else {
        let details = if stderr.is_empty() { stdout } else { stderr };
        Err(if details.is_empty() {
            "Bluetooth command failed. Is Bluetooth powered on?".to_owned()
        } else {
            details
        })
    }
}

fn parse_device_line(line: &str) -> Option<BluetoothDevice> {
    let device_text = line.trim().strip_prefix("Device ")?;
    let (address, name) = device_text.split_once(' ')?;

    valid_address(address).then(|| BluetoothDevice {
        address: address.to_owned(),
        name: name.trim().to_owned(),
        paired: false,
        trusted: false,
        connected: false,
    })
}

fn property(info: &str, property_name: &str) -> Option<String> {
    info.lines().find_map(|line| {
        let (name, value) = line.trim().split_once(':')?;
        (name == property_name).then(|| value.trim().to_owned())
    })
}

fn yes_property(info: &str, property_name: &str) -> bool {
    property(info, property_name).is_some_and(|value| value == "yes")
}

fn valid_address(address: &str) -> bool {
    let parts: Vec<_> = address.split(':').collect();
    parts.len() == 6
        && parts.iter().all(|part| {
            part.len() == 2 && part.chars().all(|character| character.is_ascii_hexdigit())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_device_line() {
        let device = parse_device_line("Device AA:BB:CC:DD:EE:FF Living Room Speaker").unwrap();

        assert_eq!(device.address, "AA:BB:CC:DD:EE:FF");
        assert_eq!(device.name, "Living Room Speaker");
    }

    #[test]
    fn parses_device_properties() {
        let info = "\tName: Headphones\n\tPaired: yes\n\tConnected: no";

        assert_eq!(property(info, "Name").as_deref(), Some("Headphones"));
        assert!(yes_property(info, "Paired"));
        assert!(!yes_property(info, "Connected"));
    }

    #[test]
    fn rejects_invalid_device_address() {
        assert!(!valid_address("not-an-address"));
        assert!(!valid_address("AA:BB:CC:DD:EE"));
    }

    #[test]
    fn partial_update_keeps_other_devices() {
        let mut service = BluetoothService {
            devices: vec![
                test_device("AA:BB:CC:DD:EE:01", "First", false),
                test_device("AA:BB:CC:DD:EE:02", "Second", false),
            ],
            ..Default::default()
        };

        service.apply_update(BluetoothUpdate::UpdateOne(test_device(
            "AA:BB:CC:DD:EE:01",
            "First",
            true,
        )));

        assert_eq!(service.devices.len(), 2);
        assert!(
            service
                .devices
                .iter()
                .find(|device| device.address == "AA:BB:CC:DD:EE:01")
                .is_some_and(|device| device.connected)
        );
    }

    fn test_device(address: &str, name: &str, connected: bool) -> BluetoothDevice {
        BluetoothDevice {
            address: address.to_owned(),
            name: name.to_owned(),
            paired: true,
            trusted: true,
            connected,
        }
    }
}
