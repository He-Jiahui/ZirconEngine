export function commandLabel(target) {
  const explicit = target.dataset.action || target.dataset.module;
  if (explicit) return explicit.replace(/-/g, " ");
  return target.getAttribute("aria-label")
    || target.getAttribute("title")
    || target.textContent.trim().replace(/\s+/g, " ")
    || target.closest(".zr-module-setting")?.querySelector("span")?.textContent.trim()
    || "command";
}

export function fieldLabel(target) {
  return target.getAttribute("aria-label")
    || target.getAttribute("placeholder")
    || target.value
    || target.closest(".zr-module-setting")?.querySelector("span")?.textContent.trim()
    || "field";
}
