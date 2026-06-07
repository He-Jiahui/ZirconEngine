import { actionPath } from "../../../foundation/action-paths.js";
import { fieldLabel } from "../../labels.js";
import { editableFieldTarget } from "./target.js";

export function handleFieldInput(event, controller) {
  const field = editableFieldTarget(event);
  if (!field) return;
  controller.recordCommand(actionPath("workbench.field.edit", fieldLabel(field)), { replaceHistory: true });
  controller.setStatus(`Edited: ${fieldLabel(field)}`);
}
