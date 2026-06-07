import { assetStrip, panel, settingsRows, tag } from "../../../shared/module-components.js";
import { extensionRouteOptions } from "../routes.js";

export function extensionReferenceRhythmPanel(config) {
  return panel("Reference Rhythm", `${assetStrip(config.tools.slice(0, 6), extensionRouteOptions(config, "references", "workbench.extension.reference"))}${settingsRows([["Source", config.source], ["Category", config.category], ["Blueprint", tag(config.blueprint ? "Reference Specific" : "Category Recipe", config.blueprint ? "green" : "blue")], ["Response", tag("Click any control", "green")]])}`);
}
