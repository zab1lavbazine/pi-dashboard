use super::*;
use crate::ipc::Client;

#[component]
pub fn System(client: Client) -> impl IntoView {
    let system = Memo::new(move |_| client.status.with(|s| s.system.clone().unwrap_or_default()));
    let cpu = Signal::derive(move || system.with(|s| s.cpu_percent.map(f64::from)));
    let memory =
        Signal::derive(move || system.with(|s| ratio(s.memory_used_bytes, s.memory_total_bytes)));
    let disk =
        Signal::derive(move || system.with(|s| ratio(s.disk_used_bytes, s.disk_total_bytes)));
    view! {
        <section class="panel system" aria-labelledby="system-title">
            <h2 id="system-title">"01 / SYSTEM MONITOR"</h2>
            <div class="identity"><strong id="hostname">{move || system.with(|s| if s.hostname.is_empty() { "HOST UNKNOWN".into() } else { s.hostname.clone() })}</strong><span id="ip">{move || system.with(|s| s.ip_address.clone().unwrap_or_else(|| "IP UNAVAILABLE".into()))}</span></div>
            <div class="metrics">
                <article><h3>"CPU USAGE"</h3><strong id="cpu">{move || number(cpu.get(), "%", 1)}</strong><Meter id="cpu-bar" label="CPU usage" value=cpu/></article>
                <article><h3>"THERMAL"</h3><strong id="temperature">{move || system.with(|s| number(s.temperature_celsius.map(f64::from), " °C", 1))}</strong><span class="caption">"SOC TEMPERATURE"</span></article>
                <article><h3>"MEMORY"</h3><strong id="memory">{move || system.with(|s| storage(s.memory_used_bytes, s.memory_total_bytes))}</strong><Meter id="memory-bar" label="Memory usage" value=memory/></article>
                <article><h3>"ROOT DISK"</h3><strong id="disk">{move || system.with(|s| storage(s.disk_used_bytes, s.disk_total_bytes))}</strong><Meter id="disk-bar" label="Root disk usage" value=disk/></article>
            </div>
            <div class="details"><span>"UPTIME "<b id="uptime">{move || system.with(|s| s.uptime_seconds.map(|v| format!("{}d {}h {}m", v/86400, v/3600%24, v/60%60)).unwrap_or_else(|| "—".into()))}</b></span><span>"LOAD / 1M "<b id="load">{move || system.with(|s| number(s.load_average.map(f64::from), "", 2))}</b></span></div>
            <p id="system-status" class="notice" class:error=move || client.status.with(|s| s.system_error.as_ref().is_some_and(|e| !e.is_empty())) role="status">{move || client.status.with(|s| s.system_error.clone().filter(|e| !e.is_empty()).unwrap_or_else(|| if s.system_loading { "Collecting first sample…".into() } else { "LIVE / unavailable readings shown as —".into() }))}</p>
        </section>
    }
}
