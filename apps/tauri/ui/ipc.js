export function pollDelay({ connected, volumeBusy, bluetoothBusy }) {
  return !connected ? 2000 : volumeBusy ? 50 : bluetoothBusy ? 250 : 750;
}

// Requests arriving during an invocation are ignored, not queued.
export function createIpcClient({ invoke, render, controls, success, failure,
  setTimer = setTimeout, clearTimer = clearTimeout }) {
  const state = { inFlight: false, connected: false, volumeBusy: true, bluetoothBusy: true };
  const actionErrors = { bluetooth: "", spotify: "", volume: "" };
  let timer;
  async function request(command, args) {
    if (state.inFlight) return;
    clearTimer(timer);
    state.inFlight = true;
    controls(state);
    try {
      const status = await invoke(command, args);
      const service = command.replace("_action", "");
      if (service in actionErrors) actionErrors[service] = "";
      state.volumeBusy = status.volume.busy;
      state.bluetoothBusy = status.bluetooth.busy;
      render(status, actionErrors);
      state.connected = true;
      success();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      const service = command.replace("_action", "");
      if (command !== "dashboard_status" && state.connected && service in actionErrors) {
        actionErrors[service] = `Action failed: ${message}`;
        failure(service, actionErrors[service]);
      } else {
        state.connected = false;
        failure(null, message);
      }
    } finally {
      state.inFlight = false;
      controls(state);
      timer = setTimer(() => { void request("dashboard_status"); }, pollDelay(state));
    }
  }
  return { request };
}

export function invokeTauri(command, args) {
  if (typeof window.__TAURI__?.core?.invoke !== "function") {
    throw new Error("Tauri IPC is unavailable. Launch this UI with cargo tauri dev, not a browser.");
  }
  return window.__TAURI__.core.invoke(command, args);
}
