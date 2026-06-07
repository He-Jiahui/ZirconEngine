import { ignoredClick } from "../utils.js";
import { closeDropdownPopupState } from "./state.js";
import { popupDismissalTarget } from "./target.js";

export function handlePopupDismissal(event, controller) {
  if (!popupDismissalTarget(event)) {
    closeDropdownPopupState(controller.popup);
  }
  return ignoredClick;
}
