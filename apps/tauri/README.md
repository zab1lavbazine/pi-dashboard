# Experimental Tauri v2 frontend

Standalone, npm-free experiment. All frontend assets are local vanilla HTML/CSS/JS
in `ui/`; Tauri embeds that directory for production. The existing egui frontend
and shared core are unchanged. `src-tauri` has its own `[workspace]` and is already
excluded by the repository root workspace. Its core dependency is
`../../../crates/core`, relative to `src-tauri`.

## Prerequisites and running

Use a current stable Rust toolchain supporting edition 2024 and the dependencies'
MSRV. Install the v2 CLI:

```sh
cargo install tauri-cli --version '^2' --locked
```

- **Windows:** install Microsoft C++ Build Tools with the Desktop development
  with C++ workload, Windows SDK, and the MSVC Rust toolchain. Microsoft Edge
  WebView2 Runtime is required. The existing core's metrics/audio implementation
  targets Linux: a Windows build does not provide native Windows telemetry or
  volume integration. Missing readings/backends remain unavailable or errors.
- **Linux / Raspberry Pi OS:** install a compiler toolchain, `pkg-config`, GTK 3
  and WebKitGTK **4.1** development packages. On supported Debian/Ubuntu releases:
  `sudo apt install build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev libayatana-appindicator3-dev patchelf`.
  Package names vary by distribution; older WebKitGTK 4.0 is not sufficient.
  A graphical desktop/session is required. Volume requires `wpctl`, `pactl`, or
  `amixer`, plus access to the user's audio session. Core metrics use Linux
  `/proc`, `/sys`, `df`, and optional `vcgencmd`/network tools.

From the repository root:

```sh
make run-tauri
```

Without Make, change to `apps/tauri` and run `cargo tauri dev`.
Use `make run-egui` from the repository root to run the existing egui frontend instead.

Build without a platform bundle/installer:

```sh
cargo tauri build --no-bundle
```

No npm install, web server, external font, or remote frontend is involved.
The executable is normally under `src-tauri/target/release/`. The standalone
package resolves its own `Cargo.lock`; it does not use the root workspace lock.

## Limited scope and behavior

Only system metrics and default-output volume are implemented. There are **no**
media, Bluetooth, Spotify, terminal, shell, power, shutdown, or reboot controls.
No placeholder feature buttons or Tauri plugins are installed. Only the two
application commands `dashboard_status` and `volume_action` are registered;
volume actions accept exactly `up`, `down`, `mute`, and `refresh`. No arbitrary
program names or arguments cross IPC. The core volume service itself invokes
fixed audio-tool commands asynchronously.

Services live behind a Rust mutex. Status requests poll completed core tasks,
request freshness (core system interval: two seconds), and initialize volume.
The browser serializes all IPC, waits for completion before scheduling another
poll, disables controls during IPC/audio work, and catches failures. Actions
return the current state immediately; subsequent polls collect their result.
Volume refresh is explicit after initialization or external audio changes.
Unavailable readings are `null`/`—`, not fabricated zeros. The first CPU reading
needs two core samples. Existing readings may remain visible after failures;
error messages identify them as potentially stale. System loading means no
snapshot/error yet; the core exposes no public system refresh-busy flag.
Volume busy state and service errors are forwarded directly.

Security: local external script/style files, a restrictive CSP (no inline or
remote script/style), `textContent` for display values, no shell plugin and no
plugin capabilities. Tauri's global API is enabled solely for
`window.__TAURI__.core.invoke`. Production bundling is disabled. This is an
experimental local UI, not a hardened kiosk or replacement for the main app.

## Working directory and resources

This limited UI does not load media/player resources. The shared core's other
features use repository-root-relative `config/` and `resources/` paths; assets
are **not** automatically bundled by this experiment. For future integrations,
launch the built executable with the repository root as the working directory,
or set absolute paths using `PI_DASHBOARD_MEDIA_DIR`,
`PI_DASHBOARD_RESOURCE_DIR`, and `PI_DASHBOARD_PLAYER_CONFIG` (and
`PI_DASHBOARD_COMPUTER_CONFIG` for computer configuration). `cargo tauri dev`
may use the app/package working directory: do not assume it is the repository
root. Do not change core defaults merely to accommodate this frontend.

## Validation

From `apps/tauri/src-tauri`:

```sh
cargo fmt --check
cargo check
cargo test
```

Full checks need dependency downloads and the native prerequisites above.
A successful compile alone does not verify WebView rendering, Linux metrics,
audio permissions/backends, or physical 800×480 touch behavior. Test those on
the target desktop/Pi, including missing audio tools, rapid taps, refresh after
an external volume change, and recovery from backend failures.
