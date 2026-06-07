import { checkbox, iconButton } from "../inputs/atoms.js";
export { listView } from "./list-view.js";
export { tableView } from "./table-view.js";
export { treeView } from "./tree-view.js";
export { alerts } from "../feedback/alerts.js";
export { toast } from "../feedback/toast.js";
export { tooltip } from "../feedback/tooltip.js";
export { menu } from "../overlays/menu.js";

export function checkLabel(label, checked) {
  return checkbox(label, checked);
}

export function miniActions() {
  return `${iconButton("filter", "Filter")}${iconButton("plus", "Add")}`;
}
