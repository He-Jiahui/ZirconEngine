import {
  moduleTable,
  panel,
  tag
} from "../../shared/module-components.js";
import { extensionModuleCard } from "./cards.js";
import { referenceBlueprintDrilldown } from "./drilldown.js";
import { libraryRouteOptions } from "./routes.js";

export function editorLibraryCenter(extensionModuleConfigs, coreModules, defaultModuleId) {
  return `<div class="zr-module-editor-grid is-library">
    ${panel("Extended Editor Modules", `<div class="zr-extension-card-grid">${extensionModuleConfigs.map(extensionModuleCard).join("")}</div>`)}
    ${panel("Reference Blueprint Drilldown", referenceBlueprintDrilldown(extensionModuleConfigs))}
    ${panel("Core Native-Synced Modules", moduleTable(["Module", "Reference", "Native"], coreModules.map((module) => ({
      cells: [module.label, module.shortLabel ?? module.label, tag("Synced", "green")],
      selected: module.id === defaultModuleId
    })), "1.2fr 1fr 82px", libraryRouteOptions("library-right:coverage", "workbench.library.core")))}
  </div>`;
}
