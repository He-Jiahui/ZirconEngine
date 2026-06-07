import { actionPath } from "../../../../foundation/action-paths.js";
import { commandLabel } from "../../../labels.js";

export function recordToggleFeedback(controller, toggle) {
  const label = commandLabel(toggle);
  controller.recordCommand(actionPath("workbench.toggle", label));
  controller.setStatus(`Toggled ${label}`);
}

export function recordRadioFeedback(controller, radio) {
  const label = commandLabel(radio);
  controller.recordCommand(actionPath("workbench.radio", label));
  controller.setStatus(`Selected ${label}`);
}
