import { activateKeyboardTarget } from "./activate.js";
import { isKeyboardActivationEvent } from "./filter.js";
import { keyboardActivationTarget } from "./target.js";

export function bindKeyboardActivation() {
  document.addEventListener("keydown", (event) => {
    if (!isKeyboardActivationEvent(event)) return;
    const target = keyboardActivationTarget(event);
    if (!target) return;
    activateKeyboardTarget(event, target);
  });
}
