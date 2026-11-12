use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct SystemSnapshot {
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
#[derive(Clone, Debug, Default, Deserialize)]
pub struct VolumeStatus {
    pub level: Option<u8>,
    pub muted: Option<bool>,
    pub backend: Option<String>,
    pub busy: bool,
    pub busy_label: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Default, Deserialize)]
pub struct BluetoothDevice {
    pub address: String,
    pub name: String,
    pub paired: bool,
    pub trusted: bool,
    pub connected: bool,
}
#[derive(Clone, Debug, Default, Deserialize)]
pub struct BluetoothStatus {
    pub devices: Vec<BluetoothDevice>,
    pub selected_address: Option<String>,
    pub busy: bool,
    pub busy_label: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ComputerStatus {
    pub url: String,
    pub connected: bool,
    pub connecting: bool,
    pub fields: Option<BTreeMap<String, String>>,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Default, Deserialize)]
pub struct SpotifyStatus {
    pub running: bool,
}
#[derive(Clone, Debug, Default, Deserialize)]
pub struct DashboardStatus {
    pub system: Option<SystemSnapshot>,
    pub system_loading: bool,
    pub system_error: Option<String>,
    pub volume: VolumeStatus,
    pub bluetooth: BluetoothStatus,
    pub computer: ComputerStatus,
    pub spotify: SpotifyStatus,
}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VolumeAction {
    Up,
    Down,
    Mute,
    Refresh,
}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BluetoothAction {
    Refresh,
    Scan,
    Select,
    Pair,
    Trust,
    Connect,
    Disconnect,
    Remove,
}
impl BluetoothAction {
    pub fn name(self) -> &'static str {
        match self {
            Self::Refresh => "refresh",
            Self::Scan => "scan",
            Self::Select => "select",
            Self::Pair => "pair",
            Self::Trust => "trust",
            Self::Connect => "connect",
            Self::Disconnect => "disconnect",
            Self::Remove => "remove",
        }
    }
}
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SpotifyAction {
    Refresh,
}
#[derive(Serialize)]
pub struct ActionArgs<T> {
    pub action: T,
}
#[derive(Serialize)]
pub struct BluetoothArgs {
    pub action: BluetoothAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}
