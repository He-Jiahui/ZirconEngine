import { handledClick, ignoredClick } from "../utils.js";
import { applyTreeRowFallback } from "./feedback.js";
import { selectExclusiveRows } from "./selection.js";

export function handleTreeRowClick(event, controller) {
  const treeRow = event.target.closest("[data-tree-row]");
  if (!treeRow) return ignoredClick;
  selectExclusiveRows(treeRow, document.querySelectorAll(".zr-tree-row, .zr-module-tree-row"));
  if (!controller.applyCommandRoute(treeRow)) {
    applyTreeRowFallback(treeRow, controller);
  }
  return handledClick;
}
