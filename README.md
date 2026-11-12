# Pi Dashboard

Pi Dashboard is a touchscreen-oriented Raspberry Pi application written in Rust with `eframe` and `egui`. It provides a single interface for media playback, Bluetooth management, volume and power controls, terminal output, Spotify service status, and an idle system-information overlay.

The default window is designed for an 800×480 display. Screens use large controls and scrollable cards so the main features can be operated without a keyboard.

The interface uses a retro green-phosphor CRT theme: monospace text, inverted title bars, square outlined controls, and subtle static scanlines. Full-screen media playback is shown without scanlines. The shared palette and visual settings live in `crates/egui/src/ui/theme.rs`; warning and error colors remain distinct for readability.

## Features

- Touch-friendly home screen and LIFO Back navigation.
- Bluetooth discovery, pairing, trust, connect, disconnect, and removal.
- Local music player with folder browsing for SD cards and USB flash drives.
- Default Bluetooth output selection with a connection check before playback.
- Track selection, previous/next, random play, shuffle, stop, and volume controls.
- YAML-configured GIF and audio buttons.
- Per-media looping or one-shot playback.
- Volume control through PipeWire, PulseAudio, or ALSA.
- Shutdown and reboot confirmation screens.
- Spotify Connect systemd-service status.
- Remote computer system status over WebSocket.
- Keyboard-operated terminal view.
- System information overlay after 10 seconds of inactivity on the Home screen.
- Shared background-task abstraction for non-blocking system operations.

## Project structure

```text
pi-dashboard/
├── config/
│   ├── media/              # YAML-defined media buttons
│   └── player.yaml         # Music folder and Bluetooth player settings
├── resources/              # GIF, audio, and other runtime assets
├── crates/
│   ├── core/               # UI-independent Rust library and service tests
│   │   └── src/            # Bluetooth, media, player, volume, etc.
│   └── egui/               # Default native GUI (binary: pi-dashboard)
│       └── src/            # Screens, theme, navigation, entry point
├── apps/
│   └── tauri/              # Experimental Tauri v2 GUI
│       ├── src-tauri/      # Rust IPC adapter using the shared core
│       └── ui/             # Local HTML/CSS/JS; no npm required
├── .cargo/config.toml      # AArch64 cross-linker configuration
├── Cargo.toml
└── Makefile
```

Screens contain presentation logic. System commands, filesystem access, configuration loading, playback, and background operations belong in `pi-dashboard-core`. The core has no GUI dependencies. `pi-dashboard` depends on this library and owns the egui lifecycle and navigation.

The root Cargo workspace contains two packages, `pi-dashboard-core` and `pi-dashboard`, with egui as the default member. Existing `cargo run` and deployment commands continue to use the egui application. Run commands from the repository root to preserve relative configuration/resource paths.

```sh
cargo run -p pi-dashboard
cargo test -p pi-dashboard-core
cargo check --workspace
cargo test --workspace
```

### Try the Tauri frontend

The optional Tauri v2 application uses the same core library through a small Rust IPC adapter. It currently implements **system metrics, volume, Computer telemetry, Spotify service status, and Bluetooth management**, not full feature parity with egui. It preserves the green CRT design and uses bundled local HTML/CSS/JS without Node or npm.

Install the native prerequisites described in [`apps/tauri/README.md`](apps/tauri/README.md), then:

```sh
cargo install tauri-cli --version '^2' --locked
make run-tauri
```

Tauri has an isolated Cargo workspace/lockfile so the default egui build does not require WebKitGTK or Tauri. On Windows, WebView2 and MSVC build tools are required; the existing Linux-oriented metrics/audio backends are not native Windows integrations. On Raspberry Pi, Tauri additionally requires GTK 3 and WebKitGTK 4.1; the existing egui cross-deployment workflow does not deploy Tauri.

## Requirements

Build requirements:

- Rust toolchain with Cargo.
- `make` for the provided workflows.
- `aarch64-linux-gnu-gcc` for cross-compiling to a 64-bit Raspberry Pi.
- The Rust `aarch64-unknown-linux-gnu` target.

