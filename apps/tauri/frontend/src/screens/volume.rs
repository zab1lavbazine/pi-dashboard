use super::*;
use crate::{
    ipc::{Client, Request},
    model::VolumeAction,
};
#[component]
pub fn Volume(client: Client) -> impl IntoView {
    let level = Signal::derive(move || client.status.with(|s| s.volume.level.map(f64::from)));
    let message = move || {
        client.errors.with(|e| {
            if !e[0].is_empty() {
                e[0].clone()
            } else {
                client.status.with(|s| {
                    let v = &s.volume;
                    v.error
                        .clone()
                        .filter(|v| !v.is_empty())
                        .or_else(|| v.busy_label.clone().filter(|v| !v.is_empty()))
                        .or_else(|| v.message.clone().filter(|v| !v.is_empty()))
                        .unwrap_or_else(|| {
                            if v.muted == Some(true) {
                                "Output muted".into()
                            } else {
                                "Output ready".into()
                            }
                        })
                })
            }
        })
    };
    view! {
        <section class="panel audio" aria-labelledby="audio-title">
            <h2 id="audio-title">"02 / AUDIO OUTPUT"</h2>
            <span id="backend" class="eyebrow">{move || client.status.with(|s| s.volume.backend.as_ref().filter(|b| !b.is_empty()).map(|b| format!("{b} / DEFAULT OUTPUT")).unwrap_or_else(|| "BACKEND UNAVAILABLE".into()))}</span>
            <div id="volume" class="volume">{move || number(level.get(), "%", 0)}</div>
            <Meter id="volume-bar" label="Output volume" value=level/>
            <div class="volume-buttons">
                <button data-action="down" aria-label="Decrease volume" disabled=move || client.disabled(0) on:click=move |_| client.request(Request::Volume(VolumeAction::Down))>"−"</button>
                <button data-action="up" aria-label="Increase volume" disabled=move || client.disabled(0) on:click=move |_| client.request(Request::Volume(VolumeAction::Up))>"+"</button>
            </div>
            <button id="mute" data-action="mute" aria-pressed=move || client.status.with(|s| (s.volume.muted == Some(true)).to_string()) disabled=move || client.disabled(0) on:click=move |_| client.request(Request::Volume(VolumeAction::Mute))>{move || client.status.with(|s| match s.volume.muted { None => "TOGGLE MUTE", Some(true) => "UNMUTE OUTPUT", Some(false) => "MUTE OUTPUT" })}</button>
            <button data-action="refresh" class="secondary" disabled=move || client.disabled(0) on:click=move |_| client.request(Request::Volume(VolumeAction::Refresh))>"REFRESH OUTPUT"</button>
            <p id="volume-status" class="notice" role="status" class:error=move || !client.errors.get()[0].is_empty() || client.status.with(|s| s.volume.error.as_ref().is_some_and(|e| !e.is_empty()))>{message}</p>
        </section>
    }
}
