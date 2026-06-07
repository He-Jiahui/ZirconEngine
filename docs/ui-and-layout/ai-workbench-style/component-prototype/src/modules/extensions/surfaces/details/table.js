import { moduleTable } from "../../../shared/module-components.js";
import { extensionRouteOptions } from "../routes.js";
import { toRows } from "../utils.js";

export function extensionDetailTablePanel(config, label) {
  return moduleTable(config.tableHeaders, toRows(config.table, 1), config.tableColumns, extensionRouteOptions(config, `right:${label}`, "workbench.extension.detail"));
}
