

use eframe::egui;
use crate::layout::layout::LayoutSizes;

pub const BUTTON_SIZE: f32 = 24.0;

pub fn big_button<F>(
    ui: &mut egui::Ui, 
    text: &str,
    size: [f32; 2],
    on_click: F,
) where F: FnOnce(),
{

    let response = ui.add_sized(
        size,
        egui::Button::new(
            egui::RichText::new(text)
                .size(BUTTON_SIZE),
        ),
    );

    if response.clicked() {
        on_click();
    }
}


pub fn heading(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.label(
        egui::RichText::new(text)
            .heading()
            .size(32.0)
    )
}

pub fn label(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.label(
        egui::RichText::new(text)
            .size(32.0)
    )
}

pub fn screen_container(
    ui: &mut egui::Ui,
    content: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 20,
            right: 20,
            top: 10,
            bottom: 10,
        })
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .show(ui, |ui| {
                    content(ui);
                });
        });
}