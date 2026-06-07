export function toolbarButtonTarget(event, selector) {
  return event.target.closest(selector);
}
