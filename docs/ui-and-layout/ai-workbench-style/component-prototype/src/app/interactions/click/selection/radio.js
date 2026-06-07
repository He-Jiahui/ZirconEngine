import { handledClick, ignoredClick } from "../utils.js";
import { recordRadioFeedback } from "./feedback.js";
import { applyRadioSelectionState } from "./state.js";
import { selectionControlTarget } from "./target.js";

export function handleRadioClick(event, controller) {
  const radio = selectionControlTarget(event, "[data-radio]");
  if (!radio) return ignoredClick;

  applyRadioSelectionState(radio);
  recordRadioFeedback(controller, radio);
  return handledClick;
}
