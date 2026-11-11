use eframe::egui;

use crate::{
    app::PiDashboardApp,
    services::volume::VolumeService,
    ui::components::{card, heading, label, scrollable_screen_container, status_row},
};

const BUTTON_SIZE: [f32; 2] = [180.0, 60.0];

pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    app.volume.ensure_initialized();

    scrollable_screen_container(ui, |ui| {
        heading(ui, "Volume");
        ui.add_space(16.0);

        show_feedback(&app.volume, ui);

        card(ui, |ui| {
            let level = app
                .volume
                .level()
                .map(|level| format!("{level}%"))
                .unwrap_or_else(|| "Unknown".to_owned());
            status_row(ui, "Volume", &level, egui::Color32::WHITE);
            status_row(
                ui,
                "Muted",
                if app.volume.is_muted() { "Yes" } else { "No" },
                if app.volume.is_muted() {
                    egui::Color32::LIGHT_RED
                } else {
                    egui::Color32::LIGHT_GREEN
                },
            );
            if let Some(backend) = app.volume.backend_name() {
                status_row(ui, "Backend", backend, egui::Color32::GRAY);
            }
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            let enabled = !app.volume.is_busy();
            ui.horizontal(|ui| {
                if volume_button(ui, "Volume −", enabled) {
                    app.volume.decrease();
                }
                if volume_button(ui, "Volume +", enabled) {
                    app.volume.increase();
                }
                if volume_button(
                    ui,
                    if app.volume.is_muted() {
                        "Unmute"
                    } else {
                        "Mute"
                    },
                    enabled,
                ) {
                    app.volume.toggle_mute();
                }
            });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if volume_button(ui, "Refresh", enabled) {
                    app.volume.refresh();
                }
                if volume_button(ui, "Back", enabled) {
                    app.go_back();
                }
            });
        });
    });
}

fn show_feedback(service: &VolumeService, ui: &mut egui::Ui) {
    if let Some(busy_label) = service.busy_label() {
        ui.horizontal(|ui| {
            ui.spinner();
            label(ui, busy_label);
        });
        ui.add_space(12.0);
    }

    if let Some((message, is_error)) = service.message() {
        let color = if is_error {
            egui::Color32::LIGHT_RED
        } else {
            egui::Color32::LIGHT_GREEN
        };
        ui.label(egui::RichText::new(message).size(18.0).color(color));
        ui.add_space(12.0);
    }
}

fn volume_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> bool {
    ui.add_enabled(
        enabled,
        egui::Button::new(egui::RichText::new(text).size(20.0)).min_size(BUTTON_SIZE.into()),
    )
    .clicked()
}
