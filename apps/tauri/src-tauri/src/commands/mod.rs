pub(crate) mod bluetooth;
pub(crate) mod spotify;
pub(crate) mod volume;

use std::sync::Mutex;

use crate::{snapshots::DashboardStatus, state::DashboardState};

#[tauri::command]
pub(crate) fn dashboard_status(
    state: tauri::State<'_, Mutex<DashboardState>>,
) -> Result<DashboardStatus, String> {
    let mut state = state
        .lock()
        .map_err(|_| "Dashboard state is unavailable.".to_owned())?;
    state.poll();
    Ok(state.status())
}
