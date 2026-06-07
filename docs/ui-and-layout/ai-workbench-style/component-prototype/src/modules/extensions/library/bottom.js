import { alerts } from "../../../components/data/collections.js";
import { moduleTable } from "../../shared/module-components.js";
import { extensionCoverageRows } from "./rows.js";
import { libraryRouteOptions } from "./routes.js";

export function editorLibraryBottom(extensionModuleConfigs) {
  return `<div class="zr-module-output-grid">
    ${moduleTable(["Sample", "Prototype Module", "Coverage"], extensionCoverageRows(extensionModuleConfigs), "1.4fr 1fr 98px", libraryRouteOptions("module-bottom-editor-library:coverage", "workbench.library.coverage"))}
    ${alerts([["success", `${extensionModuleConfigs.length} extended editor cards use the same response path`], ["info", "Native handoff remains scoped to the core 11 modules"], ["warning", "Extended modules are prototype-only until native ZUI surfaces are added"]])}
  </div>`;
}
