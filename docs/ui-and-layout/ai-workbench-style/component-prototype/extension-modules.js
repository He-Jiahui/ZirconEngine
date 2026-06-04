import { extensionModuleConfigs } from "./extension-configs.js";
import { createEditorLibraryModule } from "./extension-library.js";
import {
  extensionBottomOutput,
  extensionCenter,
  extensionDetails,
  extensionLeft
} from "./extension-surfaces.js";

export function buildExtensionModules(coreModules, defaultModuleId) {
  const extensionModules = extensionModuleConfigs.map(createExtensionModule);
  return {
    editorLibraryModule: createEditorLibraryModule(extensionModuleConfigs, coreModules, defaultModuleId),
    extensionModules
  };
}

function createExtensionModule(config) {
  return {
    id: config.id,
    label: config.label,
    shortLabel: config.shortLabel,
    icon: config.icon,
    extension: true,
    blueprint: config.blueprint,
    source: config.source,
    category: config.category,
    layoutKind: config.layoutKind,
    status: config.status,
    actions: [["grid", "More Editors"], ["check", "Native Handoff"], ...config.actions],
    left: () => extensionLeft(config),
    center: () => extensionCenter(config),
    right: () => extensionDetails(config),
    bottom: () => extensionBottomOutput(config)
  };
}
