import { alerts } from "../../../../components/data/collections.js";
import { actionButton } from "../../../shared/module-components.js";

export function extensionDetailStatusPanel(config) {
  return `${alerts([["info", `${config.label} follows the shared module panel contract`], ["success", "Visible controls route through prototype feedback"], ["warning", "Native implementation pending"]])}${actionButton("More Editors", "grid")}`;
}
