use eframe::egui;

use crate::{
    app::PiDashboardApp,
    layout::layout::calculate_layout,
    ui::components::{big_button, card, heading, label, scrollable_screen_container},
};

pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    scrollable_screen_container(ui, |ui| {
        heading(ui, "Network");
        ui.add_space(16.0);

        card(ui, |ui| {
            label(ui, "Network status.");
            ui.add_space(20.0);

            let layout = calculate_layout(ui);
            big_button(
                ui,
                "Back",
                [layout.button_width, layout.button_height],
                || {
                    app.go_back();
                },
            );
        });
    });
}
