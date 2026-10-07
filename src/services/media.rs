use serde::Deserialize;
use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    io::ErrorKind,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Arc,
};

const MEDIA_DIRECTORY_ENV: &str = "PI_DASHBOARD_MEDIA_DIR";
const RESOURCE_DIRECTORY_ENV: &str = "PI_DASHBOARD_RESOURCE_DIR";

#[derive(Clone)]
pub struct MediaItem {
    name: String,
    gif_bytes: Arc<[u8]>,
    gif_uri: String,
    audio_path: Option<PathBuf>,
}

impl MediaItem {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn gif_bytes(&self) -> Arc<[u8]> {
        Arc::clone(&self.gif_bytes)
    }

    pub fn gif_uri(&self) -> &str {
        &self.gif_uri
    }
}

#[derive(Deserialize)]
struct MediaConfigFile {
    items: Vec<MediaConfig>,
}

#[derive(Deserialize)]
struct MediaConfig {
    name: String,
    gif: PathBuf,
    #[serde(default)]
    audio: Option<PathBuf>,
    #[serde(default)]
    order: i32,
    #[serde(default = "enabled_by_default")]
    enabled: bool,
}

struct LoadedMedia {
    item: MediaItem,
    order: i32,
}

pub struct MediaService {
    config_directory: PathBuf,
    resource_directory: PathBuf,
    items: Vec<MediaItem>,
    selected_index: Option<usize>,
    load_errors: Vec<String>,
    playback_error: Option<String>,
    audio_process: Option<Child>,
    player_active: bool,
    loaded: bool,
}

impl Default for MediaService {
    fn default() -> Self {
        Self {
            config_directory: env::var_os(MEDIA_DIRECTORY_ENV)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("config/media")),
            resource_directory: env::var_os(RESOURCE_DIRECTORY_ENV)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("resources")),
            items: Vec::new(),
            selected_index: None,
            load_errors: Vec::new(),
            playback_error: None,
            audio_process: None,
            player_active: false,
            loaded: false,
        }
    }
}

impl MediaService {
    pub fn ensure_loaded(&mut self) {
        if !self.loaded {
            self.reload();
        }
    }

    pub fn reload(&mut self) {
        self.stop_audio();
        self.player_active = false;
        self.selected_index = None;
        self.load_errors.clear();
        self.playback_error = None;

        let mut loaded_items = Vec::new();
        let directory_entries = match fs::read_dir(&self.config_directory) {
            Ok(entries) => entries,
            Err(error) => {
                self.items.clear();
                self.loaded = true;
                self.load_errors.push(format!(
                    "Could not read {}: {error}",
                    self.config_directory.display()
                ));
                return;
            }
        };

        let mut config_paths = directory_entries
            .filter_map(|entry| match entry {
                Ok(entry) => Some(entry.path()),
                Err(error) => {
                    self.load_errors
                        .push(format!("Could not read a directory entry: {error}"));
                    None
                }
            })
            .filter(|path| is_yaml_file(path))
            .collect::<Vec<_>>();
        config_paths.sort();

        for config_path in config_paths {
            match load_config_file(&config_path, &self.resource_directory) {
                Ok((items, errors)) => {
                    loaded_items.extend(items);
                    self.load_errors.extend(errors);
                }
                Err(error) => self.load_errors.push(error),
            }
        }

        loaded_items.sort_by(|left, right| {
            left.order
                .cmp(&right.order)
                .then_with(|| left.item.name.cmp(&right.item.name))
        });
        self.items = loaded_items.into_iter().map(|loaded| loaded.item).collect();
        self.loaded = true;
    }

    pub fn config_directory(&self) -> &Path {
        &self.config_directory
    }

    pub fn resource_directory(&self) -> &Path {
        &self.resource_directory
    }

    pub fn items(&self) -> &[MediaItem] {
        &self.items
    }

    pub fn load_errors(&self) -> &[String] {
        &self.load_errors
    }

    pub fn playback_error(&self) -> Option<&str> {
        self.playback_error.as_deref()
    }

