import { ignoredClick, stoppedClick } from "../utils.js";
import { recordDropdownFeedback } from "./feedback.js";
import { positionDropdownPopup } from "./placement.js";
import { toggleDropdownPopupState } from "./state.js";
import { dropdownTriggerTarget } from "./target.js";

export function handleDropdownClick(event, controller) {
  const dropdown = dropdownTriggerTarget(event);
  if (!dropdown || !controller.popup) return ignoredClick;

  positionDropdownPopup(controller.popup, dropdown);
  toggleDropdownPopupState(controller.popup);
  recordDropdownFeedback(controller, dropdown);
  return stoppedClick;
}
