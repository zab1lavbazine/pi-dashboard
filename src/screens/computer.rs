use eframe::egui;

use crate::{
    app::PiDashboardApp,
    layout::layout::calculate_layout,
    ui::components::{big_button, card, heading, label, scrollable_screen_container, status_row},
};

pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    scrollable_screen_container(ui, |ui| {
        heading(ui, "Computer");
        ui.add_space(16.0);

        card(ui, |ui| {
            let (connection, color) = if app.computer.is_connected() {
                ("Connected", egui::Color32::LIGHT_GREEN)
            } else if app.computer.is_connecting() {
                ("Connecting…", egui::Color32::YELLOW)
            } else {
                ("Disconnected", egui::Color32::LIGHT_RED)
            };
            status_row(ui, "Connection", connection, color);
            status_row(ui, "Server", app.computer.url(), egui::Color32::LIGHT_BLUE);
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            if let Some(snapshot) = app.computer.snapshot() {
                if snapshot.fields.is_empty() {
                    label(ui, "The computer sent an empty status message.");
                } else {
                    for (name, value) in &snapshot.fields {
                        status_row(ui, name, value, egui::Color32::WHITE);
                    }
                }
            } else {
                label(ui, "Waiting for system status…");
            }

            if let Some(error) = app.computer.error() {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(error).color(egui::Color32::LIGHT_RED));
            }
        });

        ui.add_space(16.0);
        let layout = calculate_layout(ui);
        big_button(
            ui,
            "Back",
            [layout.button_width, layout.button_height],
            || app.go_back(),
        );
    });
}
