import { actionPath } from "../../../../foundation/action-paths.js";
import { commandLabel } from "../../../labels.js";

export function recordGenericCommandFeedback(controller, genericButton) {
  const label = commandLabel(genericButton);
  controller.recordCommand(actionPath("workbench.command", label));
  controller.setStatus(`Command: ${label}`);
}
