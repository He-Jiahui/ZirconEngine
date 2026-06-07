import { numberField } from "../fields.js";
import { esc } from "../input-utils.js";

export function rangeSlider(label, min, max, minNumber, maxNumber) {
  return `<div class="zr-slider is-range" style="--min:${min}%;--value:${max}%;--max:${max}%"><span>${esc(label)}</span><span class="zr-slider-track"><span class="zr-slider-fill"></span><span class="zr-slider-thumb is-min"></span><span class="zr-slider-thumb is-max"></span>${numberField(minNumber, { className: "zr-range-min" })}</span>${numberField(maxNumber)}</div>`;
}
