import { flashElement, handledClick, ignoredClick } from "../utils.js";
import { recordActionFallbackFeedback } from "./feedback.js";
import { activateActionGroupState } from "./group.js";
import { closeActionPopupLayer } from "./menu.js";
import { actionClickTarget } from "./target.js";

export function handleActionClick(event, controller) {
  const action = actionClickTarget(event);
  if (!action) return ignoredClick;
  flashElement(action);
  activateActionGroupState(action);
  if (!controller.applyCommandRoute(action)) {
    recordActionFallbackFeedback(controller, action);
  }
  closeActionPopupLayer(controller, action);
  return handledClick;
}
