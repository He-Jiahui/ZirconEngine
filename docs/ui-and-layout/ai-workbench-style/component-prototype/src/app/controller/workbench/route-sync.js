import { normalizeActionId } from "../../../foundation/action-paths.js";
import { syncControllerRouteState } from "../history.js";

export function createWorkbenchRouteSync(state) {
  function syncRouteState({
    moduleId = state.activeModuleId,
    panelTarget = state.activePanelTarget,
    commandId = state.latestCommandId,
    replace = false
  } = {}) {
    syncControllerRouteState({ moduleId, panelTarget, commandId, replace });
  }

  function syncModuleHash(moduleId, replace = false, panelTarget = "", commandId = state.latestCommandId) {
    syncRouteState({ moduleId, panelTarget, commandId, replace });
  }

  function recordCommand(commandId, options = {}) {
    state.latestCommandId = normalizeActionId(commandId);
    if (!state.latestCommandId) return;
    syncRouteState({ commandId: state.latestCommandId, replace: options.replaceHistory });
  }

  return { syncModuleHash, recordCommand };
}
