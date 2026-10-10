use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::Duration,
};

use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::Value;
use tokio_tungstenite::connect_async;

const COMPUTER_CONFIG_ENV: &str = "PI_DASHBOARD_COMPUTER_CONFIG";
const RECONNECT_DELAY: Duration = Duration::from_secs(3);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, Default)]
pub struct ComputerStatus {
    pub fields: BTreeMap<String, String>,
}

enum ComputerEvent {
    Connecting(String),
    Connected(String),
    Status(ComputerStatus),
    ConnectionError(String),
    MessageError(String),
}

pub struct ComputerService {
    url: String,
    connected: bool,
    connecting: bool,
    snapshot: Option<ComputerStatus>,
    error: Option<String>,
    receiver: Receiver<ComputerEvent>,
}

impl Default for ComputerService {
    fn default() -> Self {
        let target = load_target();
        let url = target.description();
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    let _ = sender.send(ComputerEvent::ConnectionError(format!(
                        "Could not start WebSocket runtime: {error}"
                    )));
                    return;
                }
            };
            runtime.block_on(run_client(target, sender));
        });

        Self {
            url,
            connected: false,
            connecting: true,
            snapshot: None,
            error: None,
            receiver,
        }
    }
}

impl ComputerService {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn is_connecting(&self) -> bool {
        self.connecting
    }

    pub fn snapshot(&self) -> Option<&ComputerStatus> {
        self.snapshot.as_ref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        loop {
            match self.receiver.try_recv() {
                Ok(ComputerEvent::Connecting(url)) => {
                    self.url = url;
                    self.connecting = true;
                    self.connected = false;
                    changed = true;
                }
                Ok(ComputerEvent::Connected(url)) => {
                    self.url = url;
                    self.connecting = false;
                    self.connected = true;
                    self.error = None;
                    changed = true;
                }
                Ok(ComputerEvent::Status(status)) => {
                    self.snapshot = Some(status);
                    self.error = None;
                    changed = true;
                }
                Ok(ComputerEvent::ConnectionError(error)) => {
                    self.connecting = false;
                    self.connected = false;
                    self.error = Some(error);
                    changed = true;
                }
                Ok(ComputerEvent::MessageError(error)) => {
                    self.error = Some(error);
                    changed = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.connecting = false;
                    self.connected = false;
                    self.error = Some("Computer monitor stopped.".to_owned());
                    break;
                }
            }
        }
        changed
    }
}

#[derive(Clone, Debug, Deserialize)]
struct ComputerConfig {
    #[serde(default)]
    computer_name: String,
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_path")]
    path: String,
}

impl Default for ComputerConfig {
    fn default() -> Self {
        Self {
            computer_name: String::new(),
            port: default_port(),
            path: default_path(),
        }
    }
}

enum ComputerTarget {
    Direct(String),
    Tailscale(ComputerConfig),
    ConfigError(String),
}

impl ComputerTarget {
    fn description(&self) -> String {
        match self {
            Self::Direct(url) => url.clone(),
            Self::Tailscale(config) if config.computer_name.trim().is_empty() => {
                "Automatic Tailscale peer".to_owned()
            }
            Self::Tailscale(config) => format!("Tailscale: {}", config.computer_name),
            Self::ConfigError(_) => "Invalid computer configuration".to_owned(),
        }
    }

    fn resolve_url(&self) -> Result<String, String> {
        match self {
            Self::Direct(url) => Ok(url.clone()),
            Self::Tailscale(config) => resolve_tailscale_url(config),
            Self::ConfigError(error) => Err(error.clone()),
        }
    }
}

fn default_port() -> u16 {
    8080
}

fn default_path() -> String {
    "/ws".to_owned()
}

