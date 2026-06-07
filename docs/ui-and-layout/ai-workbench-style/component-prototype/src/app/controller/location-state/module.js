import { moduleById } from "../../../modules/modules.js";

export function applyLocationModuleChange({ state, resetModulePanels }, { nextModuleId, requestedPanelTarget }) {
  const moduleChanged = nextModuleId !== state.activeModuleId;
  if (moduleChanged) {
    state.activeModuleId = nextModuleId;
    state.latestStatus = moduleById(state.activeModuleId).status;
    resetModulePanels();
  } else if (!requestedPanelTarget && state.activePanelTarget) {
    resetModulePanels();
  }
  return moduleChanged;
}
