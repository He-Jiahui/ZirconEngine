import { icon } from "./icons.js";

const esc = (value) => String(value ?? "").replace(/[&<>"']/g, (char) => ({
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#039;"
}[char]));

export function iconButton(name, label, options = {}) {
  const classes = ["zr-icon-button"];
  if (options.active) classes.push("is-active");
  if (options.large) classes.push("is-lg");
  if (options.danger) classes.push("is-danger");
  return `<button class="${classes.join(" ")}" type="button" title="${esc(label)}" aria-label="${esc(label)}">${icon(name)}</button>`;
}

export function button(label, options = {}) {
  const classes = ["zr-button"];
  if (options.kind) classes.push(`is-${options.kind}`);
  if (options.disabled) classes.push("is-disabled");
  return `<button class="${classes.join(" ")}" type="button" ${options.disabled ? "disabled" : ""}>${options.icon ? icon(options.icon) : ""}<span>${esc(label)}</span></button>`;
}

export function input(placeholder, options = {}) {
  const classes = ["zr-input"];
  if (options.focused) classes.push("is-focused");
  return `<input class="${classes.join(" ")}" value="${esc(options.value ?? "")}" placeholder="${esc(placeholder)}" ${options.disabled ? "disabled" : ""} />`;
}

export function searchInput(placeholder) {
  return `<label class="zr-search">${icon("search")}${input(placeholder)}</label>`;
}

export function checkbox(label, checked = false) {
  return `<button class="zr-checkbox ${checked ? "is-checked" : ""}" type="button" data-toggle="check"><span class="zr-check-box">${checked ? icon("check") : ""}</span><span>${esc(label)}</span></button>`;
}

export function radio(label, checked = false) {
  return `<button class="zr-radio ${checked ? "is-checked" : ""}" type="button" data-radio><span class="zr-radio-mark"></span><span>${esc(label)}</span></button>`;
}

export function toggle(label, checked = true) {
  return `<button class="zr-switch ${checked ? "is-on" : ""}" type="button" data-toggle="switch"><span>${esc(label)}</span><span class="zr-switch-track"><span class="zr-switch-thumb"></span></span></button>`;
}

export function tabs(items, active = 0, className = "zr-tabs") {
  return `<div class="${className}" role="tablist">${items.map((item, index) => {
    const content = typeof item === "object" ? `${item.icon ? icon(item.icon) : ""}${item.label ? esc(item.label) : ""}` : esc(item);
    return `<button class="${className === "zr-segment" ? "zr-segment-item" : "zr-tab"} ${index === active ? "is-active" : ""}" type="button" role="tab" aria-selected="${index === active ? "true" : "false"}">${content}</button>`;
  }).join("")}</div>`;
}

export function select(label, options = {}) {
  const leading = options.swatch ? '<span class="zr-select-swatch"></span>' : options.icon ? icon(options.icon) : "";
  return `<button class="zr-select ${options.open ? "is-open" : ""}" type="button" data-dropdown="${esc(options.menu ?? "")}">${leading}<span>${esc(label)}</span>${icon("chevronDown")}</button>`;
}

export function numberField(value, options = {}) {
  const classes = ["zr-number"];
  if (options.stepper) classes.push("has-stepper");
  if (options.className) classes.push(options.className);
  return `<span class="${classes.join(" ")}">${esc(value)}${options.stepper ? '<span class="zr-number-stepper"><span></span><span></span></span>' : ""}</span>`;
}

export function slider(label, value, number, stepped = false) {
  return `<div class="zr-slider ${stepped ? "is-stepped" : ""}" style="--value:${value}%"><span>${esc(label)}</span><span class="zr-slider-track"><span class="zr-slider-fill"></span><span class="zr-slider-thumb"></span></span>${numberField(number)}</div>`;
}

export function rangeSlider(label, min, max, minNumber, maxNumber) {
  return `<div class="zr-slider is-range" style="--min:${min}%;--value:${max}%;--max:${max}%"><span>${esc(label)}</span><span class="zr-slider-track"><span class="zr-slider-fill"></span><span class="zr-slider-thumb is-min"></span><span class="zr-slider-thumb is-max"></span>${numberField(minNumber, { className: "zr-range-min" })}</span>${numberField(maxNumber)}</div>`;
}
