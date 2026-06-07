import { icon } from "../../foundation/icons.js";
import { actionIcon } from "../shared/module-components.js";
import { esc } from "../shared/utils.js";
import { defaultModuleId, moduleById, webModuleTabs } from "./registry.js";

export function moduleRail(activeId = defaultModuleId) {
  return `<nav class="zr-rail">${webModuleTabs.map((module) => {
    const active = module.id === activeId || (module.id === "editor-library" && moduleById(activeId).extension);
    return `<button class="zr-icon-button zr-rail-module ${active ? "is-active" : ""}" type="button" title="${esc(module.label)}" aria-label="${esc(module.label)}" data-module="${esc(module.id)}">${icon(module.icon)}</button>`;
  }).join("")}<span class="zr-rail-spacer"></span>${actionIcon("Settings", "gear")}${actionIcon("Help", "help")}</nav>`;
}
