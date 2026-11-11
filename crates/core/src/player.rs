use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    bluetooth::{BluetoothDevice, ensure_device_connected},
    task::{BackgroundTask, TaskPoll},
};

const PLAYER_CONFIG_ENV: &str = "PI_DASHBOARD_PLAYER_CONFIG";

#[derive(Clone)]
pub struct LibraryEntry {
    pub name: String,
    path: PathBuf,
}

#[derive(Deserialize, Serialize)]
struct PlayerConfig {
    #[serde(default = "default_music_directory")]
    music_directory: PathBuf,
    #[serde(default)]
    default_bluetooth_address: Option<String>,
    #[serde(default)]
    default_bluetooth_name: Option<String>,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            music_directory: default_music_directory(),
            default_bluetooth_address: None,
            default_bluetooth_name: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum PlaybackState {
    Stopped,
    Connecting,
    Playing,
}

pub struct PlayerService {
    config_path: PathBuf,
    config: PlayerConfig,
    current_directory: PathBuf,
    folders: Vec<LibraryEntry>,
    tracks: Vec<LibraryEntry>,
    picker_directory: PathBuf,
    picker_folders: Vec<LibraryEntry>,
    picker_track_count: usize,
    current_track: Option<usize>,
    shuffle: bool,
    state: PlaybackState,
    audio_process: Option<Child>,
    connection_task: BackgroundTask<Result<(), String>>,
    message: Option<(String, bool)>,
    initialized: bool,
}

impl Default for PlayerService {
    fn default() -> Self {
        let config_path = env::var_os(PLAYER_CONFIG_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("config/player.yaml"));
        let (config, message) = load_config(&config_path);
        let current_directory = config.music_directory.clone();

        Self {
            config_path,
            config,
            current_directory,
            folders: Vec::new(),
            tracks: Vec::new(),
            picker_directory: PathBuf::new(),
            picker_folders: Vec::new(),
            picker_track_count: 0,
            current_track: None,
            shuffle: false,
            state: PlaybackState::Stopped,
            audio_process: None,
            connection_task: BackgroundTask::default(),
            message,
            initialized: false,
        }
    }
}

impl PlayerService {
    pub fn ensure_initialized(&mut self) {
        if !self.initialized {
            self.refresh_directory();
            self.initialized = true;
        }
    }

    pub fn poll(&mut self) {
        match self.connection_task.poll() {
            TaskPoll::Ready(Ok(())) => {
                self.start_audio_process();
            }
            TaskPoll::Ready(Err(error)) => {
                self.state = PlaybackState::Stopped;
                self.message = Some((error, true));
            }
            TaskPoll::Disconnected => {
                self.state = PlaybackState::Stopped;
                self.message = Some((
                    "Bluetooth connection task ended unexpectedly.".to_owned(),
                    true,
                ));
            }
            TaskPoll::Pending => {}
        }

        let process_result = self.audio_process.as_mut().map(Child::try_wait);
        match process_result {
            Some(Ok(Some(status))) => {
                self.audio_process = None;
                if status.success() {
                    self.play_next();
                } else {
                    self.state = PlaybackState::Stopped;
                    self.message = Some((format!("Audio player exited with {status}."), true));
                }
            }
            Some(Ok(None)) | None => {}
            Some(Err(error)) => {
                self.audio_process = None;
                self.state = PlaybackState::Stopped;
                self.message = Some((format!("Could not check music playback: {error}"), true));
            }
        }
    }

    pub fn folders(&self) -> &[LibraryEntry] {
        &self.folders
    }

    pub fn tracks(&self) -> &[LibraryEntry] {
        &self.tracks
    }

    pub fn current_directory(&self) -> &Path {
        &self.current_directory
    }

    pub fn can_go_up(&self) -> bool {
        self.current_directory != self.config.music_directory
    }

    pub fn enter_folder(&mut self, index: usize) {
        let Some(path) = self.folders.get(index).map(|folder| folder.path.clone()) else {
            return;
        };
        self.stop();
        self.current_directory = path;
        self.refresh_directory();
    }

    pub fn go_up(&mut self) {
        if !self.can_go_up() {
            return;
        }
        self.stop();
        if let Some(parent) = self.current_directory.parent() {
            self.current_directory = parent.to_owned();
        }
        self.refresh_directory();
    }

    pub fn refresh_directory(&mut self) {
        if self.state != PlaybackState::Stopped {
            self.stop();
        }
        self.folders.clear();
        self.tracks.clear();

        let entries = match fs::read_dir(&self.current_directory) {
            Ok(entries) => entries,
            Err(error) => {
                self.message = Some((
                    format!(
                        "Could not read music directory {}: {error}",
                        self.current_directory.display()
                    ),
                    true,
                ));
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                self.folders.push(LibraryEntry { name, path });
            } else if is_audio_file(&path) {
                self.tracks.push(LibraryEntry { name, path });
            }
        }
        self.folders.sort_by_key(|entry| entry.name.to_lowercase());
        self.tracks.sort_by_key(|entry| entry.name.to_lowercase());
        self.current_track = None;
        self.message = None;
    }

