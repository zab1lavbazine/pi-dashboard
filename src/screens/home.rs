use eframe::egui;

use crate::{
    app::PiDashboardApp,
    layout::layout::calculate_layout,
    screen::Screen,
    ui::components::{big_button, card, heading, scrollable_screen_container},
};

const NUMBER_OF_COLUMNS: usize = 3;

pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    app.clear_navigation_history();
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
                        app.navigate_to(Screen::Spotify);
                    });
                    big_button(ui, "Bluetooth", size, || {
                        app.navigate_to(Screen::Bluetooth);
                    });
                    big_button(ui, "Network", size, || {
                        app.navigate_to(Screen::Network);
                    });
                    ui.end_row();
                    big_button(ui, "Terminal", size, || {
                        app.navigate_to(Screen::Terminal);
                    });
                    big_button(ui, "Media", size, || {
                        app.navigate_to(Screen::Media);
                    });
                    big_button(ui, "Power", size, || {
                        app.navigate_to(Screen::Power);
                    });
                    ui.end_row();
                    big_button(ui, "Volume", size, || {
                        app.navigate_to(Screen::Volume);
                    });
                    big_button(ui, "Player", size, || {
                        app.navigate_to(Screen::Player);
                    });
                    big_button(ui, "Computer", size, || {
                        app.navigate_to(Screen::Computer);
                    });
                    ui.end_row();
                });
        });
    });
}
