import { bottomOutput } from "../../shared/module-components.js";
import { editorLibraryBottom } from "./bottom.js";
import { editorLibraryCenter } from "./center.js";
import { editorLibraryDetails } from "./details.js";
import { editorLibraryLeft } from "./left.js";

export function createEditorLibraryModule(extensionModuleConfigs, coreModules, defaultModuleId) {
  return {
    id: "editor-library",
    label: "More Editors",
    shortLabel: "More",
    icon: "grid",
    status: "Extended editor module library ready",
    actions: [
      ["search", "Find Editor"],
      ["folder", "Browse References"],
      ["grid", "Core Modules"],
      ["check", "Validate Coverage"]
    ],
    left: () => editorLibraryLeft(extensionModuleConfigs, coreModules),
    center: () => editorLibraryCenter(extensionModuleConfigs, coreModules, defaultModuleId),
    right: () => editorLibraryDetails(extensionModuleConfigs),
    bottom: () => bottomOutput("editor-library", ["Coverage", "Reference Notes", "Routing Log"], editorLibraryBottom(extensionModuleConfigs))
  };
}
