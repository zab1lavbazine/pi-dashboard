use eframe::egui;

use crate::{
    screen::Screen,
    app::PiDashboardApp,
    ui::components::{big_button, heading},
    layout::layout::calculate_layout,
};


pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    let layoutSize = calculate_layout(ui);
    heading(ui, "Bluetooth");

    ui.add_space(20.0);

    big_button(ui, "Back", [layoutSize.button_width, layoutSize.button_height ], || {
        app.screen = Screen::Home;
    });
}