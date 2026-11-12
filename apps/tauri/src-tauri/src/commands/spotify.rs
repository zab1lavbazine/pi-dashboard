use std::sync::Mutex;

use crate::{snapshots::DashboardStatus, state::DashboardState};

#[tauri::command]
pub(crate) fn spotify_action(
    action: String,
    state: tauri::State<'_, Mutex<DashboardState>>,
) -> Result<DashboardStatus, String> {
    validate_spotify_action(&action)?;
    let mut state = state
        .lock()
        .map_err(|_| "Dashboard state is unavailable.".to_owned())?;
    state.refresh_spotify();
    Ok(state.status())
}

fn validate_spotify_action(action: &str) -> Result<(), String> {
    match action {
        "refresh" => Ok(()),
        _ => Err("Invalid Spotify action. Allowed: refresh.".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spotify_actions_are_an_exact_allowlist() {
        assert!(validate_spotify_action("refresh").is_ok());
        for action in [
            "",
            "REFRESH",
            " refresh",
            "shell",
            "connect; reboot",
            "start",
            "stop",
        ] {
            assert!(validate_spotify_action(action).is_err());
        }
    }
}
