use std::{
    fs,
    io::ErrorKind,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

const COW_AUDIO: &[u8] = include_bytes!("../../resources/polish_cow/polish_cow.mp3");

#[derive(Default)]
pub struct CowService {
    audio_process: Option<Child>,
    audio_path: Option<PathBuf>,
    active: bool,
}

impl CowService {
    pub fn set_active(&mut self, active: bool) {
        if self.active == active {
            return;
        }

        self.active = active;
        if active {
            self.start_audio();
        } else {
            self.stop_audio();
        }
    }

    pub fn poll(&mut self) {
        let Some(process) = self.audio_process.as_mut() else {
            return;
        };

        match process.try_wait() {
            Ok(Some(status)) => {
                self.audio_process = None;
                if self.active {
                    eprintln!("Cow audio player exited with status {status}.");
                }
            }
            Ok(None) => {}
            Err(error) => {
                self.audio_process = None;
                eprintln!("Could not check cow audio playback: {error}");
            }
        }
    }

    fn start_audio(&mut self) {
        let audio_path = match self.audio_path() {
            Ok(path) => path,
            Err(error) => {
                eprintln!("{error}");
                return;
            }
        };

        let players: [(&str, &[&str]); 4] = [
            ("mpg123", &["--quiet", "--loop", "-1"]),
            ("mpv", &["--no-video", "--really-quiet", "--loop-file=inf"]),
            ("ffplay", &["-nodisp", "-loglevel", "quiet", "-loop", "0"]),
            ("cvlc", &["--intf", "dummy", "--loop"]),
        ];

        for (player, arguments) in players {
            match Command::new(player)
                .args(arguments)
                .arg(&audio_path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(process) => {
                    self.audio_process = Some(process);
                    return;
                }
                Err(error) if error.kind() == ErrorKind::NotFound => continue,
                Err(error) => {
                    eprintln!("Could not start {player}: {error}");
                    return;
                }
            }
        }

        eprintln!("No supported audio player found. Install mpg123, mpv, ffplay, or VLC.");
    }

    fn audio_path(&mut self) -> Result<PathBuf, String> {
        if let Some(path) = &self.audio_path {
            return Ok(path.clone());
        }

        let path = std::env::temp_dir().join(format!(
            "pi-dashboard-polish-cow-{}.mp3",
            std::process::id()
        ));
        fs::write(&path, COW_AUDIO)
            .map_err(|error| format!("Could not prepare cow audio: {error}"))?;
        self.audio_path = Some(path.clone());
        Ok(path)
    }

    fn stop_audio(&mut self) {
        if let Some(mut process) = self.audio_process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}

impl Drop for CowService {
    fn drop(&mut self) {
        self.stop_audio();
        if let Some(path) = self.audio_path.take() {
            let _ = fs::remove_file(path);
        }
    }
}
