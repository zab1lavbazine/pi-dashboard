use eframe::egui;

use crate::{
    app::PiDashboardApp,
    services::terminal::EntryKind,
    ui::components::{card, heading, label, screen_container},
};

pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    if ui.input(|input| input.pointer.any_pressed()) {
        app.go_back();
        return;
    }

    screen_container(ui, |ui| {
        heading(ui, "Terminal");
        label(ui, "Press anywhere on the touchscreen to return home.");
        ui.add_space(16.0);

        card(ui, |ui| {
            let output_height = (ui.available_height() - 48.0).max(120.0);

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(output_height)
                .show(ui, |ui| {
                    for entry in app.terminal.history() {
                        let color = match entry.kind {
                            EntryKind::Command => egui::Color32::LIGHT_GREEN,
                            EntryKind::Error => egui::Color32::LIGHT_RED,
                            EntryKind::Info => egui::Color32::GRAY,
                            EntryKind::Output => ui.visuals().text_color(),
                        };

                        ui.label(egui::RichText::new(&entry.text).monospace().color(color));
                    }
                });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("$").monospace().strong());

                let command_running = app.terminal.is_running();
                let response = ui.add_enabled(
                    !command_running,
                    egui::TextEdit::singleline(&mut app.terminal_input)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .hint_text(if command_running {
                            "Command is running…"
                        } else {
                            "Enter a command"
                        }),
                );

                if !command_running {
                    response.request_focus();
                }

                if !command_running && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                    let command = std::mem::take(&mut app.terminal_input);
                    app.terminal.submit(command);
                }
            });
        });
    });
}
