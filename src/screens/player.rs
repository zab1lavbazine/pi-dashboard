use eframe::egui;

use crate::{
    app::PiDashboardApp,
    screen::Screen,
    ui::components::{card, heading, label, scrollable_screen_container, status_row},
};

const CONTROL_SIZE: [f32; 2] = [150.0, 52.0];

pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    app.player.ensure_initialized();
    app.volume.ensure_initialized();

    scrollable_screen_container(ui, |ui| {
        heading(ui, "Music Player");
        ui.add_space(12.0);

        show_message(app, ui);
        card(ui, |ui| {
            status_row(
                ui,
                "Track",
                app.player.current_track_name().unwrap_or("None"),
                egui::Color32::WHITE,
            );
            status_row(
                ui,
                "Status",
                app.player.state_label(),
                if app.player.is_busy() {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::LIGHT_GREEN
                },
            );
            status_row(
                ui,
                "Bluetooth",
                app.player
                    .default_bluetooth_name()
                    .unwrap_or("Not configured"),
                if app.player.default_bluetooth_address().is_some() {
                    egui::Color32::LIGHT_GREEN
                } else {
                    egui::Color32::LIGHT_RED
                },
            );
            status_row(
                ui,
                "Volume",
                &app.volume
                    .level()
                    .map(|level| format!("{level}%"))
                    .unwrap_or_else(|| "Unknown".to_owned()),
                egui::Color32::WHITE,
            );

            ui.horizontal_wrapped(|ui| {
                if control_button(ui, "Previous", !app.player.is_busy()) {
                    app.player.play_previous();
                }
                if control_button(ui, "Stop", true) {
                    app.player.stop();
                }
                if control_button(ui, "Next", !app.player.is_busy()) {
                    app.player.play_next();
                }
                if control_button(ui, "Play random", !app.player.is_busy()) {
                    app.player.play_random();
                }
                let shuffle_label = if app.player.shuffle() {
                    "Shuffle: On"
                } else {
                    "Shuffle: Off"
                };
                if control_button(ui, shuffle_label, true) {
                    app.player.toggle_shuffle();
                }
            });

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                let volume_enabled = !app.volume.is_busy();
                if control_button(ui, "Volume −", volume_enabled) {
                    app.volume.decrease();
                }
                if control_button(ui, "Volume +", volume_enabled) {
                    app.volume.increase();
                }
                if control_button(
                    ui,
                    if app.volume.is_muted() {
                        "Unmute"
                    } else {
                        "Mute"
                    },
                    volume_enabled,
                ) {
                    app.volume.toggle_mute();
                }
            });
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            label(
                ui,
                &format!("Folder: {}", app.player.current_directory().display()),
            );
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if control_button(ui, "Up", app.player.can_go_up()) {
                    app.player.go_up();
                }
                if control_button(ui, "Refresh", true) {
                    app.player.refresh_directory();
                }
                if control_button(ui, "Settings", true) {
                    app.navigate_to(Screen::PlayerSettings);
                }
                if control_button(ui, "Back", true) {
                    app.go_back();
                }
            });
        });

        let folders = app.player.folders().to_vec();
        for (index, folder) in folders.iter().enumerate() {
            ui.add_space(8.0);
            if ui
                .add_sized(
                    [ui.available_width(), 52.0],
                    egui::Button::new(
                        egui::RichText::new(format!("📁 {}", folder.name)).size(20.0),
                    ),
                )
                .clicked()
            {
                app.player.enter_folder(index);
            }
        }

        let tracks = app.player.tracks().to_vec();
        for (index, track) in tracks.iter().enumerate() {
            ui.add_space(8.0);
            if ui
                .add_sized(
                    [ui.available_width(), 52.0],
                    egui::Button::new(egui::RichText::new(format!("▶ {}", track.name)).size(20.0)),
                )
                .clicked()
            {
                app.player.play(index);
            }
        }

        if folders.is_empty() && tracks.is_empty() {
            ui.add_space(8.0);
            card(ui, |ui| {
                label(ui, "This folder contains no music or subfolders.");
            });
        }
    });
}