fn load_target() -> ComputerTarget {
    if let Ok(url) = env::var("PI_DASHBOARD_COMPUTER_WS_URL") {
        return ComputerTarget::Direct(url);
    }

    let path = env::var_os(COMPUTER_CONFIG_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config/computer.yaml"));
    match load_config(&path) {
        Ok(config) => ComputerTarget::Tailscale(config),
        Err(error) => ComputerTarget::ConfigError(error),
    }
}

fn load_config(path: &Path) -> Result<ComputerConfig, String> {
    let yaml = fs::read_to_string(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    yaml_serde::from_str(&yaml)
        .map_err(|error| format!("Could not parse {}: {error}", path.display()))
}

fn resolve_tailscale_url(config: &ComputerConfig) -> Result<String, String> {
    let output = Command::new("tailscale")
        .args(["status", "--json"])
        .output()
        .map_err(|error| format!("Could not run `tailscale status --json`: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Tailscale status failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let status: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Could not parse Tailscale status: {error}"))?;
    resolve_url_from_status(&status, config)
}

fn resolve_url_from_status(status: &Value, config: &ComputerConfig) -> Result<String, String> {
    let peers = status
        .get("Peer")
        .and_then(Value::as_object)
        .ok_or_else(|| "Tailscale status did not contain any peers.".to_owned())?;
    let requested = config.computer_name.trim();

    let matching_peers: Vec<&serde_json::Map<String, Value>> = peers
        .values()
        .filter_map(Value::as_object)
        .filter(|peer| requested.is_empty() || peer_matches_name(peer, requested))
        .collect();

    let peer = match matching_peers.as_slice() {
        [peer] => *peer,
        [] if requested.is_empty() => {
            return Err("No Tailscale peer was found. Check `tailscale status`.".to_owned());
        }
        [] => {
            return Err(format!(
                "Tailscale device `{requested}` was not found. Check its machine name with `tailscale status`."
            ));
        }
        _ if requested.is_empty() => {
            return Err(
                "More than one Tailscale peer was found; set computer_name in config/computer.yaml."
                    .to_owned(),
            );
        }
        _ => {
            return Err(format!(
                "More than one Tailscale device matched `{requested}`."
            ));
        }
    };

    let ip = peer
        .get("TailscaleIPs")
        .and_then(Value::as_array)
        .and_then(|ips| {
            ips.iter()
                .filter_map(Value::as_str)
                .find(|ip| ip.contains('.'))
        })
        .or_else(|| {
            peer.get("TailscaleIPs")
                .and_then(Value::as_array)
                .and_then(|ips| ips.iter().filter_map(Value::as_str).next())
        })
        .ok_or_else(|| "The selected Tailscale device has no IP address.".to_owned())?;
    let host = if ip.contains(':') {
        format!("[{ip}]")
    } else {
        ip.to_owned()
    };
    let path = if config.path.starts_with('/') {
        config.path.clone()
    } else {
        format!("/{}", config.path)
    };
    Ok(format!("ws://{host}:{}{path}", config.port))
}

fn peer_matches_name(peer: &serde_json::Map<String, Value>, requested: &str) -> bool {
    ["HostName", "DNSName"]
        .iter()
        .filter_map(|key| peer.get(*key).and_then(Value::as_str))
        .any(|name| {
            name.trim_end_matches('.').eq_ignore_ascii_case(requested)
                || name
                    .split('.')
                    .next()
                    .is_some_and(|short| short.eq_ignore_ascii_case(requested))
        })
}

async fn run_client(target: ComputerTarget, sender: mpsc::Sender<ComputerEvent>) {
    loop {
        let url = match target.resolve_url() {
            Ok(url) => url,
            Err(error) => {
                if sender.send(ComputerEvent::ConnectionError(error)).is_err() {
                    return;
                }
                tokio::time::sleep(RECONNECT_DELAY).await;
                continue;
            }
        };
        if sender.send(ComputerEvent::Connecting(url.clone())).is_err() {
            return;
        }

        match tokio::time::timeout(CONNECT_TIMEOUT, connect_async(&url)).await {
            Ok(Ok((mut stream, _))) => {
                if sender.send(ComputerEvent::Connected(url.clone())).is_err() {
                    return;
                }

                while let Some(message) = stream.next().await {
                    match message {
                        Ok(message) if message.is_text() => {
                            match parse_status(message.to_text().unwrap_or_default()) {
                                Ok(status) => {
                                    if sender.send(ComputerEvent::Status(status)).is_err() {
                                        return;
                                    }
                                }
                                Err(error) => {
                                    let _ = sender.send(ComputerEvent::MessageError(error));
                                }
                            }
                        }
                        Ok(message) if message.is_close() => break,
                        Ok(_) => {}
                        Err(error) => {
                            let _ = sender.send(ComputerEvent::ConnectionError(format!(
                                "WebSocket connection lost: {error}"
                            )));
                            break;
                        }
                    }
                }
            }
            Ok(Err(error)) => {
                let _ = sender.send(ComputerEvent::ConnectionError(format!(
                    "Could not connect to {url}: {error}"
                )));
            }
            Err(_) => {
                let _ = sender.send(ComputerEvent::ConnectionError(format!(
                    "Connection to {url} timed out"
                )));
            }
        }

        tokio::time::sleep(RECONNECT_DELAY).await;
    }
}

fn parse_status(text: &str) -> Result<ComputerStatus, String> {
    let value: Value =
        serde_json::from_str(text).map_err(|error| format!("Invalid status message: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "Status message must be a JSON object.".to_owned())?;

    let fields = object
        .iter()
        .map(|(name, value)| (friendly_name(name), display_value(name, value)))
        .collect();
    Ok(ComputerStatus { fields })
}

fn friendly_name(name: &str) -> String {
    name.replace(['_', '-'], " ")
        .split_whitespace()
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_value(name: &str, value: &Value) -> String {
    match value {
        Value::Number(number)
            if name.eq_ignore_ascii_case("cpu")
                || name.eq_ignore_ascii_case("ram")
                || name.eq_ignore_ascii_case("disk") =>
        {
            format!("{number}%")
        }
        Value::Number(number) if name.to_ascii_lowercase().ends_with("_gb") => {
            format!("{number} GB")
        }
        Value::String(value) => value.clone(),
        Value::Null => "Unavailable".to_owned(),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_and_additional_status_fields() {
        let status =
            parse_status(r#"{"cpu":24.5,"ram":61,"host_name":"office-pc"}"#).expect("valid status");

        assert_eq!(status.fields.get("Cpu").map(String::as_str), Some("24.5%"));
        assert_eq!(status.fields.get("Ram").map(String::as_str), Some("61%"));
        assert_eq!(
            status.fields.get("Host Name").map(String::as_str),
            Some("office-pc")
        );
    }

    #[test]
    fn resolves_the_named_tailscale_peer() {
        let status: Value = serde_json::from_str(
            r#"{"Peer":{"node-id":{"HostName":"gaming-pc","DNSName":"gaming-pc.example.ts.net.","TailscaleIPs":["100.64.0.2","fd7a::2"]}}}"#,
        )
        .expect("valid Tailscale status");
        let config = ComputerConfig {
            computer_name: "gaming-pc".to_owned(),
            port: 8080,
            path: "/ws".to_owned(),
        };

        assert_eq!(
            resolve_url_from_status(&status, &config).as_deref(),
            Ok("ws://100.64.0.2:8080/ws")
        );
    }
}
