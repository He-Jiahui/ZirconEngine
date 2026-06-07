import { applyLocationModuleState } from "../location-state.js";

export function createWorkbenchLocationHandler({
  state,
  syncModuleHash,
  activatePanelTarget,
  resetModulePanels,
  setStatus
}) {
  return function activateLocationModuleState(options = {}) {
    applyLocationModuleState({
      state,
      syncModuleHash,
      activatePanelTarget,
      resetModulePanels,
      setStatus
    }, options);
  };
}
