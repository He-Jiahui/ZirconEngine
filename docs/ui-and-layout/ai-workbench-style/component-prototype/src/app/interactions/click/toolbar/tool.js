import { handledClick, ignoredClick } from "../utils.js";
import { recordToolToolbarFeedback } from "./feedback.js";
import { activateToolbarButtonState } from "./state.js";
import { toolbarButtonTarget } from "./target.js";

const toolButtonSelector = ".zr-topbar-tools .zr-icon-button";

export function handleToolClick(event, controller) {
  const toolButton = toolbarButtonTarget(event, toolButtonSelector);
  if (!toolButton) return ignoredClick;

  activateToolbarButtonState(toolButton, toolButtonSelector);
  recordToolToolbarFeedback(controller, toolButton);
  return handledClick;
}
