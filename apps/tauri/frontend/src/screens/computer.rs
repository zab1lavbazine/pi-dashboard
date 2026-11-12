use super::*;
use crate::ipc::Client;
#[component]
pub fn Computer(client: Client, active: Signal<bool>) -> impl IntoView {
    let fields = Memo::new(move |_| {
        client
            .status
            .with(|s| s.computer.fields.clone().unwrap_or_default())
    });
    view! {
        <section id="computer" class="panel screen" aria-labelledby="computer-title" hidden=move || !active.get()>
            <h2 id="computer-title">"03 / COMPUTER TELEMETRY"</h2>
            <p id="computer-state" class="state" role="status">{move || client.status.with(|s| if s.computer.connected { "● CONNECTED" } else if s.computer.connecting { "◌ CONNECTING" } else { "○ DISCONNECTED" })}</p>
            <p id="computer-url" class="notice">{move || client.status.with(|s| s.computer.url.clone())}</p>
            <ErrorNotice id="computer-error" message=Signal::derive(move || client.status.with(|s| s.computer.error.clone().unwrap_or_default()))/>
            <p id="computer-empty" class="notice" hidden=move || !fields.get().is_empty()>{move || client.status.with(|s| if s.computer.fields.is_none() { "Waiting for system status…" } else { "The computer sent an empty status message." })}</p>
            <dl id="computer-fields" class="fields"><For each={move || fields.get().into_iter().collect::<Vec<_>>()} key=|(key, value)| (key.clone(), value.clone()) children=|(key,value)| view! { <dt>{key}</dt><dd>{value}</dd> }/></dl>
        </section>
    }
}
