import { icon } from "../../../foundation/icons.js";
import { esc } from "../../shared/utils.js";

export function extensionModuleCard(config) {
  return `<button class="zr-extension-card is-${esc(config.layoutKind)}" type="button" data-module="${esc(config.id)}" data-module-source="extension-library" aria-label="${esc(config.label)}">
    <span class="zr-extension-card-icon">${icon(config.icon)}</span>
    <span class="zr-extension-card-copy">
      <strong>${esc(config.label)}</strong>
      <small>${esc(config.category)} / ${esc(config.source)}</small>
    </span>
    <span class="zr-extension-card-status">${config.metrics.map(([label, value]) => `${esc(label)} ${esc(value)}`).slice(0, 2).join(" | ")}</span>
  </button>`;
}
