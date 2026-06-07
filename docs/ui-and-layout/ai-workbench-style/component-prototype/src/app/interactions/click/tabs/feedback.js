import { actionPath } from "../../../../foundation/action-paths.js";
import { commandLabel } from "../../../labels.js";

export function recordPlainTabFeedback(controller, tab) {
  controller.recordCommand(actionPath("workbench.tab.select", commandLabel(tab)));
}

export function setTabStatus(controller, tab) {
  controller.setStatus(`Tab: ${commandLabel(tab)}`);
}
