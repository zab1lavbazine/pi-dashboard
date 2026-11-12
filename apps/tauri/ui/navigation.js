import { element } from "./dom.js";

export function setupNavigation() {
for (const button of document.querySelectorAll("button[data-screen]")) {
  button.addEventListener("click", () => {
    for (const tab of document.querySelectorAll("button[data-screen]")) {
      const active = tab === button;
      element(tab.dataset.screen).hidden = !active;
      if (active) tab.setAttribute("aria-current", "page");
      else tab.removeAttribute("aria-current");
    }
  });
}
}
