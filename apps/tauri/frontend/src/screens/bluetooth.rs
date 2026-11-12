use super::*;
use crate::{
    ipc::{Client, Request},
    model::{BluetoothAction, BluetoothDevice},
};
#[component]
pub fn Bluetooth(client: Client, active: Signal<bool>) -> impl IntoView {
    view! {
        <section id="bluetooth" class="panel screen" aria-labelledby="bluetooth-title" hidden=move || !active.get() aria-busy=move || client.status.with(|s| s.bluetooth.busy.to_string())>
            <h2 id="bluetooth-title">"05 / BLUETOOTH DEVICES"</h2>
            <div class="actions">
                <button data-bluetooth="refresh" disabled=move || client.disabled(2) on:click=move |_| client.request(Request::Bluetooth(BluetoothAction::Refresh,None))>"REFRESH"</button>
                <button data-bluetooth="scan" disabled=move || client.disabled(2) on:click=move |_| client.request(Request::Bluetooth(BluetoothAction::Scan,None))>"SCAN FOR DEVICES"</button>
            </div>
            <p id="bluetooth-status" class="notice" role="status">{move || client.status.with(|s| s.bluetooth.busy_label.clone().filter(|v| !v.is_empty()).or_else(|| s.bluetooth.message.clone().filter(|v| !v.is_empty())).unwrap_or_else(|| if s.bluetooth.busy { "Working…".into() } else { "Ready / select a device or scan".into() }))}</p>
            <ErrorNotice id="bluetooth-error" message=Signal::derive(move || client.errors.with(|e| if !e[2].is_empty() { e[2].clone() } else { client.status.with(|s| s.bluetooth.error.clone().unwrap_or_default()) }))/>
            <p id="bluetooth-empty" class="notice" hidden=move || client.status.with(|s| !s.bluetooth.devices.is_empty())>"No devices found. Scan to discover nearby devices."</p>
            <ul id="bluetooth-devices" class="devices" aria-label="Bluetooth devices">
                <For each=move || client.status.with(|s| s.bluetooth.devices.iter().map(|d| d.address.clone()).collect::<Vec<_>>()) key=|address| address.clone() children=move |address| view! { <DeviceRow client=client address=address/> }/>
            </ul>
        </section>
    }
}
#[component]
fn DeviceRow(client: Client, address: String) -> impl IntoView {
    let address = StoredValue::new(address);
    // Read the latest snapshot inside each keyed row, rather than capturing its initial device.
    let device = Signal::derive(move || {
        client.status.with(|s| {
            s.bluetooth
                .devices
                .iter()
                .find(|d| address.with_value(|a| a == &d.address))
                .cloned()
                .unwrap_or_default()
        })
    });
    let selected = Signal::derive(move || {
        client.status.with(|s| {
            s.bluetooth
                .selected_address
                .as_ref()
                .is_some_and(|a| address.with_value(|b| a.eq_ignore_ascii_case(b)))
        })
    });
    view! {
        <li class:selected=move || selected.get()>
            <strong>{move || { let d = device.get(); if d.name.is_empty() { "Unnamed device".into() } else { d.name } }}</strong>
            <p>{move || { let d = device.get(); format!("{} / {} / {} / {}", d.address, if d.paired { "PAIRED" } else { "UNPAIRED" }, if d.trusted { "TRUSTED" } else { "UNTRUSTED" }, if d.connected { "CONNECTED" } else { "DISCONNECTED" }) }}</p>
            <div class="actions">{[BluetoothAction::Select, BluetoothAction::Pair, BluetoothAction::Trust, BluetoothAction::Connect, BluetoothAction::Disconnect, BluetoothAction::Remove].into_iter().map(move |action| view! {
                <button data-bluetooth=action.name()
                    aria-label=move || { let d = device.get(); format!("{} {}", action.name(), if d.name.is_empty() { d.address } else { d.name }) }
                    aria-pressed=move || matches!(action, BluetoothAction::Select).then(|| selected.get().to_string())
                    disabled=move || client.disabled(2) || unavailable(action, &device.get())
                    on:click=move |_| {
                        let d = device.get_untracked();
                        if matches!(action, BluetoothAction::Remove) {
                            let name = if d.name.is_empty() { &d.address } else { &d.name };
                            let message = format!("Remove {name} ({})? This forgets the device; pairing may be required again.", d.address);
                            if !web_sys::window().is_some_and(|w| w.confirm_with_message(&message).unwrap_or(false)) { return; }
                        }
                        client.request(Request::Bluetooth(action, Some(address.get_value())));
                    }>{action.name().to_uppercase()}</button>
            }).collect_view()}</div>
        </li>
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_action_availability_tracks_snapshot() {
        let mut device = BluetoothDevice::default();
        assert!(!unavailable(BluetoothAction::Pair, &device));
        assert!(!unavailable(BluetoothAction::Trust, &device));
        assert!(!unavailable(BluetoothAction::Connect, &device));
        assert!(unavailable(BluetoothAction::Disconnect, &device));
        device.paired = true;
        device.trusted = true;
        device.connected = true;
        assert!(unavailable(BluetoothAction::Pair, &device));
        assert!(unavailable(BluetoothAction::Trust, &device));
        assert!(unavailable(BluetoothAction::Connect, &device));
        assert!(!unavailable(BluetoothAction::Disconnect, &device));
        assert!(!unavailable(BluetoothAction::Select, &device));
        assert!(!unavailable(BluetoothAction::Remove, &device));
    }
}

fn unavailable(action: BluetoothAction, d: &BluetoothDevice) -> bool {
    match action {
        BluetoothAction::Pair => d.paired,
        BluetoothAction::Trust => d.trusted,
        BluetoothAction::Connect => d.connected,
        BluetoothAction::Disconnect => !d.connected,
        _ => false,
    }
}
