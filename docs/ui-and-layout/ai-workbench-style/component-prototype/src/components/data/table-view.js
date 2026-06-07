import { tableHeader } from "./table-view/header.js";
import { tableRow } from "./table-view/row.js";

export function tableView(rows) {
  return `<div class="zr-table">${tableHeader()}${rows.map((row, index) => tableRow(row, index)).join("")}</div>`;
}
