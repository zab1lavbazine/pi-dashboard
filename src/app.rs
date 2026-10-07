use crate::screen::Screen;
use crate::screens::{
    bluetooth, cow, home, network, power, spotify, system_info, terminal, volume,
};
use crate::services::{
    bluetooth::BluetoothService, cow::CowService, power::PowerService, spotify::SpotifyService,
    system_info::SystemInfoService, terminal::TerminalService, volume::VolumeService,
};
use std::time::{Duration, Instant};

pub struct PiDashboardApp {
    pub screen: Screen,
    pub spotify: SpotifyService,
    pub bluetooth: BluetoothService,
    pub cow: CowService,
    pub power: PowerService,
    pub system_info: SystemInfoService,
    pub volume: VolumeService,
    pub terminal: TerminalService,
    pub terminal_input: String,
    last_user_activity: Instant,
    idle_overlay_visible: bool,
    last_status_refresh: Instant,
}

impl Default for PiDashboardApp {
    fn default() -> Self {
        Self {
            screen: Screen::Home,
            spotify: SpotifyService::default(),
            bluetooth: BluetoothService::default(),
            cow: CowService::default(),
            power: PowerService::default(),
            system_info: SystemInfoService::default(),
            volume: VolumeService::default(),
            terminal: TerminalService::default(),
            terminal_input: String::new(),
            last_user_activity: Instant::now(),
            idle_overlay_visible: false,
            last_status_refresh: Instant::now(),
        }
    }
}

const DURATION_CHECK: u64 = 5;
const IDLE_OVERLAY_DELAY: Duration = Duration::from_secs(10);

impl eframe::App for PiDashboardApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        let user_activity = has_user_activity(ui);
        let home_screen_active = self.screen == Screen::Home;

        if !home_screen_active {
            self.idle_overlay_visible = false;
            self.last_user_activity = Instant::now();
        }

        if self.idle_overlay_visible {
            if user_activity {
                self.last_user_activity = Instant::now();
                self.idle_overlay_visible = false;
                ui.ctx().request_repaint();
                return;
            }

            self.system_info.ensure_fresh();
            system_info::show(self, ui);
            return;
        }

        if home_screen_active && user_activity {
            self.last_user_activity = Instant::now();
        } else if home_screen_active && self.last_user_activity.elapsed() >= IDLE_OVERLAY_DELAY {
            self.idle_overlay_visible = true;
            self.system_info.ensure_fresh();
            system_info::show(self, ui);
            return;
        }

        match self.screen {
            Screen::Home => home::show(self, ui),
            Screen::Spotify => spotify::show(self, ui),
            Screen::Bluetooth => bluetooth::show(self, ui),
            Screen::BluetoothDevice => bluetooth::show_device(self, ui),
            Screen::Network => network::show(self, ui),
            Screen::Terminal => terminal::show(self, ui),
            Screen::Cow => cow::show(self, ui),
            Screen::Power => power::show(self, ui),
            Screen::Volume => volume::show(self, ui),
        }
    }

    fn logic(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.2);
        self.bluetooth.poll();
        self.terminal.poll();
        self.cow.set_active(self.screen == Screen::Cow);
        self.cow.poll();
        self.power.poll();
        self.volume.poll();
        self.system_info.poll();

        if self.last_status_refresh.elapsed() >= Duration::from_secs(DURATION_CHECK) {
            self.refresh_services();
            self.last_status_refresh = Instant::now();

            ctx.request_repaint();
        }

        ctx.request_repaint_after(Duration::from_secs(1));
    }
}

fn has_user_activity(ui: &eframe::egui::Ui) -> bool {
    use eframe::egui::{Event, TouchPhase};

    ui.input(|input| {
        input.events.iter().any(|event| {
            matches!(
                event,
                Event::PointerButton { pressed: true, .. }
                    | Event::Touch {
                        phase: TouchPhase::Start,
                        ..
                    }
                    | Event::Key { pressed: true, .. }
                    | Event::Text(_)
                    | Event::Paste(_)
                    | Event::MouseWheel { .. }
            )
        })
    })
}

impl PiDashboardApp {
    fn refresh_services(&mut self) {
        self.spotify.refresh();
    }
}
