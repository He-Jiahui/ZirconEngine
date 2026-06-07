import { extensionPrimaryPanel } from "../primary.js";
import { esc } from "../utils.js";
import { extensionMetricsPanel } from "./metrics.js";
import { extensionReferenceRhythmPanel } from "./reference-rhythm.js";

export function extensionCenter(config) {
  return `<div class="zr-module-editor-grid is-extension is-extension-${esc(config.layoutKind)}" data-extension-blueprint="${config.blueprint ? "reference" : "recipe"}">
    ${extensionPrimaryPanel(config)}
    ${extensionMetricsPanel(config)}
    ${extensionReferenceRhythmPanel(config)}
  </div>`;
}
