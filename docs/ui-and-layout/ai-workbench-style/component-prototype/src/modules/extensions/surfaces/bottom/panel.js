import { panelGroup } from "../../../shared/module-components.js";
import { esc } from "../utils.js";
import { extensionBottomHandoffPanel } from "./handoff.js";
import { extensionOutputPanel } from "./output.js";
import { extensionReferencesPanel } from "./references.js";
import { extensionValidationPanel } from "./validation.js";

export function extensionBottomOutput(config) {
  return `<section class="zr-panel zr-module-bottom" data-surface="drawer" data-module-panel="bottom" data-panel-host="module-bottom-${esc(config.id)}">
    ${panelGroup(`module-bottom-${config.id}`, [
      { label: "Output", content: extensionOutputPanel(config), active: true },
      { label: "Validation", content: extensionValidationPanel(config) },
      { label: "References", content: extensionReferencesPanel(config) },
      { label: "Handoff", content: extensionBottomHandoffPanel(config) }
    ], { className: "is-module-bottom" })}
  </section>`;
}
