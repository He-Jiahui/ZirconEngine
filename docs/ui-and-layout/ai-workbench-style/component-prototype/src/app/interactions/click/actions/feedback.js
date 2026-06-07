import { actionPath } from "../../../../foundation/action-paths.js";
import { commandLabel } from "../../../labels.js";

export function recordActionFallbackFeedback(controller, action) {
  const label = commandLabel(action);
  controller.recordCommand(action.dataset.action || actionPath("workbench.action", label));
  controller.setStatus(`Action: ${label}`);
}
