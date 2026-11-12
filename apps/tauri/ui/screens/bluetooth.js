import { element, text, showError } from "../dom.js";

export function createBluetoothScreen(request) {
  const devices = new Map();
  for (const button of document.querySelectorAll(".actions > button[data-bluetooth]")) {
    button.addEventListener("click", () => void request("bluetooth_action", { action: button.dataset.bluetooth }));
  }
  return function renderBluetooth(status, actionErrors) {
  const bluetooth = status.bluetooth;
  const bluetoothBusy = bluetooth.busy;
  element("bluetooth").setAttribute("aria-busy", String(bluetoothBusy));
  text("bluetooth-status", bluetooth.busy_label || bluetooth.message || (bluetoothBusy ? "Working…" : "Ready / select a device or scan"));
  showError("bluetooth-error", actionErrors.bluetooth || bluetooth.error);
  element("bluetooth-empty").hidden = bluetooth.devices.length > 0;
  const addresses = new Set(bluetooth.devices.map((device) => device.address));
  for (const [address, row] of devices) {
    if (!addresses.has(address)) { row.remove(); devices.delete(address); }
  }
  for (const device of bluetooth.devices) {
    let row = devices.get(device.address);
    if (!row) {
      row = document.createElement("li");
      const name = document.createElement("strong");
      const detail = document.createElement("p");
      const actions = document.createElement("div");
      actions.className = "actions";
      for (const action of ["select", "pair", "trust", "connect", "disconnect", "remove"]) {
        const button = document.createElement("button");
        button.dataset.bluetooth = action;
        button.textContent = action.toUpperCase();
        button.addEventListener("click", () => {
          if (action === "remove" && !window.confirm(`Remove ${row.device.name || device.address} (${device.address})? This forgets the device; pairing may be required again.`)) return;
          void request("bluetooth_action", { action, address: device.address });
        });
        actions.append(button);
      }
      row.append(name, detail, actions);
      devices.set(device.address, row);
      element("bluetooth-devices").append(row);
    }
    row.device = device;
    row.children[0].textContent = device.name || "Unnamed device";
    row.children[1].textContent = `${device.address} / ${device.paired ? "PAIRED" : "UNPAIRED"} / ${device.trusted ? "TRUSTED" : "UNTRUSTED"} / ${device.connected ? "CONNECTED" : "DISCONNECTED"}`;
    const selected = bluetooth.selected_address?.toLowerCase() === device.address.toLowerCase();
    row.classList.toggle("selected", selected);
    for (const button of row.querySelectorAll("button")) {
      const action = button.dataset.bluetooth;
      button.setAttribute("aria-label", `${action} ${device.name || device.address}`);
      if (action === "select") button.setAttribute("aria-pressed", String(selected));
      button.dataset.unavailable = String((action === "pair" && device.paired) || (action === "trust" && device.trusted) || (action === "connect" && device.connected) || (action === "disconnect" && !device.connected));
    }
  }
  };
}
