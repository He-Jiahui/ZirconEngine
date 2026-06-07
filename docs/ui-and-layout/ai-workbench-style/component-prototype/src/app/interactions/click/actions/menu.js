export function closeActionPopupLayer(controller, action) {
  if (action.closest(".zr-popup-layer")) {
    controller.popup?.classList.remove("is-open");
  }
}
