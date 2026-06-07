import { icon } from "../../foundation/icons.js";
import { esc } from "./input-utils.js";

export function tabs(items, active = 0, className = "zr-tabs") {
  return `<div class="${className}" role="tablist">${items.map((item, index) => {
    const content = typeof item === "object" ? `${item.icon ? icon(item.icon) : ""}${item.label ? esc(item.label) : ""}` : esc(item);
    return `<button class="${className === "zr-segment" ? "zr-segment-item" : "zr-tab"} ${index === active ? "is-active" : ""}" type="button" role="tab" aria-selected="${index === active ? "true" : "false"}">${content}</button>`;
  }).join("")}</div>`;
}
