use std::collections::BTreeMap;

use pi_dashboard_core::system_info::SystemInfoSnapshot;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct SystemSnapshot {
    pub(crate) cpu_percent: Option<f32>,
    pub(crate) temperature_celsius: Option<f32>,
    pub(crate) memory_used_bytes: Option<u64>,
    pub(crate) memory_total_bytes: Option<u64>,
    pub(crate) disk_used_bytes: Option<u64>,
    pub(crate) disk_total_bytes: Option<u64>,
    pub(crate) uptime_seconds: Option<u64>,
    pub(crate) load_average: Option<f32>,
    pub(crate) hostname: String,
    pub(crate) ip_address: Option<String>,
}

impl From<&SystemInfoSnapshot> for SystemSnapshot {
    fn from(value: &SystemInfoSnapshot) -> Self {
        Self {
            cpu_percent: value.cpu_percent,
            temperature_celsius: value.temperature_celsius,
            memory_used_bytes: value.memory_used_bytes,
            memory_total_bytes: value.memory_total_bytes,
            disk_used_bytes: value.disk_used_bytes,
            disk_total_bytes: value.disk_total_bytes,
            uptime_seconds: value.uptime_seconds,
            load_average: value.load_average,
            hostname: value.hostname.clone(),
            ip_address: value.ip_address.clone(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct VolumeStatus {
    pub(crate) level: Option<u8>,
    pub(crate) muted: Option<bool>,
    pub(crate) backend: Option<String>,
    pub(crate) busy: bool,
    pub(crate) busy_label: Option<String>,
    pub(crate) message: Option<String>,
    pub(crate) error: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct BluetoothDeviceSnapshot {
    pub(crate) address: String,
    pub(crate) name: String,
    pub(crate) paired: bool,
    pub(crate) trusted: bool,
    pub(crate) connected: bool,
}

#[derive(Serialize)]
pub(crate) struct BluetoothStatus {
    pub(crate) devices: Vec<BluetoothDeviceSnapshot>,
    pub(crate) selected_address: Option<String>,
    pub(crate) busy: bool,
    pub(crate) busy_label: Option<String>,
    pub(crate) message: Option<String>,
    pub(crate) error: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ComputerStatus {
    pub(crate) url: String,
    pub(crate) connected: bool,
    pub(crate) connecting: bool,
    pub(crate) fields: Option<BTreeMap<String, String>>,
    pub(crate) error: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct SpotifyStatus {
    // Core reports false both for a stopped service and an unsuccessful status check.
    pub(crate) running: bool,
}

#[derive(Serialize)]
pub(crate) struct DashboardStatus {
    pub(crate) system: Option<SystemSnapshot>,
    pub(crate) system_loading: bool,
    pub(crate) system_error: Option<String>,
    pub(crate) volume: VolumeStatus,
    pub(crate) bluetooth: BluetoothStatus,
    pub(crate) computer: ComputerStatus,
    pub(crate) spotify: SpotifyStatus,
}
