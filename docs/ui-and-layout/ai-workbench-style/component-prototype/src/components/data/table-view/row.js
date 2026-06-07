import { actionPath } from "../../../foundation/action-paths.js";
import { icon } from "../../../foundation/icons.js";
import { esc } from "../collection-utils.js";

export function tableRow(row, index) {
  return `<button class="zr-table-row ${index === 1 ? "is-selected" : ""}" type="button" data-action="${actionPath("workbench.collection.table", row[0])}" aria-label="${esc(row[0])}">${row.map((cell) => `<span>${esc(cell)}</span>`).join("")}${icon("more")}</button>`;
}
