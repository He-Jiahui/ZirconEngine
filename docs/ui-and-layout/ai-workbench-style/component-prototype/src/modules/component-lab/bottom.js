import { alerts } from "../../components/data/collections.js";
import { actionButton, moduleTable, settingsRows, tag } from "../shared/module-components.js";
import { componentCoverage, labToolbarPanels } from "./data.js";
import { componentLabRouteOptions } from "./routes.js";

export function componentLabBottom() {
  return `${moduleTable(
    ["Audit", "Scope", "State"],
    [
      { cells: ["Input atom sweep", "buttons / fields / selection", tag("Ready", "green")], selected: true },
      { cells: ["Collection sweep", "list / tree / table / popup", tag("Ready", "green")] },
      { cells: ["Surface sweep", "drawer / window / panel group", tag("Ready", "green")] },
      { cells: ["Responsive sweep", "6 viewports plus live resize", tag("Browser", "cyan")] }
    ],
    "1fr 1.35fr 0.8fr",
    componentLabRouteOptions("module-bottom-component-lab:audit-log", "workbench.component_lab.audit")
  )}${settingsRows([
    ["Toolbar Routes", `${labToolbarPanels.size} scoped commands`],
    ["Families", `${componentCoverage.length} browser component families`],
    ["Native Contract", tag("component-family handoff only", "cyan")]
  ])}${alerts([
    ["info", "Component Lab is a web-only audit module; nativeModules remains the eleven editor modules."],
    ["success", "Every visible control still writes a routed response through the shared workbench controller."]
  ])}${actionButton("Open Component Routes", "target", componentLabRouteOptions("module-bottom-component-lab:routes", "workbench.component_lab.audit"))}`;
}
