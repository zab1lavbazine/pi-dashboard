use eframe::egui;

pub const HEADING_SIZE: f32 = 32.0;
pub const LABEL_SIZE: f32 = 18.0;
pub const BUTTON_SIZE: f32 = 24.0;
pub const STATUS_SIZE: f32 = 18.0;

pub fn big_button<F>(ui: &mut egui::Ui, text: &str, size: [f32; 2], on_click: F)
where
    F: FnOnce(),
{
    let response = ui.add_sized(
        size,
        egui::Button::new(egui::RichText::new(text).size(BUTTON_SIZE)),
    );

    if response.clicked() {
        on_click();
    }
}

pub fn heading(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.label(egui::RichText::new(text).heading().size(HEADING_SIZE))
}

pub fn label(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.label(egui::RichText::new(text).size(LABEL_SIZE))
}

pub fn screen_container(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 20,
            right: 20,
            top: 10,
            bottom: 10,
        })
        .show(ui, |ui| {
            content(ui);
        });
}

pub fn scrollable_screen_container(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
    screen_container(ui, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                content(ui);
            });
    });
}

pub fn status_row(ui: &mut egui::Ui, name: &str, value: &str, color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(name).size(STATUS_SIZE));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(value)
                    .size(STATUS_SIZE)
                    .color(color)
                    .strong(),
            );
        });
    });

    ui.add_space(8.0);
}

pub fn card(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(8.0)
        .inner_margin(16)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            content(ui);
        });
}
