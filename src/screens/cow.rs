use eframe::egui;

use crate::{app::PiDashboardApp, screen::Screen};

const COW_GIF: &[u8] = include_bytes!("../../resources/polish_cow/polish_cow.gif");

pub fn show(app: &mut PiDashboardApp, ui: &mut egui::Ui) {
    if ui.input(|input| input.pointer.any_pressed()) {
        app.screen = Screen::Home;
        return;
    }

    let screen_size = ui.available_size();
    ui.add(
        egui::Image::from_bytes("bytes://polish_cow.gif", COW_GIF)
            .fit_to_exact_size(screen_size)
            .maintain_aspect_ratio(false),
    );
}
