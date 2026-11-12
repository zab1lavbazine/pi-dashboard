export function number(value, suffix = "", digits = 1) {
  return Number.isFinite(value) ? `${value.toFixed(digits)}${suffix}` : "—";
}

export function storage(used, total) {
  if (!Number.isFinite(used) || !Number.isFinite(total) || total <= 0) return "—";
  return `${(used / 1073741824).toFixed(1)} / ${(total / 1073741824).toFixed(1)} GiB`;
}

export function ratio(used, total) {
  return Number.isFinite(used) && Number.isFinite(total) && total > 0 ? used / total * 100 : null;
}
