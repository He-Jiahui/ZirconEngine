import { moduleById } from "../../../modules/modules.js";

export function locationStatusMessage(state, nextModuleId) {
  const panelLabel = state.activePanelTarget
    ? ` / ${state.activePanelTarget.split(":").at(-1).replace(/-/g, " ")}`
    : "";
  const commandText = state.latestCommandId ? ` / ${state.latestCommandId.replace(/-/g, " ")}` : "";
  return `History: ${moduleById(nextModuleId).label}${panelLabel}${commandText}`;
}
