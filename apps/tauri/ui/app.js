import { element, text, showError } from "./dom.js";
import { setupNavigation } from "./navigation.js";
import { createIpcClient, invokeTauri } from "./ipc.js";
import { renderSystem } from "./screens/system.js";
import { renderVolume, setupVolume } from "./screens/volume.js";
import { renderComputer } from "./screens/computer.js";
import { renderSpotify, setupSpotify } from "./screens/spotify.js";
import { createBluetoothScreen } from "./screens/bluetooth.js";

const client = createIpcClient({
  invoke: invokeTauri,
  controls({ inFlight, connected, volumeBusy, bluetoothBusy }) {
    for (const button of document.querySelectorAll("button[data-action]")) {
      button.disabled = inFlight || !connected || volumeBusy;
    }
    element("spotify-refresh").disabled = inFlight || !connected;
    for (const button of document.querySelectorAll("button[data-bluetooth]")) {
      button.disabled = inFlight || !connected || bluetoothBusy || button.dataset.unavailable === "true";
    }
  },
  render(status, actionErrors) {
    renderSystem(status);
    renderVolume(status, actionErrors);
    renderComputer(status);
    renderSpotify(status, actionErrors);
    renderBluetooth(status, actionErrors);
    text("updated", `IPC / ${new Date().toLocaleTimeString()}`);
  },
  success() {
    text("connection", "● CONNECTED");
    element("error").hidden = true;
  },
  failure(service, message) {
    if (service === "volume") {
      text("volume-status", message);
      element("volume-status").classList.add("error");
    } else if (service) showError(`${service}-error`, message);
    else {
      text("connection", "○ IPC ERROR");
      showError("error", `${message} Last displayed readings may be stale. Retrying…`);
    }
  },
});

setupNavigation();
setupVolume(client.request);
setupSpotify(client.request);
const renderBluetooth = createBluetoothScreen(client.request);
void client.request("dashboard_status");
