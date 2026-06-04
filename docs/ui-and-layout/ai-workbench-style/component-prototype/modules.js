import { icon } from "./icons.js";
import { coreModules } from "./core-modules.js";
import { buildExtensionModules } from "./extension-modules.js";
import {
  actionButton,
  actionIcon,
  moduleLeft,
  moduleMain,
  moduleRight
} from "./module-components.js";

const esc = (value) => String(value ?? "").replace(/[&<>"']/g, (char) => ({
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#039;"
}[char]));

export const defaultModuleId = "gameplay-effect";

const { editorLibraryModule, extensionModules } = buildExtensionModules(coreModules, defaultModuleId);

export const nativeModules = coreModules;
export const webModuleTabs = [...coreModules, editorLibraryModule];
export const modules = [...coreModules, editorLibraryModule, ...extensionModules];
export { extensionModules };

export function moduleById(id) {
  return modules.find((module) => module.id === id) ?? modules.find((module) => module.id === defaultModuleId);
}

export function moduleTabs(activeId = defaultModuleId) {
  return `<nav class="zr-module-tabs" aria-label="Editor modules">${webModuleTabs.map((module) => {
    const active = module.id === activeId || (module.id === "editor-library" && moduleById(activeId).extension);
    return `<button class="zr-module-tab ${active ? "is-active" : ""}" type="button" data-module="${esc(module.id)}" aria-selected="${active ? "true" : "false"}">${icon(module.icon)}<span>${esc(module.shortLabel ?? module.label)}</span></button>`;
  }).join("")}</nav>`;
}

export function moduleToolbar(activeId = defaultModuleId) {
  const module = moduleById(activeId);
  return `<div class="zr-module-toolbar" data-action-group="module-toolbar">${module.actions.map(([glyph, label], index) => (
    actionButton(label, glyph, { active: index === 2 && label === "Compile" })
  )).join("")}</div>`;
}

export function moduleRail(activeId = defaultModuleId) {
  return `<nav class="zr-rail">${webModuleTabs.map((module) => {
    const active = module.id === activeId || (module.id === "editor-library" && moduleById(activeId).extension);
    return `<button class="zr-icon-button zr-rail-module ${active ? "is-active" : ""}" type="button" title="${esc(module.label)}" aria-label="${esc(module.label)}" data-module="${esc(module.id)}">${icon(module.icon)}</button>`;
  }).join("")}<span class="zr-rail-spacer"></span>${actionIcon("Settings", "gear")}${actionIcon("Help", "help")}</nav>`;
}

export function moduleWorkspace(activeId = defaultModuleId) {
  const module = moduleById(activeId);
  return `${moduleLeft(module)}${moduleMain(module)}${moduleRight(module)}${module.bottom()}`;
}
