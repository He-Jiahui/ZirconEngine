import { flashElement, handledClick, ignoredClick } from "../utils.js";
import { recordGenericCommandFeedback } from "./feedback.js";
import { genericCommandTarget } from "./target.js";

export function handleGenericCommandClick(event, controller, alreadyHandled) {
  if (alreadyHandled) return ignoredClick;
  const genericButton = genericCommandTarget(event);
  if (!genericButton) return ignoredClick;
  flashElement(genericButton);
  recordGenericCommandFeedback(controller, genericButton);
  return handledClick;
}