    pub fn select(&mut self, index: usize) -> bool {
        if index >= self.items.len() {
            return false;
        }

        self.selected_index = Some(index);
        self.playback_error = None;
        true
    }

    pub fn selected_item(&self) -> Option<&MediaItem> {
        self.selected_index.and_then(|index| self.items.get(index))
    }

    pub fn set_player_active(&mut self, active: bool) {
        if self.player_active == active {
            return;
        }

        self.player_active = active;
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
                if self.player_active && !status.success() {
                    self.playback_error = Some(format!("Audio player exited with {status}."));
                }
            }
            Ok(None) => {}
            Err(error) => {
                self.audio_process = None;
                self.playback_error = Some(format!("Could not check audio playback: {error}"));
            }
        }
    }

    fn start_audio(&mut self) {
        let Some(audio_path) = self
            .selected_item()
            .and_then(|item| item.audio_path.as_ref())
            .cloned()
        else {
            return;
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
                    self.playback_error = Some(format!("Could not start {player}: {error}"));
                    return;
                }
            }
        }

        self.playback_error = Some(
            "No supported audio player found. Install mpg123, mpv, ffplay, or VLC.".to_owned(),
        );
    }

    fn stop_audio(&mut self) {
        if let Some(mut process) = self.audio_process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}

impl Drop for MediaService {
    fn drop(&mut self) {
        self.stop_audio();
    }
}

fn load_config_file(
    config_path: &Path,
    resource_directory: &Path,
) -> Result<(Vec<LoadedMedia>, Vec<String>), String> {
    let yaml = fs::read_to_string(config_path)
        .map_err(|error| format!("Could not read {}: {error}", config_path.display()))?;
    let config_file: MediaConfigFile = yaml_serde::from_str(&yaml)
        .map_err(|error| format!("Invalid YAML in {}: {error}", config_path.display()))?;

    let mut loaded_items = Vec::new();
    let mut errors = Vec::new();

    for (index, config) in config_file.items.into_iter().enumerate() {
        match load_item(config_path, resource_directory, config) {
            Ok(Some(item)) => loaded_items.push(item),
            Ok(None) => {}
            Err(error) => errors.push(format!(
                "{} item #{}: {error}",
                config_path.display(),
                index + 1
            )),
        }
    }

    Ok((loaded_items, errors))
}

fn load_item(
    config_path: &Path,
    resource_directory: &Path,
    config: MediaConfig,
) -> Result<Option<LoadedMedia>, String> {
    if !config.enabled {
        return Ok(None);
    }
    if config.name.trim().is_empty() {
        return Err("name cannot be empty.".to_owned());
    }

    let gif_path = resolve_path(resource_directory, &config.gif);
    let audio_path = config
        .audio
        .as_ref()
        .map(|path| resolve_path(resource_directory, path));

    let gif_bytes = fs::read(&gif_path)
        .map_err(|error| format!("Could not read GIF {}: {error}", gif_path.display()))?;
    if let Some(path) = &audio_path {
        fs::metadata(path)
            .map_err(|error| format!("Could not read audio {}: {error}", path.display()))?;
    }

    let mut hasher = DefaultHasher::new();
    config_path.hash(&mut hasher);
    gif_bytes.hash(&mut hasher);
    let gif_uri = format!("bytes://media/{:016x}.gif", hasher.finish());

    Ok(Some(LoadedMedia {
        item: MediaItem {
            name: config.name,
            gif_bytes: Arc::from(gif_bytes),
            gif_uri,
            audio_path,
        },
        order: config.order,
    }))
}

fn resolve_path(base_directory: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        base_directory.join(path)
    }
}

fn is_yaml_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("yaml") || extension.eq_ignore_ascii_case("yml")
            })
}

const fn enabled_by_default() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_media_items_from_a_yaml_list() {
        let config_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("config/media/polish_cow.yaml");
        let resource_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
        let (items, errors) =
            load_config_file(&config_path, &resource_directory).expect("configuration should load");

        assert!(errors.is_empty());
        assert!(items.iter().any(|item| item.item.name == "Polish Cow"));
    }
}
