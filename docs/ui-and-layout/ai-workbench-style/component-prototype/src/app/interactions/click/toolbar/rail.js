import { handledClick, ignoredClick } from "../utils.js";
import { recordRailToolbarFeedback } from "./feedback.js";
import { activateToolbarButtonState } from "./state.js";
import { toolbarButtonTarget } from "./target.js";

const railButtonSelector = ".zr-rail .zr-icon-button";

export function handleRailClick(event, controller) {
  const railButton = toolbarButtonTarget(event, railButtonSelector);
  if (!railButton) return ignoredClick;

  activateToolbarButtonState(railButton, railButtonSelector);
  recordRailToolbarFeedback(controller, railButton);
  return handledClick;
}
