const activationKeys = ["Enter", " ", "Spacebar"];

export function isKeyboardActivationEvent(event) {
  if (event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey) return false;
  return activationKeys.includes(event.key);
}
