import {
  moduleTree,
  panel,
  settingsRows,
  tag
} from "../../shared/module-components.js";
import { referenceGroupsList } from "./rows.js";
import { libraryRouteOptions } from "./routes.js";

export function editorLibraryLeft(extensionModuleConfigs, coreModules) {
  return [
    panel("Reference Groups", referenceGroupsList(extensionModuleConfigs)),
    panel("Implementation Rule", settingsRows([
      ["Shell", tag("Shared", "cyan")],
      ["Layout", tag("Left / Main / Right / Bottom", "green")],
      ["Style", tag("Workbench Dark", "blue")],
      ["Native Sync", tag("Core 11 only", "orange")]
    ])),
    panel("Core Modules", moduleTree(coreModules.map((module, index) => [module.label, module.icon, index === 1, 0]), libraryRouteOptions("library-right:catalog", "workbench.library.core")))
  ];
}
