mod app;
mod layout;
mod screen;
mod screens;
mod services;
mod ui;

use app::PiDashboardApp;
use eframe::egui;

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 480.0;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size((SCREEN_WIDTH, SCREEN_HEIGHT)),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Pi Dashboard",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<PiDashboardApp>::default())
        }),
    );
}
