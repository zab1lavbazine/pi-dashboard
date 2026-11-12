import { element, text, showError } from "../dom.js";

export function setupSpotify(request) {
  element("spotify-refresh").addEventListener("click", () => void request("spotify_action", { action: "refresh" }));
}

export function renderSpotify(status, actionErrors) {
  text("spotify-state", status.spotify.running ? "● RUNNING" : "○ NOT RUNNING / STATUS UNAVAILABLE");
  showError("spotify-error", actionErrors.spotify);
}
