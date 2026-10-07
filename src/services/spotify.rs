use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct SpotifyService {
    running: bool,
}

impl SpotifyService {
    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn refresh(&mut self) {
        self.running = Command::new("systemctl")
            .args(["--user", "is-active", "--quiet", "spotify-connect.service"])
            .status()
            .is_ok_and(|status| status.success());
    }
}
