import { coreModules } from "../core/core-modules.js";
import { componentLabModule } from "../component-lab/module.js";
import { buildExtensionModules } from "../extensions/extension-modules.js";

export const defaultModuleId = "gameplay-effect";

const { editorLibraryModule, extensionModules: builtExtensionModules } = buildExtensionModules(coreModules, defaultModuleId);

export const nativeModules = coreModules;
export const webModuleTabs = [...coreModules, editorLibraryModule, componentLabModule];
export const extensionModules = builtExtensionModules;
export const modules = [...coreModules, editorLibraryModule, componentLabModule, ...extensionModules];

export function moduleById(id) {
  return modules.find((module) => module.id === id) ?? modules.find((module) => module.id === defaultModuleId);
}
