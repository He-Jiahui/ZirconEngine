import { icon } from "../../../foundation/icons.js";
import { esc } from "../input-utils.js";

export function iconButton(name, label, options = {}) {
  const classes = ["zr-icon-button"];
  if (options.active) classes.push("is-active");
  if (options.large) classes.push("is-lg");
  if (options.danger) classes.push("is-danger");
  return `<button class="${classes.join(" ")}" type="button" title="${esc(label)}" aria-label="${esc(label)}">${icon(name)}</button>`;
}
