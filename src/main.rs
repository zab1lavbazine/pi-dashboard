mod ui;
mod screen;
mod screens;
mod app;
mod layout;

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
        Box::new(|_cc| Ok(Box::<PiDashboardApp>::default())),
    );
}