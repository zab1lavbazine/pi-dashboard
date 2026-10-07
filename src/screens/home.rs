use eframe::egui;

use crate::{
    app::PiDashboardApp,
    layout::layout::calculate_layout,
    screen::Screen,
    ui::components::{big_button, card, heading, scrollable_screen_container},
};

const NUMBER_OF_COLUMNS: usize = 3;

pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    scrollable_screen_container(ui, |ui| {
        heading(ui, "Pi Dashboard");
        ui.add_space(16.0);

        card(ui, |ui| {
            let layout = calculate_layout(ui);

            egui::Grid::new("home_grid")
                .num_columns(NUMBER_OF_COLUMNS)
                .spacing([layout.spacing, layout.spacing])
                .show(ui, |ui| {
                    let size = [layout.button_width, layout.button_height];
                    big_button(ui, "Spotify", size, || {
                        app.screen = Screen::Spotify;
                    });
                    big_button(ui, "Bluetooth", size, || {
                        app.screen = Screen::Bluetooth;
                    });
                    big_button(ui, "Network", size, || {
                        app.screen = Screen::Network;
                    });
                    ui.end_row();
                    big_button(ui, "Terminal", size, || {
                        app.screen = Screen::Terminal;
                    });
                    big_button(ui, "Media", size, || {
                        app.screen = Screen::Media;
                    });
                    big_button(ui, "Power", size, || {
                        app.screen = Screen::Power;
                    });
                    ui.end_row();
                    big_button(ui, "Volume", size, || {
                        app.screen = Screen::Volume;
                    });
                    ui.end_row();
                });
        });
    });
}
