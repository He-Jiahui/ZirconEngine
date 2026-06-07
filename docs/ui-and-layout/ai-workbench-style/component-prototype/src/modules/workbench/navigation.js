import { icon } from "../../foundation/icons.js";
import { esc } from "../shared/utils.js";
import { defaultModuleId, moduleById, webModuleTabs } from "./registry.js";

export function moduleTabs(activeId = defaultModuleId) {
  return `<nav class="zr-module-tabs" aria-label="Editor modules">${webModuleTabs.map((module) => {
    const active = module.id === activeId || (module.id === "editor-library" && moduleById(activeId).extension);
    return `<button class="zr-module-tab ${active ? "is-active" : ""}" type="button" data-module="${esc(module.id)}" aria-selected="${active ? "true" : "false"}">${icon(module.icon)}<span>${esc(module.shortLabel ?? module.label)}</span></button>`;
  }).join("")}</nav>`;
}
