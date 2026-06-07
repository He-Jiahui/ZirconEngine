import { handleFieldFocus } from "./focus.js";
import { handleFieldInput } from "./input.js";

export function bindFieldInteractions(controller) {
  document.addEventListener("focusin", (event) => handleFieldFocus(event, controller));
  document.addEventListener("input", (event) => handleFieldInput(event, controller));
}
