import { moduleTable } from "../../../shared/module-components.js";
import { extensionRouteOptions } from "../routes.js";
import { esc, toRows } from "../utils.js";

export function extensionOutputPanel(config) {
  return `<div class="zr-module-output-grid">
    ${moduleTable(config.tableHeaders, toRows(config.table), config.tableColumns, extensionRouteOptions(config, "output", "workbench.extension.output"))}
    <div class="zr-module-log"><p>${esc(config.label)}: opened from ${esc(config.source)}</p><p class="is-success">Prototype route active and response feedback enabled.</p><p class="is-warning">Native ZUI surface not generated for this extended editor.</p></div>
  </div>`;
}
