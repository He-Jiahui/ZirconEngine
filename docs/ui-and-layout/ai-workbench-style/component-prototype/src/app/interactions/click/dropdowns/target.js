export function dropdownTriggerTarget(event) {
  return event.target.closest("[data-dropdown]");
}

export function popupDismissalTarget(event) {
  return event.target.closest(".zr-popup-layer");
}
