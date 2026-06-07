import { normalizeActionId } from "../../../foundation/action-paths.js";
import { moduleById } from "../../../modules/modules.js";

export function createModuleActivation({
  state,
  renderWorkbench,
  setStatus,
  syncModuleHash,
  activatePanelTarget
}) {
  return function activateModule(moduleId, statusMessage = `Opened ${moduleById(moduleId).label}`, options = {}) {
    const resolvedModule = moduleById(moduleId);
    state.activeModuleId = resolvedModule.id;
    state.activePanelTarget = "";
    state.latestCommandId = options.commandId ? normalizeActionId(options.commandId) : "";
    state.latestStatus = moduleById(state.activeModuleId).status;
    renderWorkbench();
    if (options.panelTarget) {
      activatePanelTarget(options.panelTarget, { fromHistory: true });
    }
    if (!options.fromHistory) {
      syncModuleHash(state.activeModuleId, options.replaceHistory, state.activePanelTarget, state.latestCommandId);
    }
    setStatus(statusMessage);
  };
}
