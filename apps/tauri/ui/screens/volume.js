import { element, text, meter } from "../dom.js";
import { number } from "../format.js";

export function setupVolume(request) {
  for (const button of document.querySelectorAll("button[data-action]")) {
    button.addEventListener("click", () => void request("volume_action", { action: button.dataset.action }));
  }
}

export function renderVolume(status, actionErrors) {
  const volume = status.volume;
  text("backend", volume.backend ? `${volume.backend} / DEFAULT OUTPUT` : "BACKEND UNAVAILABLE");
  text("volume", number(volume.level, "%", 0));
  meter("volume-bar", volume.level);
  text("mute", volume.muted === null ? "TOGGLE MUTE" : volume.muted ? "UNMUTE OUTPUT" : "MUTE OUTPUT");
  element("mute").setAttribute("aria-pressed", String(volume.muted === true));
  text("volume-status", volume.error || volume.busy_label || volume.message || (volume.muted ? "Output muted" : "Output ready"));
  element("volume-status").classList.toggle("error", Boolean(volume.error));
  if (actionErrors.volume) text("volume-status", actionErrors.volume);
  element("volume-status").classList.toggle("error", Boolean(volume.error || actionErrors.volume));
}
