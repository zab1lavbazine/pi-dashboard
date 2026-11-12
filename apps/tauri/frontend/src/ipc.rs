use crate::model::*;
use gloo_timers::callback::Timeout;
use leptos::prelude::*;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen(
    inline_js = "export async function dashboardInvoke(command, args) { if (typeof window.__TAURI__?.core?.invoke !== 'function') throw new Error('Tauri IPC is unavailable. Launch this UI inside Tauri, not a browser.'); return await window.__TAURI__.core.invoke(command, args); }"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = dashboardInvoke)]
    async fn invoke(command: &str, args: JsValue) -> Result<JsValue, JsValue>;
}
#[derive(Clone)]
pub enum Request {
    Status,
    Volume(VolumeAction),
    Spotify,
    Bluetooth(BluetoothAction, Option<String>),
}
impl Request {
    fn service(&self) -> Option<usize> {
        match self {
            Self::Status => None,
            Self::Volume(_) => Some(0),
            Self::Spotify => Some(1),
            Self::Bluetooth(..) => Some(2),
        }
    }
    async fn invoke(&self) -> Result<DashboardStatus, String> {
        fn args(value: &impl Serialize) -> Result<JsValue, String> {
            serde_wasm_bindgen::to_value(value).map_err(|e| e.to_string())
        }
        let (command, value) = match self {
            Self::Status => ("dashboard_status", JsValue::UNDEFINED),
            Self::Volume(action) => ("volume_action", args(&ActionArgs { action })?),
            Self::Spotify => (
                "spotify_action",
                args(&ActionArgs {
                    action: SpotifyAction::Refresh,
                })?,
            ),
            Self::Bluetooth(action, address) => (
                "bluetooth_action",
                args(&BluetoothArgs {
                    action: *action,
                    address: address.clone(),
                })?,
            ),
        };
        let value = invoke(command, value).await.map_err(|e| {
            e.as_string()
                .or_else(|| {
                    e.dyn_ref::<js_sys::Error>()
                        .map(|e| String::from(e.message()))
                })
                .unwrap_or_else(|| format!("{e:?}"))
        })?;
        serde_wasm_bindgen::from_value(value)
            .map_err(|e| format!("Invalid dashboard snapshot: {e}"))
    }
}
pub fn poll_delay(connected: bool, volume_busy: bool, bluetooth_busy: bool) -> u32 {
    if !connected {
        2000
    } else if volume_busy {
        50
    } else if bluetooth_busy {
        250
    } else {
        750
    }
}
#[derive(Clone, Copy)]
pub struct Client {
    pub status: RwSignal<DashboardStatus>,
    pub connected: RwSignal<bool>,
    pub in_flight: RwSignal<bool>,
    pub errors: RwSignal<[String; 3]>,
    pub error: RwSignal<String>,
    pub updated: RwSignal<String>,
    timer: StoredValue<Rc<RefCell<Option<Timeout>>>, LocalStorage>,
}
impl Client {
    pub fn new() -> Self {
        let timer = Rc::new(RefCell::new(None));
        let timer = StoredValue::new_local(timer);
        on_cleanup(move || {
            timer.with_value(|t| {
                t.borrow_mut().take();
            });
        });
        Self {
            status: RwSignal::new(DashboardStatus::default()),
            connected: RwSignal::new(false),
            in_flight: RwSignal::new(false),
            errors: RwSignal::new(Default::default()),
            error: RwSignal::new(String::new()),
            updated: RwSignal::new("NO SAMPLE RECEIVED".into()),
            timer,
        }
    }
    pub fn disabled(self, service: usize) -> bool {
        self.in_flight.get()
            || !self.connected.get()
            || self.status.with(|s| match service {
                0 => s.volume.busy,
                2 => s.bluetooth.busy,
                _ => false,
            })
    }
    pub fn request(self, request: Request) {
        if self.in_flight.get_untracked() {
            return;
        }
        self.timer.with_value(|t| {
            t.borrow_mut().take();
        });
        self.in_flight.set(true);
        spawn_local(async move {
            let result = request.invoke().await;
            if self.status.is_disposed() {
                return;
            }
            match result {
                Ok(status) => {
                    if let Some(i) = request.service() {
                        self.errors.update(|e| e[i].clear());
                    }
                    self.status.set(status);
                    self.connected.set(true);
                    self.error.set(String::new());
                    self.updated.set(format!(
                        "IPC / {}",
                        js_sys::Date::new_0()
                            .to_locale_time_string("en-GB")
                            .as_string()
                            .unwrap_or_default()
                    ));
                }
                Err(message) => {
                    if let Some(i) = request.service().filter(|_| self.connected.get_untracked()) {
                        self.errors
                            .update(|e| e[i] = format!("Action failed: {message}"));
                    } else {
                        self.connected.set(false);
                        self.error.set(format!(
                            "{message} Last displayed readings may be stale. Retrying…"
                        ));
                    }
                }
            }
            self.in_flight.set(false);
            let delay = self.status.with_untracked(|s| {
                poll_delay(
                    self.connected.get_untracked(),
                    s.volume.busy,
                    s.bluetooth.busy,
                )
            });
            self.timer.with_value(|t| {
                *t.borrow_mut() = Some(Timeout::new(delay, move || self.request(Request::Status)))
            });
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn polling_priorities() {
        assert_eq!(poll_delay(false, true, true), 2000);
        assert_eq!(poll_delay(true, true, true), 50);
        assert_eq!(poll_delay(true, false, true), 250);
        assert_eq!(poll_delay(true, false, false), 750);
    }
}
