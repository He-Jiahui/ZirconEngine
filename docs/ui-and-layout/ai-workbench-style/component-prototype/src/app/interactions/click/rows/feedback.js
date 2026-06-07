import { actionPath } from "../../../../foundation/action-paths.js";
import { normalizeCommand } from "../../../../routing/routes.js";
import { commandLabel } from "../../../labels.js";

export function applyTreeRowFallback(treeRow, controller) {
  const command = normalizeCommand(commandLabel(treeRow));
  controller.recordCommand(treeRow.dataset.action || actionPath("workbench.tree.select", command));
  controller.setStatus(`Selected ${command.replace(/-/g, " ")}`);
}

export function applyDataRowFallback(row, controller) {
  controller.recordCommand(row.dataset.action || actionPath("workbench.row.select", commandLabel(row)));
  controller.setStatus(`Selected ${commandLabel(row)}`);
}
