import { esc } from "../input-utils.js";

export function radio(label, checked = false) {
  return `<button class="zr-radio ${checked ? "is-checked" : ""}" type="button" data-radio><span class="zr-radio-mark"></span><span>${esc(label)}</span></button>`;
}
