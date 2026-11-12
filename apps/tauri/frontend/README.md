# Leptos CSR frontend

Standalone Rust/WASM replacement for `../ui`, without modifying that UI or the Tauri backend. The empty `[workspace]` isolates dependency resolution from the repository workspace. `src/main.rs` is an application **binary**, mounted with `leptos::mount::mount_to_body`; no `cdylib` or library entry is necessary for Trunk.

## Build

Run in `apps/tauri/frontend`:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cargo check --locked --target wasm32-unknown-unknown
cargo test --locked
trunk build --release
```

Use stable Rust supporting edition 2024 and the locked dependency MSRVs (validated with Rust 1.99.1). Trunk consumes `index.html`, compiles the binary, runs its matching wasm-bindgen tooling, copies CSS, and emits `dist/`. Release builds request `wasm-opt -Oz` through `data-wasm-opt="z"`; Trunk may download wasm-bindgen/Binaryen tools if they are not cached, requiring separate network access. `trunk serve` can preview rendering, but a plain browser cannot invoke Tauri IPC and intentionally displays an error/retries.

### Tauri integration is deliberately not applied

The existing backend still points `build.frontendDist` at `../ui`. To activate this frontend, a separate integration change must point it at `../frontend/dist` and build that directory first. Development can use a Trunk dev URL with matching Tauri dev configuration. Keep `app.withGlobalTauri = true`; the bridge requires `window.__TAURI__.core.invoke`.

**CSP must also be integrated deliberately:** the existing policy only allows `script-src 'self'`. WebAssembly compilation generally needs `'wasm-unsafe-eval'`, and Trunk's generated inline module bootstrap needs an approved hash/nonce (or an externalized bootstrap permitted by `'self'`). Inspect the generated output and use a matching bootstrap hash/nonce, rather than broadly relaxing CSP. The inline-js wasm-bindgen bridge becomes an emitted JS module, not an `eval` of backend text. Preserve the existing restricted IPC `connect-src`. No backend config or security policy is changed here.

## Dependencies

`Cargo.lock` records exact direct and transitive versions. Direct resolved versions at implementation:

| Dependency | Version | Purpose/features |
| --- | --- | --- |
| leptos | 0.8.23 | CSR components, signals, keyed lists (`csr`) |
| serde | 1.0.229 | Snapshot deserialization, typed action serialization (`derive`) |
| serde-wasm-bindgen | 0.6.5 | JS snapshot/argument conversion |
| wasm-bindgen | 0.2.130 | Caught async inline-JS bridge |
| wasm-bindgen-futures | 0.4.80 | Local async requests |
| gloo-timers | 0.3.0 | Cancellable poll timeouts (`futures` enabled) |
| js-sys | 0.3.107 | Error and timestamp handling |
| web-sys | 0.3.107 | Window / removal confirmation (`Window`) |

No npm dependencies, old UI JavaScript, shell API, arbitrary command input, or backend crate dependency is used. `style.css` is an unchanged copy of `../ui/style.css`.

## Contract and behavior

`src/model.rs` mirrors all fields in `../src-tauri/src/snapshots.rs`, including optional telemetry and computer field maps. The only invocations are:

- `dashboard_status`, without arguments.
- `volume_action`, `{ action: "up" | "down" | "mute" | "refresh" }`.
- `spotify_action`, `{ action: "refresh" }`.
- `bluetooth_action`, `{ action: "refresh" | "scan" }`, or `{ action: "select" | "pair" | "trust" | "connect" | "disconnect" | "remove", address }`.

All resolve to a dashboard snapshot. Rust enums constrain actions; command names are private fixed literals. Backend validation remains authoritative.

One client serializes polling and actions: requests arriving while another invocation is pending are dropped, not queued. Each completion cancels/replaces the one-shot timer: disconnected **2000 ms**, otherwise volume busy **50 ms**, otherwise Bluetooth busy **250 ms**, otherwise **750 ms**. Actions cancel the pending poll. Volume, Bluetooth, and Spotify controls have the same per-service busy/connection gates as the original. Action errors persist until a successful action for that service, while IPC errors retain old readings and retry. Cleanup cancels timers and ignores results after disposal; it cannot cancel an already dispatched backend invocation.

Screens stay mounted across navigation. Bluetooth rows are keyed by exact address and read current device data reactively, preserving nodes/focus across polls and confirming removal with the latest name. Computer fields only reconcile when the field map changes. All backend strings render as text, never HTML.

Spotify remains status-only because that is all the backend supports. Computer telemetry remains read-only. No playback, connection configuration, or other unsupported commands are invented.

## Validation / limitations

Native `cargo check`, WASM-target `cargo check`, a full debug `cargo build --target wasm32-unknown-unknown`, and all three native unit tests were run successfully. Tests cover polling priorities, unavailable/numeric telemetry formatting, and Bluetooth action availability. The copied stylesheet was verified byte-identical with `cmp`. The WASM target was initially absent and was installed for validation. Trunk is not installed in the validation environment, so generated Trunk assets, CSP integration, and live Tauri/device interactions were not exercised. Native tests cover pure Rust behavior, not browser DOM or Tauri runtime behavior. The old backend still serves the old UI until the explicit integration described above.
