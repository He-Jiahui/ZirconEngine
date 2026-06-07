import { icon } from "../../../foundation/icons.js";
import { esc } from "../input-utils.js";

export function select(label, options = {}) {
  const leading = options.swatch ? '<span class="zr-select-swatch"></span>' : options.icon ? icon(options.icon) : "";
  return `<button class="zr-select ${options.open ? "is-open" : ""}" type="button" data-dropdown="${esc(options.menu ?? "")}">${leading}<span>${esc(label)}</span>${icon("chevronDown")}</button>`;
}
