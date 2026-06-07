import { menuRow } from "./menu/row.js";

export function menu(items) {
  return `<div class="zr-menu">${items.map((item) => menuRow(item)).join("")}</div>`;
}
