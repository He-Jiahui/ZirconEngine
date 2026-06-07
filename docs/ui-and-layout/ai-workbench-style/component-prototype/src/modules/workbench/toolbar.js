import { actionButton } from "../shared/module-components.js";
import { defaultModuleId, moduleById } from "./registry.js";

export function moduleToolbar(activeId = defaultModuleId) {
  const module = moduleById(activeId);
  return `<div class="zr-module-toolbar" data-action-group="module-toolbar">${module.actions.map(([glyph, label], index) => (
    actionButton(label, glyph, { active: index === 2 && label === "Compile", actionScope: "workbench.module.toolbar" })
  )).join("")}</div>`;
}
