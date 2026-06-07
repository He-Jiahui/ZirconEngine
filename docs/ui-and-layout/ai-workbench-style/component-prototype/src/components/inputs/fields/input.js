import { esc } from "../input-utils.js";

export function input(placeholder, options = {}) {
  const classes = ["zr-input"];
  if (options.focused) classes.push("is-focused");
  return `<input class="${classes.join(" ")}" value="${esc(options.value ?? "")}" placeholder="${esc(placeholder)}" ${options.disabled ? "disabled" : ""} />`;
}
