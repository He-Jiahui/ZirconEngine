import { listRow } from "./list-view/row.js";

export function listView(items) {
  return `<div class="zr-list">${items.map((item) => listRow(item)).join("")}</div>`;
}