pub fn show_settings(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    app.bluetooth.ensure_initialized();

    scrollable_screen_container(ui, |ui| {
        heading(ui, "Player Settings");
        ui.add_space(12.0);
        show_message(app, ui);
        if let Some(busy_label) = app.bluetooth.busy_label() {
            ui.horizontal(|ui| {
                ui.spinner();
                label(ui, busy_label);
            });
            ui.add_space(8.0);
        }
        if let Some((message, is_error)) = app.bluetooth.message() {
            ui.label(egui::RichText::new(message).size(18.0).color(if is_error {
                egui::Color32::LIGHT_RED
            } else {
                egui::Color32::LIGHT_GREEN
            }));
            ui.add_space(8.0);
        }

        card(ui, |ui| {
            label(ui, "Music library directory");
            ui.add_space(8.0);
            label(
                ui,
                &format!("Current: {}", app.player.music_directory().display()),
            );
            ui.add_space(10.0);
            if control_button(ui, "Browse storage", true) {
                app.player.begin_directory_selection();
                app.navigate_to(Screen::PlayerDirectoryPicker);
            }
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            label(ui, "Default Bluetooth output");
            ui.add_space(8.0);
            label(
                ui,
                app.player
                    .default_bluetooth_name()
                    .unwrap_or("Not configured"),
            );
            if let Some(address) = app.player.default_bluetooth_address() {
                ui.label(egui::RichText::new(address).monospace().weak());
            }
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if control_button(ui, "Refresh devices", !app.bluetooth.is_busy()) {
                    app.bluetooth.refresh();
                }
                if control_button(ui, "Clear default", true) {
                    app.player.clear_default_bluetooth_device();
                }
            });
        });

        let devices = app.bluetooth.devices().to_vec();
        for device in devices {
            ui.add_space(8.0);
            card(ui, |ui| {
                ui.label(egui::RichText::new(&device.name).size(20.0).strong());
                ui.label(egui::RichText::new(&device.address).monospace().weak());
                let status = if device.connected {
                    "Connected"
                } else if device.paired {
                    "Paired"
                } else {
                    "Not paired"
                };
                ui.label(status);
                if control_button(ui, "Use for player", device.paired) {
                    app.player.set_default_bluetooth_device(&device);
                }
            });
        }

        ui.add_space(12.0);
        card(ui, |ui| {
            label(
                ui,
                &format!("Settings file: {}", app.player.config_path().display()),
            );
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if control_button(ui, "Manage Bluetooth", true) {
                    app.navigate_to(Screen::Bluetooth);
                }
                if control_button(ui, "Back to player", true) {
                    app.go_back();
                }
            });
        });
    });
}

pub fn show_directory_picker(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    scrollable_screen_container(ui, |ui| {
        heading(ui, "Select Music Folder");
        ui.add_space(12.0);
        show_message(app, ui);

        card(ui, |ui| {
            label(
                ui,
                &format!("Folder: {}", app.player.picker_directory().display()),
            );
            ui.add_space(6.0);
            label(
                ui,
                &format!(
                    "{} supported track(s) in this folder",
                    app.player.picker_track_count()
                ),
            );
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                if control_button(ui, "Use this folder", true) {
                    app.player.use_picker_directory();
                    app.go_back();
                }
                if control_button(ui, "Up", app.player.picker_can_go_up()) {
                    app.player.picker_go_up();
                }
                if control_button(ui, "Cancel", true) {
                    app.go_back();
                }
            });
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            label(ui, "Storage locations");
            ui.add_space(8.0);
            let locations = app.player.storage_locations();
            ui.horizontal_wrapped(|ui| {
                for location in locations {
                    if control_button(ui, &location.display().to_string(), true) {
                        app.player.picker_open_location(&location);
                    }
                }
            });
        });

        let folders = app.player.picker_folders().to_vec();
        for (index, folder) in folders.iter().enumerate() {
            ui.add_space(8.0);
            if ui
                .add_sized(
                    [ui.available_width(), 52.0],
                    egui::Button::new(
                        egui::RichText::new(format!("📁 {}", folder.name)).size(20.0),
                    ),
                )
                .clicked()
            {
                app.player.picker_enter_folder(index);
            }
        }

        if folders.is_empty() {
            ui.add_space(8.0);
            card(ui, |ui| {
                label(ui, "This folder contains no subfolders.");
            });
        }
    });
}

fn show_message(app: &PiDashboardApp, ui: &mut egui::Ui) {
    if let Some((message, is_error)) = app.player.message() {
        ui.label(egui::RichText::new(message).size(18.0).color(if is_error {
            egui::Color32::LIGHT_RED
        } else {
            egui::Color32::LIGHT_GREEN
        }));
        ui.add_space(8.0);
    }
}

fn control_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> bool {
    ui.add_enabled(
        enabled,
        egui::Button::new(egui::RichText::new(text).size(19.0)).min_size(CONTROL_SIZE.into()),
    )
    .clicked()
}
