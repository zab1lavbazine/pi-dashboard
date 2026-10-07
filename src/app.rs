use crate::screen::Screen;
use crate::screens::{bluetooth, cow, home, network, spotify, terminal};
use crate::services::{
    bluetooth::BluetoothService, cow::CowService, spotify::SpotifyService,
    terminal::TerminalService,
};
use std::time::{Duration, Instant};

pub struct PiDashboardApp {
    pub screen: Screen,
    pub spotify: SpotifyService,
    pub bluetooth: BluetoothService,
    pub cow: CowService,
    pub terminal: TerminalService,
    pub terminal_input: String,
    last_status_refresh: Instant,
}

impl Default for PiDashboardApp {
    fn default() -> Self {
        Self {
            screen: Screen::Home,
            spotify: SpotifyService::default(),
            bluetooth: BluetoothService::default(),
            cow: CowService::default(),
            terminal: TerminalService::default(),
            terminal_input: String::new(),
            last_status_refresh: Instant::now(),
        }
    }
}

const DURATION_CHECK: u64 = 5;

impl eframe::App for PiDashboardApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        match self.screen {
            Screen::Home => home::show(self, ui),
            Screen::Spotify => spotify::show(self, ui),
            Screen::Bluetooth => bluetooth::show(self, ui),
            Screen::BluetoothDevice => bluetooth::show_device(self, ui),
            Screen::Network => network::show(self, ui),
            Screen::Terminal => terminal::show(self, ui),
            Screen::Cow => cow::show(self, ui),
        }
    }

    fn logic(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.2);
        self.bluetooth.poll();
        self.terminal.poll();
        self.cow.set_active(self.screen == Screen::Cow);
        self.cow.poll();

        if self.last_status_refresh.elapsed() >= Duration::from_secs(DURATION_CHECK) {
            self.refresh_services();
            self.last_status_refresh = Instant::now();

            ctx.request_repaint();
        }

        ctx.request_repaint_after(Duration::from_secs(1));
    }
}

impl PiDashboardApp {
    fn refresh_services(&mut self) {
        self.spotify.refresh();
    }
}
