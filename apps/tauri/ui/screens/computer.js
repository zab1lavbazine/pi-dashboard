import { element, text, showError } from "../dom.js";

export function renderComputer(status) {
  const computer = status.computer;
  text("computer-state", computer.connected ? "● CONNECTED" : computer.connecting ? "◌ CONNECTING" : "○ DISCONNECTED");
  text("computer-url", computer.url);
  showError("computer-error", computer.error);
  const fields = Object.entries(computer.fields || {});
  element("computer-empty").hidden = fields.length > 0;
  text("computer-empty", computer.fields === null
    ? "Waiting for system status…"
    : "The computer sent an empty status message.");
  // Only replace telemetry nodes when their contents actually change.
  const signature = JSON.stringify(fields);
  if (element("computer-fields").dataset.signature !== signature) {
    element("computer-fields").dataset.signature = signature;
    const nodes = [];
    for (const [key, value] of fields) {
      const term = document.createElement("dt");
      const description = document.createElement("dd");
      term.textContent = key;
      description.textContent = value;
      nodes.push(term, description);
    }
    element("computer-fields").replaceChildren(...nodes);
  }
}
