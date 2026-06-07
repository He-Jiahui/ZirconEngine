import { normalizeActionId } from "../../../foundation/action-paths.js";
import { applyPanelRoute } from "../../../routing/routes.js";

export function createPanelActivation({ state, setStatus, syncModuleHash }) {
  return function activatePanelTarget(panelTarget, options = {}) {
    if (!panelTarget) {
      state.activePanelTarget = "";
      if (options.clearCommand) {
        state.latestCommandId = "";
      }
      if (!options.fromHistory) {
        syncModuleHash(state.activeModuleId, options.replaceHistory, "", state.latestCommandId);
      }
      return false;
    }
    if (!applyPanelRoute(panelTarget)) {
      state.activePanelTarget = "";
      if (options.clearCommand) {
        state.latestCommandId = "";
      }
      if (!options.fromHistory) {
        syncModuleHash(state.activeModuleId, true, "", state.latestCommandId);
      }
      return false;
    }
    state.activePanelTarget = panelTarget;
    if (options.commandId) {
      state.latestCommandId = normalizeActionId(options.commandId);
    }
    if (!options.fromHistory) {
      syncModuleHash(state.activeModuleId, options.replaceHistory, state.activePanelTarget, state.latestCommandId);
    }
    if (options.statusMessage) {
      setStatus(options.statusMessage);
    }
    return true;
  };
}
