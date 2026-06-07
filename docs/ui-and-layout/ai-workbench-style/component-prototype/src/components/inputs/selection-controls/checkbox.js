import { icon } from "../../../foundation/icons.js";
import { esc } from "../input-utils.js";

export function checkbox(label, checked = false) {
  return `<button class="zr-checkbox ${checked ? "is-checked" : ""}" type="button" data-toggle="check"><span class="zr-check-box">${checked ? icon("check") : ""}</span><span>${esc(label)}</span></button>`;
}
