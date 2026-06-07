import { actionButton, settingsRows } from "../../../shared/module-components.js";

export function extensionSummaryPanel(config) {
  return `${settingsRows(config.settings)}${actionButton(config.actions[0][1], config.actions[0][0])}`;
}
