export function activateToolbarButtonState(button, selector) {
  document.querySelectorAll(selector).forEach((item) => item.classList.remove("is-active"));
  button.classList.add("is-active");
}