    pub fn play(&mut self, index: usize) {
        if index >= self.tracks.len() {
            return;
        }
        self.current_track = Some(index);
        self.prepare_current_track();
    }

    pub fn stop(&mut self) {
        self.connection_task.cancel();
        self.stop_audio_process();
        self.state = PlaybackState::Stopped;
    }

    pub fn play_next(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        let next = if self.shuffle {
            random_index(self.tracks.len(), self.current_track)
        } else {
            self.current_track
                .map_or(0, |index| (index + 1) % self.tracks.len())
        };
        self.play(next);
    }

    pub fn play_previous(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        let previous = self.current_track.map_or(0, |index| {
            if index == 0 {
                self.tracks.len() - 1
            } else {
                index - 1
            }
        });
        self.play(previous);
    }

    pub fn play_random(&mut self) {
        if !self.tracks.is_empty() {
            self.play(random_index(self.tracks.len(), self.current_track));
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
    }

    pub fn shuffle(&self) -> bool {
        self.shuffle
    }

    pub fn current_track_name(&self) -> Option<&str> {
        self.current_track
            .and_then(|index| self.tracks.get(index))
            .map(|track| track.name.as_str())
    }

    pub fn state_label(&self) -> &'static str {
        match self.state {
            PlaybackState::Stopped => "Stopped",
            PlaybackState::Connecting => "Connecting Bluetooth…",
            PlaybackState::Playing => "Playing",
        }
    }

    pub fn is_busy(&self) -> bool {
        self.state == PlaybackState::Connecting
    }

    pub fn message(&self) -> Option<(&str, bool)> {
        self.message
            .as_ref()
            .map(|(message, is_error)| (message.as_str(), *is_error))
    }

    pub fn default_bluetooth_name(&self) -> Option<&str> {
        self.config.default_bluetooth_name.as_deref()
    }

    pub fn default_bluetooth_address(&self) -> Option<&str> {
        self.config.default_bluetooth_address.as_deref()
    }

    pub fn set_default_bluetooth_device(&mut self, device: &BluetoothDevice) {
        self.config.default_bluetooth_address = Some(device.address.clone());
        self.config.default_bluetooth_name = Some(device.name.clone());
        self.save_config();
    }

    pub fn clear_default_bluetooth_device(&mut self) {
        self.config.default_bluetooth_address = None;
        self.config.default_bluetooth_name = None;
        self.save_config();
    }

