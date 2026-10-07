use eframe::egui;

use crate::{
    screen::Screen,
    app::PiDashboardApp,
    ui::components::{big_button, heading, label, screen_container},
    layout::layout::calculate_layout,
};


pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    let layout = calculate_layout(ui);
    screen_container(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
                heading(ui, "Spotify");

                label(ui, "Status.");

                ui.add_space(20.0);

                let size = [
                    layout.button_width,
                    layout.button_height,
                ];

                big_button(ui, "Back", size, || {
                    app.screen = Screen::Home;
                });
        });
    });
}