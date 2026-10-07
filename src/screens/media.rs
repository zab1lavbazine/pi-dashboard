use eframe::egui;

use crate::{
    app::PiDashboardApp,
    layout::layout::calculate_layout,
    screen::Screen,
    ui::components::{big_button, card, heading, label, scrollable_screen_container},
};

const NUMBER_OF_COLUMNS: usize = 3;

pub fn show_menu(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    app.media.ensure_loaded();

    scrollable_screen_container(ui, |ui| {
        heading(ui, "Media");
        ui.add_space(16.0);

        if let Some(error) = app.media.playback_error() {
            ui.label(
                egui::RichText::new(error)
                    .size(18.0)
                    .color(egui::Color32::LIGHT_RED),
            );
            ui.add_space(12.0);
        }

        if !app.media.load_errors().is_empty() {
            card(ui, |ui| {
                label(ui, "Some media configurations could not be loaded:");
                ui.add_space(8.0);
                for error in app.media.load_errors() {
                    ui.label(egui::RichText::new(error).color(egui::Color32::LIGHT_RED));
                }
            });
            ui.add_space(12.0);
        }

        let buttons = app
            .media
            .items()
            .iter()
            .enumerate()
            .map(|(index, item)| (index, item.name().to_owned()))
            .collect::<Vec<_>>();

        card(ui, |ui| {
            if buttons.is_empty() {
                label(ui, "No configured media found.");
            } else {
                let layout = calculate_layout(ui);
                let size = [layout.button_width, layout.button_height];

                egui::Grid::new("media_grid")
                    .num_columns(NUMBER_OF_COLUMNS)
                    .spacing([layout.spacing, layout.spacing])
                    .show(ui, |ui| {
                        for (position, (index, name)) in buttons.iter().enumerate() {
                            big_button(ui, name, size, || {
                                if app.media.select(*index) {
                                    app.screen = Screen::MediaPlayer;
                                }
                            });
                            if (position + 1) % NUMBER_OF_COLUMNS == 0 {
                                ui.end_row();
                            }
                        }

                        if buttons.len() % NUMBER_OF_COLUMNS != 0 {
                            ui.end_row();
                        }
                    });
            }
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            label(
                ui,
                &format!(
                    "Configuration directory: {}",
                    app.media.config_directory().display()
                ),
            );
            label(
                ui,
                &format!(
                    "Resource directory: {}",
                    app.media.resource_directory().display()
                ),
            );
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui
                    .add_sized([180.0, 60.0], egui::Button::new("Reload"))
                    .clicked()
                {
                    app.media.reload();
                }
                if ui
                    .add_sized([180.0, 60.0], egui::Button::new("Back"))
                    .clicked()
                {
                    app.screen = Screen::Home;
                }
            });
        });
    });
}

pub fn show_player(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    if ui.input(|input| input.pointer.any_pressed()) {
        app.screen = Screen::Media;
        return;
    }

    let Some(item) = app.media.selected_item() else {
        app.screen = Screen::Media;
        return;
    };

    ui.add(
        egui::Image::from_bytes(item.gif_uri().to_owned(), item.gif_bytes())
            .fit_to_exact_size(ui.available_size())
            .maintain_aspect_ratio(false),
    );
}
