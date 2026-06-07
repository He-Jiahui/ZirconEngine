import { grid } from "../../../foundation/layout.js";
import { listItems, menuItems, tableRows } from "../../../foundation/data.js";
import { listView, menu, tableView } from "../../../components/data/collections.js";
import { moduleTable, panel, tag } from "../../shared/module-components.js";
import { componentLabRouteOptions } from "../routes.js";

export function collectionPalette() {
  return grid({ className: "zr-lower-demo", gap: "md", children: [
    panel("List", listView(listItems)),
    panel("Menu", menu(menuItems)),
    panel("Table", tableView(tableRows)),
    panel("Module Rows", moduleTable(
      ["Component", "Route", "State"],
      [
        { cells: ["List row", "workbench.component_lab.collection.list", tag("Routed", "green")], selected: true },
        { cells: ["Tree row", "workbench.component_lab.collection.tree", tag("Routed", "green")] },
        { cells: ["Table row", "workbench.component_lab.collection.table", tag("Routed", "green")] }
      ],
      "1fr 1.4fr 0.8fr",
      componentLabRouteOptions("component-lab-main:collections", "workbench.component_lab.collection")
    ))
  ] });
}
