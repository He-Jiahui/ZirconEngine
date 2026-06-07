import { actionPath } from "../../../foundation/action-paths.js";
import { icon } from "../../../foundation/icons.js";
import { esc } from "../collection-utils.js";

export function treeRow(node, depth = 0) {
  const hasChildren = node.children?.length;
  const depthClass = depth === 1 ? "is-child" : depth > 1 ? "is-grandchild" : "";
  const openIcon = hasChildren ? (node.collapsed ? "chevronRight" : "chevronDown") : "";
  const more = node.selected ? icon("more") : "";
  const lock = node.locked ? icon("lock") : "";
  return [
    `<button class="zr-tree-row ${depthClass} ${node.selected ? "is-selected" : ""}" type="button" data-action="${actionPath("workbench.collection.tree", node.label)}" data-tree-row="${node.id}" aria-label="${esc(node.label)}">`,
    `<span>${openIcon ? icon(openIcon) : ""}</span>${icon(node.icon)}<span class="zr-tree-label">${esc(node.label)}</span>`,
    `<span class="zr-tree-action">${icon("eye")}</span><span class="zr-tree-action">${lock || more}</span></button>`,
    ...(hasChildren && !node.collapsed ? node.children.map((child) => treeRow(child, depth + 1)) : [])
  ].join("");
}
