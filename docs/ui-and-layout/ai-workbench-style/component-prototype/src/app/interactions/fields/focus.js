import { actionPath } from "../../../foundation/action-paths.js";
import { fieldLabel } from "../../labels.js";
import { editableFieldTarget } from "./target.js";

export function handleFieldFocus(event, controller) {
  const field = editableFieldTarget(event);
  if (!field) return;
  field.classList.add("is-focused");
  controller.recordCommand(actionPath("workbench.field.focus", fieldLabel(field)), { replaceHistory: true });
  controller.setStatus(`Focused: ${fieldLabel(field)}`);
}