    pub fn music_directory(&self) -> &Path {
        &self.config.music_directory
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn begin_directory_selection(&mut self) {
        self.picker_directory = if self.config.music_directory.is_dir() {
            self.config.music_directory.clone()
        } else {
            storage_locations()
                .into_iter()
                .next()
                .unwrap_or_else(|| PathBuf::from("/"))
        };
        self.refresh_picker_directory();
    }

    pub fn picker_directory(&self) -> &Path {
        &self.picker_directory
    }

    pub fn picker_folders(&self) -> &[LibraryEntry] {
        &self.picker_folders
    }

    pub fn picker_track_count(&self) -> usize {
        self.picker_track_count
    }

    pub fn picker_can_go_up(&self) -> bool {
        self.picker_directory.parent().is_some()
    }

    pub fn picker_go_up(&mut self) {
        if let Some(parent) = self.picker_directory.parent() {
            self.picker_directory = parent.to_owned();
            self.refresh_picker_directory();
        }
    }

    pub fn picker_enter_folder(&mut self, index: usize) {
        let Some(path) = self
            .picker_folders
            .get(index)
            .map(|folder| folder.path.clone())
        else {
            return;
        };
        self.picker_directory = path;
        self.refresh_picker_directory();
    }

    pub fn storage_locations(&self) -> Vec<PathBuf> {
        storage_locations()
    }

    pub fn picker_open_location(&mut self, path: &Path) {
        if path.is_dir() {
            self.picker_directory = path.to_owned();
            self.refresh_picker_directory();
        }
    }

    pub fn use_picker_directory(&mut self) {
        let path = self.picker_directory.clone();
        if !path.is_dir() {
            self.message = Some((
                format!("Music directory does not exist: {}", path.display()),
                true,
            ));
            return;
        }

        self.stop();
        self.config.music_directory = path.clone();
        self.current_directory = path;
        self.refresh_directory();
        self.save_config();
    }

    fn refresh_picker_directory(&mut self) {
        self.picker_folders.clear();
        self.picker_track_count = 0;

        let entries = match fs::read_dir(&self.picker_directory) {
            Ok(entries) => entries,
            Err(error) => {
                self.message = Some((
                    format!(
                        "Could not read folder {}: {error}",
                        self.picker_directory.display()
                    ),
                    true,
                ));
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                self.picker_folders.push(LibraryEntry { name, path });
            } else if is_audio_file(&path) {
                self.picker_track_count += 1;
            }
        }
        self.picker_folders
            .sort_by_key(|entry| entry.name.to_lowercase());
        self.message = None;
    }

    fn prepare_current_track(&mut self) {
        self.stop_audio_process();
        self.connection_task.cancel();
        self.message = None;

        let Some(address) = self.config.default_bluetooth_address.clone() else {
            self.state = PlaybackState::Stopped;
            self.message = Some((
                "Choose a default Bluetooth device in Player Settings before playing.".to_owned(),
                true,
            ));
            return;
        };

        self.state = PlaybackState::Connecting;
        self.connection_task
            .start(move || ensure_device_connected(&address));
    }

    fn start_audio_process(&mut self) {
        let Some(track_path) = self
            .current_track
            .and_then(|index| self.tracks.get(index))
            .map(|track| track.path.clone())
        else {
            self.state = PlaybackState::Stopped;
            return;
        };

        let players: [(&str, &[&str]); 4] = [
            ("mpv", &["--no-video", "--really-quiet"]),
            ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"]),
            ("cvlc", &["--intf", "dummy", "--play-and-exit"]),
            ("mpg123", &["--quiet"]),
        ];

        for (player, arguments) in players {
            match Command::new(player)
                .args(arguments)
                .arg(&track_path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(process) => {
                    self.audio_process = Some(process);
                    self.state = PlaybackState::Playing;
                    return;
                }
                Err(error) if error.kind() == ErrorKind::NotFound => continue,
                Err(error) => {
                    self.state = PlaybackState::Stopped;
                    self.message = Some((format!("Could not start {player}: {error}"), true));
                    return;
                }
            }
        }

        self.state = PlaybackState::Stopped;
        self.message = Some((
            "No supported music player found. Install mpv, ffplay, VLC, or mpg123.".to_owned(),
            true,
        ));
    }

    fn stop_audio_process(&mut self) {
        if let Some(mut process) = self.audio_process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }

    fn save_config(&mut self) {
        let result = (|| -> Result<(), String> {
            if let Some(parent) = self.config_path.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!("Could not create player configuration directory: {error}")
                })?;
            }
            let yaml = yaml_serde::to_string(&self.config)
                .map_err(|error| format!("Could not serialize player settings: {error}"))?;
            fs::write(&self.config_path, yaml)
                .map_err(|error| format!("Could not save player settings: {error}"))
        })();

        self.message = match result {
            Ok(()) => Some(("Player settings saved.".to_owned(), false)),
            Err(error) => Some((error, true)),
        };
    }
}

impl Drop for PlayerService {
    fn drop(&mut self) {
        self.stop_audio_process();
    }
}

fn load_config(path: &Path) -> (PlayerConfig, Option<(String, bool)>) {
    match fs::read_to_string(path) {
        Ok(yaml) => match yaml_serde::from_str(&yaml) {
            Ok(config) => (config, None),
            Err(error) => (
                PlayerConfig::default(),
                Some((format!("Invalid player configuration: {error}"), true)),
            ),
        },
        Err(error) if error.kind() == ErrorKind::NotFound => (PlayerConfig::default(), None),
        Err(error) => (
            PlayerConfig::default(),
            Some((
                format!("Could not read player configuration: {error}"),
                true,
            )),
        ),
    }
}

fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "mp3" | "flac" | "wav" | "ogg" | "opus" | "m4a" | "aac"
            )
        })
}

fn random_index(length: usize, current: Option<usize>) -> usize {
    if length <= 1 {
        return 0;
    }
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos() as usize);
    let mut index = seed % length;
    if Some(index) == current {
        index = (index + 1) % length;
    }
    index
}

fn default_music_directory() -> PathBuf {
    PathBuf::from("resources")
}

fn storage_locations() -> Vec<PathBuf> {
    ["/media", "/run/media", "/mnt", "/"]
        .into_iter()
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_supported_audio_files() {
        assert!(is_audio_file(Path::new("song.MP3")));
        assert!(is_audio_file(Path::new("song.flac")));
        assert!(!is_audio_file(Path::new("cover.png")));
    }

    #[test]
    fn shuffle_does_not_repeat_when_other_tracks_exist() {
        for _ in 0..10 {
            assert_ne!(random_index(3, Some(1)), 1);
        }
    }
}
