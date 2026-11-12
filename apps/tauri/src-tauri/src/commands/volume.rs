use std::sync::Mutex;

use crate::{snapshots::DashboardStatus, state::DashboardState};

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
pub(crate) fn volume_action(
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
}
