#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod snapshots;
mod state;

use std::sync::Mutex;

use state::DashboardState;

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(DashboardState::default()))
        .invoke_handler(tauri::generate_handler![
            commands::dashboard_status,
            commands::volume::volume_action,
            commands::bluetooth::bluetooth_action,
            commands::spotify::spotify_action
        ])
        .run(tauri::generate_context!())
        .expect("Failed to run the experimental dashboard");
}
