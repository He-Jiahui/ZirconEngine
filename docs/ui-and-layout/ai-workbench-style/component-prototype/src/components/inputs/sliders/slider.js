import { numberField } from "../fields.js";
import { esc } from "../input-utils.js";

export function slider(label, value, number, stepped = false) {
  return `<div class="zr-slider ${stepped ? "is-stepped" : ""}" style="--value:${value}%"><span>${esc(label)}</span><span class="zr-slider-track"><span class="zr-slider-fill"></span><span class="zr-slider-thumb"></span></span>${numberField(number)}</div>`;
}
