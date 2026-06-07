import { handledClick, ignoredClick } from "../utils.js";
import { recordToggleFeedback } from "./feedback.js";
import { applyToggleSelectionState } from "./state.js";
import { selectionControlTarget } from "./target.js";

export function handleToggleClick(event, controller) {
  const toggle = selectionControlTarget(event, "[data-toggle]");
  if (!toggle) return ignoredClick;

  applyToggleSelectionState(toggle);
  recordToggleFeedback(controller, toggle);
  return handledClick;
}
