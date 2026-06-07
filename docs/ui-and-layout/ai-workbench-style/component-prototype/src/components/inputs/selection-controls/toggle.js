import { esc } from "../input-utils.js";

export function toggle(label, checked = true) {
  return `<button class="zr-switch ${checked ? "is-on" : ""}" type="button" data-toggle="switch"><span>${esc(label)}</span><span class="zr-switch-track"><span class="zr-switch-thumb"></span></span></button>`;
}
