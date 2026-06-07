export function editableFieldTarget(event) {
  return event.target.closest("input:not([disabled]), textarea:not([disabled])");
}
