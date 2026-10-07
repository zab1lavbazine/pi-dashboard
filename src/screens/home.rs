use eframe::egui;



use crate::{
    screen::Screen,
    app::PiDashboardApp,
    ui::components::{big_button, heading, screen_container},
    layout::layout::calculate_layout,
};

const NUMBER_OF_COLUMNS: usize = 3;


pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    heading(ui, "Pi Dashboard");
    ui.add_space(20.0);

    screen_container(ui, |ui| {
        let layout = calculate_layout(ui);
        
        egui::Grid::new("home_grid")
            .num_columns(NUMBER_OF_COLUMNS)
            .spacing([layout.spacing, layout.spacing])
            .show(ui, |ui| {
                let size = [
                    layout.button_width,
                    layout.button_height,
                ];
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
            });
    });
   


    ui.add_space(20.0);
}