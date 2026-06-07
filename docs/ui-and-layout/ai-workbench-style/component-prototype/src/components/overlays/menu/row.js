import { actionPath } from "../../../foundation/action-paths.js";
import { icon } from "../../../foundation/icons.js";
import { actionKey, esc } from "../../data/collection-utils.js";

export function menuRow([label, glyph, tone]) {
  return `<button class="zr-menu-row ${tone === "danger" ? "is-danger" : ""}" type="button" data-menu-item="${actionKey(label)}" data-action="${actionPath("workbench.collection.menu", label)}" aria-label="${esc(label)}"><span>${esc(label)}</span>${icon(glyph)}</button>`;
}
