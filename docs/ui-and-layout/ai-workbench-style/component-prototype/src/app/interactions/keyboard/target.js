const editableSelector = "input, textarea, select, [contenteditable='true']";
const activationSelector = 'button[data-action], [role="button"]:not(button), [tabindex="0"][data-action]:not(button)';

export function keyboardActivationTarget(event) {
  if (event.target.closest(editableSelector)) return null;
  return event.target.closest(activationSelector);
}
