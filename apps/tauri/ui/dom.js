export const element = (id) => document.getElementById(id);
export const text = (id, value) => { element(id).textContent = value; };

export function meter(id, value) {
  const bar = element(id);
  bar.hidden = !Number.isFinite(value);
  bar.value = Number.isFinite(value) ? Math.max(0, Math.min(100, value)) : 0;
}

export function showError(id, message) {
  text(id, message || "");
  element(id).hidden = !message;
}
