import { alerts } from "../../../components/data/collections.js";
import { input, select, toggle } from "../../../components/inputs/atoms.js";
import { icon } from "../../../foundation/icons.js";
import { actionButton, moduleTable, settingsRows } from "../../shared/module-components.js";
import { coreBottomRouteOptions } from "./routes.js";

export function hudBottom() {
  const routeOptions = coreBottomRouteOptions("hud-editor", "validation");
  return `<div class="zr-module-output-grid">
    ${alerts([["warning", "Text MatchTimer is not localized"], ["error", "Binding AmmoCount could not be resolved"], ["info", "DPI scale set to 1.00"]])}
    ${moduleTable(["Type", "Severity", "Message", "Widget", "Line"], [
      { cells: [icon("warning"), "Warning", "Text 'MatchTimer' is not localized", "MatchTimer", "--"] },
      { cells: [icon("x"), "Error", "Binding 'AmmoCount' could not be resolved", "Ammo_Reserve", "57"], selected: true },
      { cells: [icon("info"), "Info", "Image 'Minimap' has no alt text", "Minimap", "--"] }
    ], "46px 82px 2fr 1fr 64px", routeOptions)}
    ${settingsRows([["Filter", select("All")], ["Clear", actionButton("Clear All", "trash", routeOptions)], ["Auto Preview", toggle("", true)]])}
  </div>`;
}
