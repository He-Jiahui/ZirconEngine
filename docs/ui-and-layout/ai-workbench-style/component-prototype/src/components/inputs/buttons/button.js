import { icon } from "../../../foundation/icons.js";
import { esc } from "../input-utils.js";

export function button(label, options = {}) {
  const classes = ["zr-button"];
  if (options.kind) classes.push(`is-${options.kind}`);
  if (options.disabled) classes.push("is-disabled");
  return `<button class="${classes.join(" ")}" type="button" ${options.disabled ? "disabled" : ""}>${options.icon ? icon(options.icon) : ""}<span>${esc(label)}</span></button>`;
}
