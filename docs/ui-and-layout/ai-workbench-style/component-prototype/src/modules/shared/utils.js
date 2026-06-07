export const esc = (value) => String(value ?? "").replace(/[&<>"']/g, (char) => ({
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#039;"
}[char]));

export function tabKey(value) {
  return String(value).toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
}

export function titleCase(value) {
  return String(value)
    .split("-")
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

export function routeAttrs(options = {}) {
  const attrs = [];
  if (options.routeModule) {
    attrs.push(`data-route-module="${esc(options.routeModule)}"`);
  }
  if (options.routePanel) {
    attrs.push(`data-route-panel="${esc(options.routePanel)}"`);
  }
  return attrs.length ? ` ${attrs.join(" ")}` : "";
}
