import { icon } from "../../foundation/icons.js";
import { actionPath } from "../../foundation/action-paths.js";
import { esc, routeAttrs, tabKey } from "./utils.js";

function tableRowActionId(row, index) {
  const readableCell = row.cells.find((cell) => {
    const text = String(cell ?? "").trim();
    return text && !/^\d+(?:\.\d+)?$/.test(text) && !/[<>]/.test(text);
  });
  return readableCell ?? `row ${index + 1}`;
}

function tableRowLabel(row, index) {
  const readableCell = row.cells.find((cell) => {
    const text = String(cell ?? "").trim();
    return text && !/^\d+(?:\.\d+)?$/.test(text) && !/[<>]/.test(text);
  });
  return String(readableCell ?? `Row ${index + 1}`);
}

export function settingsRows(rows) {
  return `<div class="zr-module-settings">${rows.map(([label, control]) => `<div class="zr-module-setting"><span>${esc(label)}</span><span>${control}</span></div>`).join("")}</div>`;
}

export function listRows(items, selected = 0, values = [], options = {}) {
  return `<div class="zr-list zr-module-list">${items.map((item, index) => `<button class="zr-list-item zr-module-list-row ${index === selected ? "is-selected" : ""}" type="button" data-action="${actionPath(options.actionScope ?? "workbench.module.list", item)}"${routeAttrs(options)} aria-label="${esc(item)}"><span>${esc(item)}</span><small>${esc(values[index] ?? "")}</small></button>`).join("")}</div>`;
}

export function moduleTree(rows, options = {}) {
  return `<div class="zr-tree zr-module-tree">${rows.map(([label, glyph, selected, depth]) => {
    const actionAttr = options.actionScope ? ` data-action="${actionPath(options.actionScope, label)}"` : "";
    return `<button class="zr-tree-row zr-module-tree-row ${selected ? "is-selected" : ""}" type="button"${actionAttr}${routeAttrs(options)} data-tree-row="${tabKey(label)}" data-depth="${depth}"><span>${depth > 0 ? icon("chevronRight") : icon("chevronDown")}</span>${icon(glyph)}<span>${esc(label)}</span><small>${selected ? icon("check") : ""}</small></button>`;
  }).join("")}</div>`;
}

export function segmentButtons(items, active = 0) {
  return `<div class="zr-segment zr-module-segment" role="tablist">${items.map((item, index) => `<button class="zr-segment-item ${index === active ? "is-active" : ""}" type="button" role="tab" aria-selected="${index === active ? "true" : "false"}">${esc(item)}</button>`).join("")}</div>`;
}

export function moduleTable(headers, rows, columns, options = {}) {
  return `<div class="zr-table zr-module-table" style="--module-table-cols:${esc(columns)}">
    <div class="zr-module-table-row is-head">${headers.map((header) => `<span>${esc(header)}</span>`).join("")}</div>
    ${rows.map((row, index) => `<div class="zr-module-table-row ${row.selected ? "is-selected" : ""}" role="button" tabindex="0" aria-label="${esc(tableRowLabel(row, index))}" data-action="${actionPath(options.actionScope ?? "workbench.module.table", tableRowActionId(row, index))}"${routeAttrs(options)}>${row.cells.map((cell) => `<span>${cell}</span>`).join("")}</div>`).join("")}
  </div>`;
}
