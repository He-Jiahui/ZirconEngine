import { icon } from "./icons.js";
import { checkbox, iconButton } from "./atoms.js";

const esc = (value) => String(value ?? "").replace(/[&<>"']/g, (char) => ({
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#039;"
}[char]));

function actionKey(value) {
  return String(value ?? "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

function treeRow(node, depth = 0) {
  const hasChildren = node.children?.length;
  const depthClass = depth === 1 ? "is-child" : depth > 1 ? "is-grandchild" : "";
  const openIcon = hasChildren ? (node.collapsed ? "chevronRight" : "chevronDown") : "";
  const more = node.selected ? icon("more") : "";
  const lock = node.locked ? icon("lock") : "";
  return [
    `<button class="zr-tree-row ${depthClass} ${node.selected ? "is-selected" : ""}" type="button" data-action="${actionKey(node.label)}" data-tree-row="${node.id}" aria-label="${esc(node.label)}">`,
    `<span>${openIcon ? icon(openIcon) : ""}</span>${icon(node.icon)}<span class="zr-tree-label">${node.label}</span>`,
    `<span class="zr-tree-action">${icon("eye")}</span><span class="zr-tree-action">${lock || more}</span></button>`,
    ...(hasChildren && !node.collapsed ? node.children.map((child) => treeRow(child, depth + 1)) : [])
  ].join("");
}

export function treeView(nodes) {
  return `<div class="zr-tree">${nodes.map((node) => treeRow(node)).join("")}</div>`;
}

export function tableView(rows) {
  return `<div class="zr-table"><div class="zr-table-row zr-table-head"><span>Name</span><span>Type</span><span>Size</span><span>Modified</span>${icon("gear")}</div>${rows.map((row, index) => `<button class="zr-table-row ${index === 1 ? "is-selected" : ""}" type="button" data-action="${actionKey(row[0])}" aria-label="${esc(row[0])}">${row.map((cell) => `<span>${esc(cell)}</span>`).join("")}${icon("more")}</button>`).join("")}</div>`;
}

export function listView(items) {
  return `<div class="zr-list">${items.map((item) => `<button class="zr-list-item ${item.selected ? "is-selected" : ""} ${item.disabled ? "is-disabled" : ""}" type="button" data-action="${actionKey(item.label)}" aria-label="${esc(item.label)}" ${item.disabled ? "disabled" : ""}><span class="zr-list-handle"></span><span>${esc(item.label)}</span>${item.selected ? icon("check") : icon("cube")}</button>`).join("")}</div>`;
}

export function menu(items) {
  return `<div class="zr-menu">${items.map(([label, glyph, tone]) => `<button class="zr-menu-row ${tone === "danger" ? "is-danger" : ""}" type="button" data-menu-item="${actionKey(label)}" data-action="menu-${actionKey(label)}" aria-label="${esc(label)}"><span>${esc(label)}</span>${icon(glyph)}</button>`).join("")}</div>`;
}

export function alerts(items) {
  return `<div class="zr-alert-stack">${items.map(([tone, label]) => `<div class="zr-alert is-${tone}"><span class="zr-alert-status">${alertMark(tone)}</span><span>${label}</span>${icon("x")}</div>`).join("")}</div>`;
}

function alertMark(tone) {
  if (tone === "success") return icon("check");
  if (tone === "warning") return `<span>!</span>`;
  if (tone === "error") return `<span>x</span>`;
  return `<span>i</span>`;
}

export function tooltip() {
  return `<div class="zr-tooltip"><div class="zr-tooltip-bubble"><div>Tooltip</div><div class="zr-tooltip-small">This is a tooltip</div></div>${icon("info")}</div>`;
}

export function toast() {
  return `<div class="zr-toast"><span class="zr-toast-status">${icon("check")}</span><span>Operation completed successfully</span><strong>UNDO</strong>${icon("x")}</div>`;
}

export function checkLabel(label, checked) {
  return checkbox(label, checked);
}

export function miniActions() {
  return `${iconButton("filter", "Filter")}${iconButton("plus", "Add")}`;
}
