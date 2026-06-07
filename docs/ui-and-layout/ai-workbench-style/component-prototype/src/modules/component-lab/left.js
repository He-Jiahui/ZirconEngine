import { actionPath } from "../../foundation/action-paths.js";
import { searchInput } from "../../components/inputs/atoms.js";
import { panel, tag } from "../shared/module-components.js";
import { esc } from "../shared/utils.js";
import { componentNav } from "./data.js";

export function componentLabLeft() {
  return [
    panel("Component Families", `${searchInput("Filter components...")}${componentNavRows()}`),
    panel("Assembly Rules", `<div class="zr-module-settings">
      <div class="zr-module-setting"><span>Taxonomy</span><span>${tag("Functional Path", "green")}</span></div>
      <div class="zr-module-setting"><span>Actions</span><span>${tag("Dotted IDs", "cyan")}</span></div>
      <div class="zr-module-setting"><span>Layout</span><span>${tag("Flex / Grid", "blue")}</span></div>
    </div>`)
  ];
}

function componentNavRows() {
  return `<div class="zr-list zr-module-list">${componentNav.map(([label, summary, panelTarget], index) => `
    <button class="zr-list-item zr-module-list-row ${index === 0 ? "is-selected" : ""}" type="button"
      data-action="${actionPath("workbench.component_lab.left", label)}"
      data-route-panel="${esc(panelTarget)}"
      aria-label="${esc(label)}">
      <span>${esc(label)}</span><small>${esc(summary)}</small>
    </button>`).join("")}</div>`;
}