Add the Rust target with:

```bash
rustup target add aarch64-unknown-linux-gnu
```

Runtime functionality uses Linux commands supplied by these components:

- BlueZ: `bluetoothctl`
- PipeWire, PulseAudio, or ALSA: `wpctl`, `pactl`, or `amixer`
- One audio player: `mpv`, `ffplay`, VLC (`cvlc`), or `mpg123`
- systemd: `systemctl`
- Optional Raspberry Pi temperature fallback: `vcgencmd`

`mpv` is the preferred general-purpose audio player.

## Run locally

Run either frontend from the repository root:

```bash
make run-egui    # Full-featured native egui application
make run-tauri   # Experimental Tauri UI (requires tauri-cli v2 and native prerequisites)
make run         # Alias for make run-egui
```

Without Make, run `cargo run -p pi-dashboard` from the root for egui, or
`cargo tauri dev` from `apps/tauri` for Tauri.

Other local commands:

```bash
make build       # Local release build
make check       # Formatting check, cargo check, and tests
make test        # Tests only
make clean       # Remove Cargo build artifacts
```

Avoid running `cargo clean` before every build. The `eframe`/`wgpu` graphics stack is large and a complete rebuild can take a long time.

## Build and deploy to Raspberry Pi

The default deployment settings are:

```make
PI_TARGET = aarch64-unknown-linux-gnu
PI_HOST = bareldan@debil4ik
PI_DIR = /home/bareldan/pi-dashboard
```

Cross-compile the application:

```bash
make build-pi
```

Build from a clean target directory only when necessary:

```bash
make build-pi-clean
```

Upload the binary as `pi-dashboard.new`:

```bash
make upload-pi
```

Upload the binary, media configuration, resources, and atomically activate the new binary:

```bash
make deploy-pi
```

Override the destination without editing the Makefile:

```bash
make deploy-pi \
  PI_HOST=user@raspberrypi \
  PI_DIR=/opt/pi-dashboard
```

Normal deployment preserves the Pi's `config/player.yaml` because that file stores runtime selections. Replace it explicitly with:

```bash
make upload-player-config
```

Replacing the executable does not restart an already-running process. Restart the application or its systemd service after deployment.

## Deployment layout

Configuration and resources are loaded at runtime and are not embedded in the executable. Keep this layout on the Pi:

```text
/home/bareldan/pi-dashboard/
├── pi-dashboard
├── config/
│   ├── player.yaml
│   └── media/
│       └── polish_cow.yaml
└── resources/
    ├── polish_cow/
    │   ├── polish_cow.gif
    │   └── polish_cow.mp3
    └── first_of_the_month/
        ├── first_of_the_month.gif
        └── first_of_the_month.mp3
```

The default paths are relative to the application's working directory. When launching manually:

```bash
cd /home/bareldan/pi-dashboard
./pi-dashboard
```

For a systemd service, set:

```ini
[Service]
WorkingDirectory=/home/bareldan/pi-dashboard
ExecStart=/home/bareldan/pi-dashboard/pi-dashboard
```

Paths can also be overridden:

```bash
PI_DASHBOARD_MEDIA_DIR=/opt/pi-dashboard/config/media \
PI_DASHBOARD_RESOURCE_DIR=/opt/pi-dashboard/resources \
PI_DASHBOARD_PLAYER_CONFIG=/opt/pi-dashboard/config/player.yaml \
./pi-dashboard
```

The Computer screen normally discovers its WebSocket host through Tailscale.
Configure `config/computer.yaml`:

```yaml
# Empty automatically selects the only other device in a two-device tailnet.
computer_name: ""
port: 8080
path: /ws
```

Set `computer_name` to the target's Tailscale machine name when the tailnet has
multiple peers. The dashboard runs `tailscale status --json`, finds the matching
peer, and connects to its Tailscale IP. The name may be the short hostname shown
by `tailscale status` or its MagicDNS name.

The configuration path can be changed with
`PI_DASHBOARD_COMPUTER_CONFIG`. A complete URL can also override Tailscale
discovery for troubleshooting:

