import { actionPath } from "../../../../foundation/action-paths.js";
import { commandLabel } from "../../../labels.js";

export function recordDropdownFeedback(controller, dropdown) {
  const label = commandLabel(dropdown);
  controller.recordCommand(actionPath("workbench.dropdown.open", label));
  controller.setStatus(`Dropdown: ${label}`);
}
