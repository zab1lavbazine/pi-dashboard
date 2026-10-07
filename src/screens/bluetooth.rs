use eframe::egui;

use crate::{
    app::PiDashboardApp,
    layout::layout::calculate_layout,
    screen::Screen,
    services::bluetooth::{BluetoothService, DeviceAction},
    ui::components::{big_button, card, heading, label, scrollable_screen_container, status_row},
};

pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    app.bluetooth.ensure_initialized();

    scrollable_screen_container(ui, |ui| {
        heading(ui, "Bluetooth");
        ui.add_space(16.0);

        card(ui, |ui| {
            let layout = calculate_layout(ui);
            let size = [layout.button_width, layout.button_height];

            ui.horizontal(|ui| {
                if device_button(ui, "Refresh", !app.bluetooth.is_busy()) {
                    app.bluetooth.refresh();
                }
                if device_button(ui, "Scan", !app.bluetooth.is_busy()) {
                    app.bluetooth.scan();
                }
            });

            ui.add_space(12.0);
            big_button(ui, "Back", size, || {
                app.screen = Screen::Home;
            });
        });

        show_feedback(&app.bluetooth, ui);
        ui.add_space(12.0);

        if app.bluetooth.devices().is_empty() && !app.bluetooth.is_busy() {
            card(ui, |ui| {
                label(
                    ui,
                    "No Bluetooth devices found. Put a device in pairing mode, then press Scan.",
                );
            });
            return;
        }

        let devices = app.bluetooth.devices().to_vec();
        for device in devices {
            card(ui, |ui| {
                ui.label(egui::RichText::new(&device.name).size(22.0).strong());
                ui.label(egui::RichText::new(&device.address).monospace().weak());
                ui.add_space(10.0);

                status_row(
                    ui,
                    "Paired",
                    yes_no(device.paired),
                    status_color(device.paired),
                );
                status_row(
                    ui,
                    "Connected",
                    yes_no(device.connected),
                    status_color(device.connected),
                );

                if device_button(ui, "Manage", !app.bluetooth.is_busy())
                    && app.bluetooth.select_device(&device.address)
                {
                    app.screen = Screen::BluetoothDevice;
                }
            });
            ui.add_space(12.0);
        }
    });
}

pub fn show_device(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    scrollable_screen_container(ui, |ui| {
        heading(ui, "Bluetooth Device");
        ui.add_space(16.0);

        show_feedback(&app.bluetooth, ui);

        let Some(device) = app.bluetooth.selected_device().cloned() else {
            card(ui, |ui| {
                label(
                    ui,
                    "This device is no longer available. Scan again to rediscover it.",
                );
            });
            ui.add_space(12.0);

            if device_button(ui, "Back to devices", true) {
                app.screen = Screen::Bluetooth;
            }
            return;
        };

        card(ui, |ui| {
            ui.label(egui::RichText::new(&device.name).size(24.0).strong());
            ui.label(egui::RichText::new(&device.address).monospace().weak());
            ui.add_space(12.0);

            status_row(
                ui,
                "Paired",
                yes_no(device.paired),
                status_color(device.paired),
            );
            status_row(
                ui,
                "Trusted",
                yes_no(device.trusted),
                status_color(device.trusted),
            );
            status_row(
                ui,
                "Connected",
                yes_no(device.connected),
                status_color(device.connected),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            let enabled = !app.bluetooth.is_busy();

            if !device.paired && device_button(ui, "Pair", enabled) {
                app.bluetooth.perform(DeviceAction::Pair);
            }
            if device.paired && !device.trusted && device_button(ui, "Trust", enabled) {
                app.bluetooth.perform(DeviceAction::Trust);
            }
            if !device.connected && device_button(ui, "Connect", enabled) {
                app.bluetooth.perform(DeviceAction::Connect);
            }
            if device.connected && device_button(ui, "Disconnect", enabled) {
                app.bluetooth.perform(DeviceAction::Disconnect);
            }
            if device.paired && device_button(ui, "Remove pairing", enabled) {
                app.bluetooth.perform(DeviceAction::Remove);
            }

            ui.add_space(8.0);
            if device_button(ui, "Back to devices", enabled) {
                app.screen = Screen::Bluetooth;
            }
        });
    });
}

fn show_feedback(service: &BluetoothService, ui: &mut egui::Ui) {
    if let Some(busy_label) = service.busy_label() {
        ui.horizontal(|ui| {
            ui.spinner();
            label(ui, busy_label);
        });
        ui.add_space(12.0);
    }

    if let Some((message, is_error)) = service.message() {
        let color = if is_error {
            egui::Color32::LIGHT_RED
        } else {
            egui::Color32::LIGHT_GREEN
        };
        ui.label(egui::RichText::new(message).size(18.0).color(color));
        ui.add_space(12.0);
    }
}

fn device_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> bool {
    ui.add_enabled(
        enabled,
        egui::Button::new(egui::RichText::new(text).size(20.0)).min_size([170.0, 52.0].into()),
    )
    .clicked()
}

fn yes_no(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}

fn status_color(value: bool) -> egui::Color32 {
    if value {
        egui::Color32::LIGHT_GREEN
    } else {
        egui::Color32::GRAY
    }
}
