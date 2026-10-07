use eframe::egui;

use crate::{
    app::PiDashboardApp,
    layout::layout::calculate_layout,
    screen::Screen,
    ui::components::{big_button, card, heading, scrollable_screen_container, status_row},
};

pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    scrollable_screen_container(ui, |ui| {
        heading(ui, "Spotify");
        ui.add_space(16.0);

        card(ui, |ui| {
            status_row(
                ui,
                "Service",
                if app.spotify.is_running() {
                    "Running"
                } else {
                    "Stopped"
                },
                if app.spotify.is_running() {
                    egui::Color32::LIGHT_GREEN
                } else {
                    egui::Color32::LIGHT_RED
                },
            );

            let layout = calculate_layout(ui);
            big_button(
                ui,
                "Back",
                [layout.button_width, layout.button_height],
                || {
                    app.screen = Screen::Home;
                },
            );
        });
    });
}
