use crate::screen::Screen;
use crate::screens::{home, spotify, bluetooth, network};


pub struct PiDashboardApp {
    pub screen: Screen,
}

impl Default for PiDashboardApp {
    fn default() -> Self {
        Self {
            screen: Screen::Home,
        }
    }
}


impl eframe::App for PiDashboardApp {
    fn ui(
        &mut self,
        ui: &mut eframe::egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        match self.screen {
            Screen::Home => home::show(self, ui),
            Screen::Spotify => spotify::show(self, ui),
            Screen::Bluetooth => bluetooth::show(self, ui),
            Screen::Network => network::show(self, ui),
        }
    }

    fn logic(
        &mut self,
        ctx: &eframe::egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        ctx.set_pixels_per_point(1.2);
    }
}