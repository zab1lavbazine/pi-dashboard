use eframe::egui;

use crate::{
    screen::Screen,
    app::PiDashboardApp,
    ui::components::{big_button, heading, label},
    layout::layout::calculate_layout,
};


pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    let layout = calculate_layout(ui);
    heading(ui, "Network");

    label(ui, "Network status.");

    ui.add_space(20.0);

    big_button(ui, "Back", [layout.button_width, layout.button_height], || {
        app.screen = Screen::Home;
    });

}