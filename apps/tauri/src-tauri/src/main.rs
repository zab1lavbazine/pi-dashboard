#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use pi_dashboard_core::{
    system_info::{SystemInfoService, SystemInfoSnapshot},
    volume::VolumeService,
};
use serde::Serialize;

#[derive(Default)]
struct DashboardState {
    system: SystemInfoService,
    volume: VolumeService,
}

#[derive(Serialize)]
struct SystemSnapshot {
    cpu_percent: Option<f32>,
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
struct VolumeStatus {
    level: Option<u8>,
    muted: Option<bool>,
    backend: Option<String>,
    busy: bool,
    busy_label: Option<String>,
    message: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct DashboardStatus {
    system: Option<SystemSnapshot>,
    system_loading: bool,
    system_error: Option<String>,
    volume: VolumeStatus,
}

impl DashboardState {
    fn status(&self) -> DashboardStatus {
        let message = self.volume.message();
        DashboardStatus {
            system: self.system.snapshot().map(SystemSnapshot::from),
            system_loading: self.system.snapshot().is_none() && self.system.error().is_none(),
            system_error: self.system.error().map(str::to_owned),
            volume: VolumeStatus {
                level: self.volume.level(),
                muted: self.volume.level().map(|_| self.volume.is_muted()),
                backend: self.volume.backend_name().map(str::to_owned),
                busy: self.volume.is_busy(),
                busy_label: self.volume.busy_label().map(str::to_owned),
                message: message
                    .filter(|(_, error)| !error)
                    .map(|(text, _)| text.to_owned()),
                error: message
                    .filter(|(_, error)| *error)
                    .map(|(text, _)| text.to_owned()),
            },
        }
    }
}

#[tauri::command]
fn dashboard_status(
    state: tauri::State<'_, Mutex<DashboardState>>,
) -> Result<DashboardStatus, String> {
    let mut state = state
        .lock()
        .map_err(|_| "Dashboard state is unavailable.".to_owned())?;
    state.system.poll();
    state.system.ensure_fresh();
    state.volume.poll();
    state.volume.ensure_initialized();
    Ok(state.status())
}

#[derive(Debug, PartialEq)]
enum VolumeAction {
    Up,
    Down,
    Mute,
    Refresh,
}

impl VolumeAction {
    fn parse(action: &str) -> Result<Self, String> {
        match action {
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            "mute" => Ok(Self::Mute),
            "refresh" => Ok(Self::Refresh),
            _ => Err("Invalid volume action. Allowed: up, down, mute, refresh.".to_owned()),
        }
    }
}

#[tauri::command]
fn volume_action(
    action: String,
    state: tauri::State<'_, Mutex<DashboardState>>,
) -> Result<DashboardStatus, String> {
    let action = VolumeAction::parse(&action)?;
    let mut state = state
        .lock()
        .map_err(|_| "Dashboard state is unavailable.".to_owned())?;
    state.volume.poll();
    if state.volume.is_busy() {
        return Err("Volume is busy. Wait for the current operation to finish.".to_owned());
    }
    match action {
        VolumeAction::Up => state.volume.increase(),
        VolumeAction::Down => state.volume.decrease(),
        VolumeAction::Mute => state.volume.toggle_mute(),
        VolumeAction::Refresh => state.volume.refresh(),
    }
    Ok(state.status())
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(DashboardState::default()))
        .invoke_handler(tauri::generate_handler![dashboard_status, volume_action])
        .run(tauri::generate_context!())
        .expect("Failed to run the experimental dashboard");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_actions_are_an_exact_allowlist() {
        for (text, expected) in [
            ("up", VolumeAction::Up),
            ("down", VolumeAction::Down),
            ("mute", VolumeAction::Mute),
            ("refresh", VolumeAction::Refresh),
        ] {
            assert_eq!(VolumeAction::parse(text), Ok(expected));
        }
        for text in ["", "UP", "shutdown", "shell", "up; reboot", " refresh"] {
            assert!(VolumeAction::parse(text).is_err());
        }
    }

    #[test]
    fn initial_status_does_not_invent_readings() {
        let status = DashboardState::default().status();
        assert!(status.system.is_none());
        assert!(status.system_loading);
        assert!(status.volume.level.is_none());
        assert!(status.volume.muted.is_none());
        assert!(!status.volume.busy);
    }
}
