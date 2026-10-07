use eframe::egui;

use crate::{
    app::PiDashboardApp,
    services::system_info::SystemInfoSnapshot,
    ui::components::{heading, label, status_row},
};

pub fn show(app: &PiDashboardApp, ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(8, 12, 18))
        .inner_margin(32)
        .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            heading(ui, "System status");

            if let Some(snapshot) = app.system_info.snapshot() {
                ui.label(
                    egui::RichText::new(system_identity(snapshot))
                        .size(18.0)
                        .color(egui::Color32::GRAY),
                );
                ui.add_space(24.0);
                show_metrics(snapshot, ui);
            } else {
                ui.add_space(24.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    label(ui, "Loading system information…");
                });
            }

            if let Some(error) = app.system_info.error() {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(error)
                        .size(18.0)
                        .color(egui::Color32::LIGHT_RED),
                );
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("Touch the screen to return")
                        .size(18.0)
                        .color(egui::Color32::GRAY),
                );
            });
        });
}

fn show_metrics(snapshot: &SystemInfoSnapshot, ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(18, 25, 35))
        .corner_radius(10.0)
        .inner_margin(20)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            status_row(
                ui,
                "Temperature",
                &snapshot
                    .temperature_celsius
                    .map(|value| format!("{value:.1} °C"))
                    .unwrap_or_else(|| "Unavailable".to_owned()),
                temperature_color(snapshot.temperature_celsius),
            );
            status_row(
                ui,
                "CPU usage",
                &snapshot
                    .cpu_percent
                    .map(|value| format!("{value:.0}%"))
                    .unwrap_or_else(|| "Calculating…".to_owned()),
                egui::Color32::LIGHT_BLUE,
            );
            status_row(
                ui,
                "Load average",
                &snapshot
                    .load_average
                    .map(|value| format!("{value:.2}"))
                    .unwrap_or_else(|| "Unavailable".to_owned()),
                egui::Color32::WHITE,
            );
            status_row(
                ui,
                "Memory",
                &usage_text(snapshot.memory_used_bytes, snapshot.memory_total_bytes),
                egui::Color32::LIGHT_GREEN,
            );
            status_row(
                ui,
                "Disk",
                &usage_text(snapshot.disk_used_bytes, snapshot.disk_total_bytes),
                egui::Color32::LIGHT_GREEN,
            );
            status_row(
                ui,
                "Uptime",
                &snapshot
                    .uptime_seconds
                    .map(format_uptime)
                    .unwrap_or_else(|| "Unavailable".to_owned()),
                egui::Color32::WHITE,
            );
        });
}

fn system_identity(snapshot: &SystemInfoSnapshot) -> String {
    match &snapshot.ip_address {
        Some(address) => format!("{}  •  {address}", snapshot.hostname),
        None => snapshot.hostname.clone(),
    }
}

fn usage_text(used: Option<u64>, total: Option<u64>) -> String {
    match (used, total) {
        (Some(used), Some(total)) if total > 0 => {
            let percent = used as f64 / total as f64 * 100.0;
            format!(
                "{percent:.0}%  ({:.1}/{:.1} GB)",
                bytes_to_gb(used),
                bytes_to_gb(total)
            )
        }
        _ => "Unavailable".to_owned(),
    }
}

fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / 1_073_741_824.0
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else {
        format!("{hours}h {minutes}m")
    }
}

fn temperature_color(temperature: Option<f32>) -> egui::Color32 {
    match temperature {
        Some(value) if value >= 80.0 => egui::Color32::LIGHT_RED,
        Some(value) if value >= 65.0 => egui::Color32::YELLOW,
        Some(_) => egui::Color32::LIGHT_GREEN,
        None => egui::Color32::GRAY,
    }
}
