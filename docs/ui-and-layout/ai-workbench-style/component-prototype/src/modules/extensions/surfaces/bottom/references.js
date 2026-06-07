import { actionButton, assetStrip, moduleTable, moduleTree, panel, tag } from "../../../shared/module-components.js";
import { extensionRouteOptions } from "../routes.js";
import { esc } from "../utils.js";

export function extensionReferencesPanel(config) {
  return `<div class="zr-module-output-grid">
    ${moduleTable(["Reference", "Kind", "Route"], referenceRows(config), "1.4fr 0.8fr 1fr", extensionRouteOptions(config, "references", "workbench.extension.references"))}
    ${panel("Reference assets", `${moduleTree(config.assets, extensionRouteOptions(config, "references", "workbench.extension.asset"))}${assetStrip(config.tools.slice(0, 4), extensionRouteOptions(config, "references", "workbench.extension.reference"))}`)}
    <div class="zr-module-log"><p>${esc(config.label)} reference source: ${esc(config.source)}</p><p class="is-success">Cards, rows, and toolbar actions keep the user inside this module route.</p><p>${esc(config.category)} panels reuse the same bottom output, validation, and reference component grammar.</p>${actionButton("More Editors", "grid")}</div>
  </div>`;
}

function referenceRows(config) {
  return [
    { cells: [config.source, "AI sample", tag("Current", "cyan")], selected: true },
    { cells: [config.category, "Category", tag(config.layoutKind, "blue")] },
    { cells: [config.tools[0] ?? config.label, "Primary tool", tag("Output", "green")] },
    { cells: [config.detailTabs[0] ?? "Details", "Right panel", tag("Details", "purple")] }
  ];
}
