"use strict";

const element = (id) => document.getElementById(id);
const text = (id, value) => { element(id).textContent = value; };
const buttons = [...document.querySelectorAll("button[data-action]")];
let inFlight = false;
let connected = false;
let volumeBusy = true;
let timer;

function controls() {
  for (const button of buttons) button.disabled = inFlight || !connected || volumeBusy;
}

function meter(id, value) {
  const bar = element(id);
  bar.hidden = !Number.isFinite(value);
  bar.value = Number.isFinite(value) ? Math.max(0, Math.min(100, value)) : 0;
}

function number(value, suffix = "", digits = 1) {
  return Number.isFinite(value) ? `${value.toFixed(digits)}${suffix}` : "—";
}

function storage(used, total) {
  if (!Number.isFinite(used) || !Number.isFinite(total) || total <= 0) return "—";
  return `${(used / 1073741824).toFixed(1)} / ${(total / 1073741824).toFixed(1)} GiB`;
}

function ratio(used, total) {
  return Number.isFinite(used) && Number.isFinite(total) && total > 0 ? used / total * 100 : null;
}

function render(status) {
  const system = status.system;
  text("hostname", system?.hostname || "HOST UNKNOWN");
  text("ip", system?.ip_address ?? "IP UNAVAILABLE");
  text("cpu", number(system?.cpu_percent, "%"));
  text("temperature", number(system?.temperature_celsius, " °C"));
  text("memory", storage(system?.memory_used_bytes, system?.memory_total_bytes));
  text("disk", storage(system?.disk_used_bytes, system?.disk_total_bytes));
  meter("cpu-bar", system?.cpu_percent);
  meter("memory-bar", ratio(system?.memory_used_bytes, system?.memory_total_bytes));
  meter("disk-bar", ratio(system?.disk_used_bytes, system?.disk_total_bytes));
  const seconds = system?.uptime_seconds;
  text("uptime", Number.isFinite(seconds) ? `${Math.floor(seconds / 86400)}d ${Math.floor(seconds / 3600) % 24}h ${Math.floor(seconds / 60) % 60}m` : "—");
  text("load", number(system?.load_average, "", 2));
  text("system-status", status.system_error || (status.system_loading ? "Collecting first sample…" : "LIVE / unavailable readings shown as —"));
  element("system-status").classList.toggle("error", Boolean(status.system_error));
  const volume = status.volume;
  volumeBusy = volume.busy;
  text("backend", volume.backend ? `${volume.backend} / DEFAULT OUTPUT` : "BACKEND UNAVAILABLE");
  text("volume", number(volume.level, "%", 0));
  meter("volume-bar", volume.level);
  text("mute", volume.muted === null ? "TOGGLE MUTE" : volume.muted ? "UNMUTE OUTPUT" : "MUTE OUTPUT");
  element("mute").setAttribute("aria-pressed", String(volume.muted === true));
  text("volume-status", volume.error || volume.busy_label || volume.message || (volume.muted ? "Output muted" : "Output ready"));
  element("volume-status").classList.toggle("error", Boolean(volume.error));
  text("updated", `IPC / ${new Date().toLocaleTimeString()}`);
}

async function request(command, args) {
  if (inFlight) return;
  clearTimeout(timer);
  inFlight = true;
  controls();
  try {
    if (typeof window.__TAURI__?.core?.invoke !== "function") {
      throw new Error("Tauri IPC is unavailable. Launch this UI with cargo tauri dev, not a browser.");
    }
    const status = await window.__TAURI__.core.invoke(command, args);
    render(status);
    connected = true;
    text("connection", "● CONNECTED");
    element("error").hidden = true;
  } catch (error) {
    connected = false;
    text("connection", "○ IPC ERROR");
    text("error", `${error instanceof Error ? error.message : String(error)} Last displayed readings may be stale. Retrying…`);
    element("error").hidden = false;
  } finally {
    inFlight = false;
    controls();
    timer = setTimeout(() => { void request("dashboard_status"); }, connected ? 750 : 2000);
  }
}

for (const button of buttons) {
  button.addEventListener("click", () => {
    void request("volume_action", { action: button.dataset.action });
  });
}
void request("dashboard_status");
