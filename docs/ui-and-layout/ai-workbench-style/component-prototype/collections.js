import { icon } from "./icons.js";
import { checkbox, iconButton } from "./atoms.js";

function treeRow(node, depth = 0) {
  const hasChildren = node.children?.length;
  const depthClass = depth === 1 ? "is-child" : depth > 1 ? "is-grandchild" : "";
  const openIcon = hasChildren ? (node.collapsed ? "chevronRight" : "chevronDown") : "";
  const more = node.selected ? icon("more") : "";
  const lock = node.locked ? icon("lock") : "";
  return [
    `<div class="zr-tree-row ${depthClass} ${node.selected ? "is-selected" : ""}" data-tree-row="${node.id}">`,
    `<span>${openIcon ? icon(openIcon) : ""}</span>${icon(node.icon)}<span class="zr-tree-label">${node.label}</span>`,
    `<span class="zr-tree-action">${icon("eye")}</span><span class="zr-tree-action">${lock || more}</span></div>`,
    ...(hasChildren && !node.collapsed ? node.children.map((child) => treeRow(child, depth + 1)) : [])
  ].join("");
}

export function treeView(nodes) {
  return `<div class="zr-tree">${nodes.map((node) => treeRow(node)).join("")}</div>`;
}

export function tableView(rows) {
  return `<div class="zr-table"><div class="zr-table-row zr-table-head"><span>Name</span><span>Type</span><span>Size</span><span>Modified</span>${icon("gear")}</div>${rows.map((row, index) => `<div class="zr-table-row ${index === 1 ? "is-selected" : ""}">${row.map((cell) => `<span>${cell}</span>`).join("")}${icon("more")}</div>`).join("")}</div>`;
}

export function listView(items) {
  return `<div class="zr-list">${items.map((item) => `<div class="zr-list-item ${item.selected ? "is-selected" : ""} ${item.disabled ? "is-disabled" : ""}"><span class="zr-list-handle"></span><span>${item.label}</span>${item.selected ? icon("check") : icon("cube")}</div>`).join("")}</div>`;
}

export function menu(items) {
  return `<div class="zr-menu">${items.map(([label, glyph, tone]) => `<div class="zr-menu-row ${tone === "danger" ? "is-danger" : ""}"><span>${label}</span>${icon(glyph)}</div>`).join("")}</div>`;
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
