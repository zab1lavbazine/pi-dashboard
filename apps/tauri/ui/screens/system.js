import { element, text, meter } from "../dom.js";
import { number, storage, ratio } from "../format.js";

export function renderSystem(status) {
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
}
