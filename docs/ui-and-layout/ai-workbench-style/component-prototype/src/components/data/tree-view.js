import { treeRow } from "./tree-view/row.js";

export function treeView(nodes) {
  return `<div class="zr-tree">${nodes.map((node) => treeRow(node)).join("")}</div>`;
}
