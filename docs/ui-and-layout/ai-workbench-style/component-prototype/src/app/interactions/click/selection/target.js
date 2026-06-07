export function selectionControlTarget(event, selector) {
  return event.target.closest(selector);
}