```bash
PI_DASHBOARD_COMPUTER_WS_URL=ws://192.168.1.20:8080/ws ./pi-dashboard
```

Each text message must be a JSON object. The screen displays every top-level
field; numeric `cpu` and `ram` values are formatted as percentages. For example:

```json
{"hostname":"office-pc","cpu":24.5,"ram":61,"temperature":54}
```

The companion Rust application in `windows-system-monitor/` collects and sends
these values from Windows. Its README contains foreground testing, native
Windows Service installation, and firewall instructions.

## Configurable media

The application reads every `.yaml` and `.yml` file from `config/media`. Each file can define multiple buttons:

```yaml
items:
  - name: Polish Cow
    gif: polish_cow/polish_cow.gif
    audio: polish_cow/polish_cow.mp3
    order: 10
    enabled: true
    repeat: true

  - name: First of the month
    gif: first_of_the_month/first_of_the_month.gif
    audio: first_of_the_month/first_of_the_month.mp3
    order: 20
    enabled: true
    repeat: false
```

Relative asset paths are resolved from the `resources` directory. Absolute paths are also accepted.

Media fields:

- `name`: button label; required.
- `gif`: GIF path; required.
- `audio`: optional audio path.
- `order`: numeric display order; defaults to `0`.
- `enabled`: whether the item is loaded; defaults to `true`.
- `repeat`: whether playback loops; defaults to `true`.

For `repeat: false`, items with audio remain open until the audio player finishes. GIF-only items use the total duration of their GIF frames. Touching a full-screen media item returns to the Media list. Press **Reload** after changing YAML files.

## Music player

The player configuration is stored in `config/player.yaml`:

```yaml
music_directory: resources
default_bluetooth_address: null
default_bluetooth_name: null
```

The file is updated when settings are changed from the touchscreen.

To use music from removable storage:

1. Mount the SD card or USB flash drive in Linux.
2. Open **Player → Settings → Browse storage**.
3. Choose `/media`, `/run/media`, or `/mnt`.
4. Navigate to the drive's `Music` folder.
5. Press **Use this folder**.

The player recognizes MP3, FLAC, WAV, OGG, Opus, M4A, and AAC files. It displays tracks from the selected folder and allows navigation into its subfolders.

Before playing each track, the player checks the configured Bluetooth device. If it is paired but disconnected, the application connects it before starting audio. Pair devices from the Bluetooth screen before selecting them in Player Settings.

## Navigation and idle overlay

Navigation uses a LIFO screen stack:

```text
Home → Player → Settings → Bluetooth
Back → Settings → Player → Home
```

Opening Home clears the history. New screen transitions should use:

```rust
app.navigate_to(Screen::PlayerSettings);
app.go_back();
app.go_home();
```

Do not assign `app.screen` directly for normal navigation.

The system-information overlay appears only while Home has been idle for 10 seconds. Touching the display dismisses it and restores Home without losing an in-progress service operation.

## Background tasks

One-result system operations use `BackgroundTask<T>` from `crates/core/src/task.rs`. It centralizes thread spawning, channel polling, busy-state protection, disconnected-worker detection, and receiver cancellation.

Audio playback is managed separately because it is a long-running child process that must be stopped explicitly.

## Troubleshooting

### Media configuration cannot be found

Confirm that the process working directory contains both `config/` and `resources/`, or set the three `PI_DASHBOARD_*` environment variables shown above.

### A media item is missing

The Media screen reports invalid YAML and missing asset paths. Every configured GIF and audio file must exist on the Pi.

### Bluetooth connection is slow

Refresh and Scan intentionally enumerate known devices; Scan waits up to eight seconds. Connect/pair/trust actions update only the selected device. A speaker that is asleep may still require several seconds to wake.

### Music does not play

Check that:

- A supported audio player is installed.
- The default Bluetooth device is paired and selected.
- The flash card is mounted and the saved music directory still exists.
- The application user has permission to access audio and Bluetooth services.

### Cross-linking fails

Confirm that the Rust AArch64 target and `aarch64-linux-gnu-gcc` are installed. The linker is configured in `.cargo/config.toml`.
