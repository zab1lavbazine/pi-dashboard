mod ipc;
mod model;
mod screens;
use ipc::{Client, Request};
use leptos::prelude::*;
use screens::{
    bluetooth::Bluetooth, computer::Computer, spotify::Spotify, system::System, volume::Volume,
};

#[component]
fn App() -> impl IntoView {
    let client = Client::new();
    let screen = RwSignal::new("overview");
    client.request(Request::Status);
    view! {
        <main>
            <header><div><span class="eyebrow">"LOCAL TELEMETRY / EXPERIMENTAL"</span><h1>"PI DASHBOARD"<span class="cursor">"_"</span></h1></div><span id="connection" role="status">{move || if client.connected.get() { "● CONNECTED" } else if client.error.get().is_empty() { "CONNECTING" } else { "○ IPC ERROR" }}</span></header>
            <nav aria-label="Dashboard screens">{["overview", "computer", "spotify", "bluetooth"].into_iter().map(move |name| view! {
                <button data-screen=name aria-controls=name aria-current=move || (screen.get() == name).then_some("page") on:click=move |_| screen.set(name)>{name.to_uppercase()}</button>
            }).collect_view()}</nav>
            <div id="overview" class="layout" aria-label="Overview" hidden=move || screen.get() != "overview"><System client=client/><Volume client=client/></div>
            <Computer client=client active=Signal::derive(move || screen.get() == "computer")/>
            <Spotify client=client active=Signal::derive(move || screen.get() == "spotify")/>
            <Bluetooth client=client active=Signal::derive(move || screen.get() == "bluetooth")/>
            <footer><span>"LOCAL CONTROL / LIVE TELEMETRY"</span><span id="updated">{move || client.updated.get()}</span></footer>
            <screens::ErrorNotice id="error" message=client.error.into()/>
        </main>
    }
}
fn main() {
    leptos::mount::mount_to_body(App);
}
