import { actionPath } from "../../../foundation/action-paths.js";
import { icon } from "../../../foundation/icons.js";
import { esc } from "../collection-utils.js";

export function listRow(item) {
  return `<button class="zr-list-item ${item.selected ? "is-selected" : ""} ${item.disabled ? "is-disabled" : ""}" type="button" data-action="${actionPath("workbench.collection.list", item.label)}" aria-label="${esc(item.label)}" ${item.disabled ? "disabled" : ""}><span class="zr-list-handle"></span><span>${esc(item.label)}</span>${item.selected ? icon("check") : icon("cube")}</button>`;
}
