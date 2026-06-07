import { actionPath } from "../../../../foundation/action-paths.js";
import { commandLabel } from "../../../labels.js";

export function recordRailToolbarFeedback(controller, railButton) {
  controller.recordCommand(actionPath("workbench.rail.select", commandLabel(railButton)));
}

export function recordToolToolbarFeedback(controller, toolButton) {
  controller.recordCommand(actionPath("workbench.tool.select", commandLabel(toolButton)));
}
