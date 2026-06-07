import {
  assetStrip,
  graphBoard,
  graphLink,
  node,
  panel
} from "../../shared/module-components.js";
import { labAssets } from "../data.js";
import { componentLabRouteOptions } from "../routes.js";

export function layoutGrammarPanel() {
  return panel("Layout Grammar", `${graphBoard("component-lab", [
    node("Stack", "vertical", 16, 26, "cyan", componentLabRouteOptions("component-lab-right:layout", "workbench.component_lab.layout")),
    node("Cluster", "inline", 42, 18, "green", componentLabRouteOptions("component-lab-right:layout", "workbench.component_lab.layout")),
    node("Grid", "matrix", 67, 30, "blue", componentLabRouteOptions("component-lab-right:layout", "workbench.component_lab.layout")),
    node("Panel Group", "tabs", 36, 66, "orange", componentLabRouteOptions("component-lab-main:surfaces", "workbench.component_lab.layout")),
    node("Drawer", "region", 72, 70, "purple", componentLabRouteOptions("component-lab-right:surfaces", "workbench.component_lab.layout"))
  ], `${graphLink(26, 31, 18, -10)}${graphLink(51, 27, 15, 15)}${graphLink(45, 56, 18, 65)}${graphLink(58, 70, 12, 0)}`)}${assetStrip(labAssets, componentLabRouteOptions("component-lab-right:surfaces", "workbench.component_lab.asset"))}`);
}
