import { handledClick, ignoredClick } from "../utils.js";
import { applyDataRowFallback } from "./feedback.js";
import { selectExclusiveRows } from "./selection.js";

const dataRowSelector = ".zr-list-item:not(.is-disabled), .zr-table-row:not(.zr-table-head), .zr-module-list-row, .zr-module-table-row:not(.is-head)";

export function handleDataRowClick(event, controller) {
  const row = event.target.closest(dataRowSelector);
  if (!row) return ignoredClick;
  selectExclusiveRows(row, row.parentElement.querySelectorAll(".is-selected"));
  if (!controller.applyCommandRoute(row)) {
    applyDataRowFallback(row, controller);
  }
  return handledClick;
}
