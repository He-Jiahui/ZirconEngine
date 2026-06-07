import { searchInput } from "../../../components/inputs/atoms.js";
import {
  compactStats,
  moduleTable,
  moduleTree,
  panelGroup,
  settingsRows,
  tag
} from "../../shared/module-components.js";
import { libraryRouteOptions } from "./routes.js";

export function editorLibraryDetails(extensionModuleConfigs) {
  return panelGroup("library-right", [
    { label: "Catalog", active: true, content: `${searchInput("Filter modules...")}${moduleTree(extensionModuleConfigs.map((config, index) => [config.label, config.icon, index === 0, 0]), libraryRouteOptions("library-right:catalog", "workbench.library.catalog"))}` },
    {
      label: "Coverage",
      content: `${compactStats([["Core", "11"], ["Extended", String(extensionModuleConfigs.length)], ["AI Refs", String(extensionModuleConfigs.length)], ["Shells", "1"]])}${settingsRows([
        ["Top Tabs", tag("Core + More", "cyan")],
        ["Rail", tag("Core + More", "cyan")],
        ["Extended", tag("Library Cards", "green")],
        ["Native", tag("Core 11", "orange")]
      ])}`
    },
    {
      label: "Routing",
      content: moduleTable(["Route", "Target", "Mode"], [
        { cells: ["More Editors", "editor-library", tag("Module", "cyan")], selected: true },
        { cells: ["Extension Card", "Selected editor", tag("Module", "green")] },
        { cells: ["Extension Toolbar", "Output / Validation / References", tag("Panel", "blue")] }
      ], "1fr 1fr 86px", libraryRouteOptions("library-right:routing", "workbench.library.route"))
    }
  ]);
}
