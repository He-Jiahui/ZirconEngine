import { grid } from "../../../foundation/layout.js";
import { alerts as alertData } from "../../../foundation/data.js";
import { alerts, toast, tooltip } from "../../../components/data/collections.js";
import {
  actionButton,
  moduleTable,
  panel,
  settingsRows,
  tag
} from "../../shared/module-components.js";
import { layoutCoverage } from "../data.js";
import { componentLabRouteOptions } from "../routes.js";

export function surfacePalette() {
  return `${grid({ className: "zr-lower-demo", gap: "md", children: [
    panel("Feedback", `${alerts(alertData)}${tooltip()}${toast()}`),
    panel("Surface Roles", moduleTable(
      ["Surface", "Use", "Route"],
      layoutCoverage.map(([surface, use, route], index) => ({
        cells: [surface, use, tag(route, index < 3 ? "cyan" : "blue")],
        selected: index === 0
      })),
      "0.9fr 1fr 1fr",
      componentLabRouteOptions("component-lab-right:surfaces", "workbench.component_lab.surface")
    )),
    panel("Responsive Contract", settingsRows([
      ["Desktop", tag("left / center / right", "green")],
      ["Compact", tag("right overlay", "cyan")],
      ["Narrow", tag("single column stack", "blue")],
      ["Audit", actionButton("Run Browser Sweep", "check", componentLabRouteOptions("module-bottom-component-lab:responsive", "workbench.component_lab.responsive"))]
    ]))
  ] })}`;
}
