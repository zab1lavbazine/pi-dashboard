use std::process::Command;

use super::task::{BackgroundTask, TaskPoll};

#[derive(Default)]
pub struct SpotifyService {
    running: bool,
    task: BackgroundTask<bool>,
}

impl SpotifyService {
    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn refresh(&mut self) {
        self.task.start(|| {
            Command::new("systemctl")
                .args(["--user", "is-active", "--quiet", "spotify-connect.service"])
                .status()
                .is_ok_and(|status| status.success())
        });
    }

    pub fn poll(&mut self) {
        match self.task.poll() {
            TaskPoll::Ready(running) => self.running = running,
            TaskPoll::Disconnected => self.running = false,
            TaskPoll::Pending => {}
        }
    }
}
