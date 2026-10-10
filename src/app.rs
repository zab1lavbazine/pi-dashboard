use crate::screen::Screen;
use crate::screens::{
    bluetooth, computer, home, media, network, player, power, spotify, system_info, terminal,
    volume,
};
use crate::services::{
    bluetooth::BluetoothService, computer::ComputerService, media::MediaService,
    player::PlayerService, power::PowerService, spotify::SpotifyService,
    system_info::SystemInfoService, terminal::TerminalService, volume::VolumeService,
};
use std::time::{Duration, Instant};

pub struct PiDashboardApp {
    pub screen: Screen,
    pub spotify: SpotifyService,
    pub computer: ComputerService,
    pub bluetooth: BluetoothService,
    pub media: MediaService,
    pub player: PlayerService,
    pub power: PowerService,
    pub system_info: SystemInfoService,
    pub volume: VolumeService,
    pub terminal: TerminalService,
    pub terminal_input: String,
    navigation_history: Vec<Screen>,
    last_user_activity: Instant,
    idle_overlay_visible: bool,
    last_status_refresh: Instant,
}

impl Default for PiDashboardApp {
    fn default() -> Self {
        Self {
            screen: Screen::Home,
            spotify: SpotifyService::default(),
            computer: ComputerService::default(),
            bluetooth: BluetoothService::default(),
            media: MediaService::default(),
            player: PlayerService::default(),
            power: PowerService::default(),
            system_info: SystemInfoService::default(),
            volume: VolumeService::default(),
            terminal: TerminalService::default(),
            terminal_input: String::new(),
            navigation_history: Vec::new(),
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
            Screen::Computer => computer::show(self, ui),
            Screen::Spotify => spotify::show(self, ui),
            Screen::Bluetooth => bluetooth::show(self, ui),
            Screen::BluetoothDevice => bluetooth::show_device(self, ui),
            Screen::Network => network::show(self, ui),
            Screen::Terminal => terminal::show(self, ui),
            Screen::Media => media::show_menu(self, ui),
            Screen::MediaPlayer => media::show_player(self, ui),
            Screen::Player => player::show(self, ui),
            Screen::PlayerSettings => player::show_settings(self, ui),
            Screen::PlayerDirectoryPicker => player::show_directory_picker(self, ui),
            Screen::Power => power::show(self, ui),
            Screen::Volume => volume::show(self, ui),
        }
    }

    fn logic(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.2);
        if self.bluetooth.poll() {
            ctx.request_repaint();
        }
        self.terminal.poll();
        self.media
            .set_player_active(self.screen == Screen::MediaPlayer);
        self.media.poll();
        if self.media.take_playback_finished() && self.screen == Screen::MediaPlayer {
            self.go_back();
        }
        self.player.poll();
        self.power.poll();
        self.spotify.poll();
        if self.computer.poll() {
            ctx.request_repaint();
        }
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
    pub fn navigate_to(&mut self, screen: Screen) {
        if screen == Screen::Home {
            self.go_home();
        } else if screen != self.screen {
            self.navigation_history.push(self.screen);
            self.screen = screen;
        }
    }

    pub fn go_back(&mut self) {
        let previous = self.navigation_history.pop().unwrap_or(Screen::Home);
        self.screen = previous;
        if previous == Screen::Home {
            self.navigation_history.clear();
        }
    }

    pub fn go_home(&mut self) {
        self.screen = Screen::Home;
        self.navigation_history.clear();
    }

    pub fn clear_navigation_history(&mut self) {
        if self.screen == Screen::Home {
            self.navigation_history.clear();
        }
    }

    fn refresh_services(&mut self) {
        self.spotify.refresh();
    }
}

// ---------------------------TEST--------------------------------
/*
    TESTS for current class
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_returns_in_the_order_screens_were_opened() {
        let mut app = PiDashboardApp::default();

        app.navigate_to(Screen::Player);
        app.navigate_to(Screen::PlayerSettings);
        app.navigate_to(Screen::PlayerDirectoryPicker);
        app.go_back();
        assert!(app.screen == Screen::PlayerSettings);
        app.go_back();
        assert!(app.screen == Screen::Player);
        app.go_back();
        assert!(app.screen == Screen::Home);
        assert!(app.navigation_history.is_empty());
    }

    #[test]
    fn going_home_clears_navigation_history() {
        let mut app = PiDashboardApp::default();
        app.navigate_to(Screen::Bluetooth);
        app.navigate_to(Screen::BluetoothDevice);

        app.go_home();

        assert!(app.screen == Screen::Home);
        assert!(app.navigation_history.is_empty());
    }
}
