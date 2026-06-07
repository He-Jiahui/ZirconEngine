import { alerts } from "../../../../components/data/collections.js";
import { moduleTable, settingsRows, tag } from "../../../shared/module-components.js";
import { extensionRouteOptions } from "../routes.js";

export function extensionValidationPanel(config) {
  return `<div class="zr-module-output-grid">
    ${moduleTable(["Check", "Scope", "State"], validationRows(config), "1.2fr 1fr 0.8fr", extensionRouteOptions(config, "validation", "workbench.extension.validation"))}
    ${settingsRows([
      ["Blueprint", tag(config.blueprint ? "Reference Specific" : "Category Recipe", config.blueprint ? "green" : "blue")],
      ["Category", tag(config.category, "cyan")],
      ["Native Surface", tag("Pending", "orange")],
      ["Route State", tag("Panel Hash Active", "green")]
    ])}
    ${alerts([["success", `${config.label} uses the shared module bottom drawer`], ["warning", "Native ZUI surface is not generated for this extension"], ["info", "Validation rows are prototype route targets"]])}
  </div>`;
}

function validationRows(config) {
  return [
    { cells: ["Component Stack", config.layoutKind, tag("Ready", "green")], selected: true },
    { cells: ["Toolbar Routes", `${config.actions.length + 2} actions`, tag("Ready", "green")] },
    { cells: ["Reference Blueprint", config.source, tag(config.blueprint ? "Specific" : "Recipe", config.blueprint ? "cyan" : "blue")] },
    { cells: ["Native Handoff", "ZUI retained host", tag("Pending", "orange")] }
  ];
}
