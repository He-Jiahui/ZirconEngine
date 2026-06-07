export const ignoredClick = Object.freeze({ handled: false, stop: false });
export const handledClick = Object.freeze({ handled: true, stop: false });
export const stoppedClick = Object.freeze({ handled: true, stop: true });

export function flashElement(element) {
  element.classList.remove("zr-action-flash");
  requestAnimationFrame(() => element.classList.add("zr-action-flash"));
}
