use eframe::egui;

use crate::{
    app::PiDashboardApp,
    services::power::PowerAction,
    ui::components::{card, heading, label, scrollable_screen_container},
};

const BUTTON_SIZE: [f32; 2] = [220.0, 64.0];

pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    scrollable_screen_container(ui, |ui| {
        heading(ui, "Power");
        ui.add_space(16.0);

        show_feedback(app, ui);

        if let Some(action) = app.power.confirmation() {
            show_confirmation(app, ui, action);
        } else {
            show_actions(app, ui);
        }
    });
}

fn show_actions(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    card(ui, |ui| {
        label(ui, "Choose a power action.");
        ui.add_space(16.0);

        ui.horizontal(|ui| {
            if power_button(
                ui,
                "Shut down",
                egui::Color32::DARK_RED,
                !app.power.is_busy(),
            ) {
                app.power.request_confirmation(PowerAction::Shutdown);
            }
            if power_button(
                ui,
                "Reboot",
                egui::Color32::from_rgb(160, 100, 0),
                !app.power.is_busy(),
            ) {
                app.power.request_confirmation(PowerAction::Reboot);
            }
        });

        ui.add_space(16.0);
        if power_button(
            ui,
            "Back",
            ui.visuals().faint_bg_color,
            !app.power.is_busy(),
        ) {
            app.power.cancel_confirmation();
            app.go_back();
        }
    });
}

fn show_confirmation(app: &mut PiDashboardApp, ui: &mut egui::Ui, action: PowerAction) {
    card(ui, |ui| {
        ui.label(
            egui::RichText::new(action.confirmation_text())
                .size(22.0)
                .strong()
                .color(egui::Color32::LIGHT_RED),
        );
        ui.add_space(16.0);

        ui.horizontal(|ui| {
            let confirm_color = match action {
                PowerAction::Shutdown => egui::Color32::DARK_RED,
                PowerAction::Reboot => egui::Color32::from_rgb(160, 100, 0),
            };

            if power_button(ui, action.label(), confirm_color, true) {
                app.power.confirm();
            }
            if power_button(ui, "Cancel", ui.visuals().faint_bg_color, true) {
                app.power.cancel_confirmation();
            }
        });
    });
}

fn show_feedback(app: &PiDashboardApp, ui: &mut egui::Ui) {
    if app.power.is_busy() {
        ui.horizontal(|ui| {
            ui.spinner();
            label(ui, "Sending power command…");
        });
        ui.add_space(12.0);
    }

    if let Some((message, is_error)) = app.power.message() {
        let color = if is_error {
            egui::Color32::LIGHT_RED
        } else {
            egui::Color32::LIGHT_GREEN
        };
        ui.label(egui::RichText::new(message).size(18.0).color(color));
        ui.add_space(12.0);
    }
}

fn power_button(ui: &mut egui::Ui, text: &str, color: egui::Color32, enabled: bool) -> bool {
    ui.add_enabled(
        enabled,
        egui::Button::new(egui::RichText::new(text).size(22.0))
            .fill(color)
            .min_size(BUTTON_SIZE.into()),
    )
    .clicked()
}
