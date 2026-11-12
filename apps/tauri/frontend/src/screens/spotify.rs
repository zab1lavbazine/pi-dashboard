use super::*;
use crate::ipc::{Client, Request};
#[component]
pub fn Spotify(client: Client, active: Signal<bool>) -> impl IntoView {
    view! {
        <section id="spotify" class="panel screen" aria-labelledby="spotify-title" hidden=move || !active.get()>
            <h2 id="spotify-title">"04 / SPOTIFY SERVICE"</h2>
            <p id="spotify-state" class="state" role="status">{move || client.status.with(|s| if s.spotify.running { "● RUNNING" } else { "○ NOT RUNNING / STATUS UNAVAILABLE" })}</p>
            <p class="notice">"A non-running result can also mean the service status check failed. Playback controls are not available."</p>
            <div class="actions"><button id="spotify-refresh" disabled=move || client.disabled(1) on:click=move |_| client.request(Request::Spotify)>"REFRESH STATUS"</button></div>
            <ErrorNotice id="spotify-error" message=Signal::derive(move || client.errors.get()[1].clone())/>
        </section>
    }
}
