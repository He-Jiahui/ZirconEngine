import { normalizeActionId } from "../../../foundation/action-paths.js";
import { applyLocationModuleChange } from "./module.js";
import { applyLocationPanelTarget } from "./panel.js";
import { locationStateRequest } from "./request.js";
import { locationStatusMessage } from "./status.js";

export function applyLocationModuleState(services, options = {}) {
  const request = locationStateRequest();
  const { state, syncModuleHash, setStatus } = services;
  applyRedirectedModuleHash(syncModuleHash, request);
  const stateModuleChanged = applyLocationModuleChange(services, request);
  const panelApplied = applyLocationPanelTarget(services, request);
  applyInvalidPanelRedirect(syncModuleHash, request, panelApplied);
  state.latestCommandId = request.requestedCommandId ? normalizeActionId(request.requestedCommandId) : "";
  if (!options.silent && (stateModuleChanged || request.requestedPanelTarget || state.activePanelTarget || state.latestCommandId)) {
    setStatus(locationStatusMessage(state, request.nextModuleId));
  }
}

function applyRedirectedModuleHash(syncModuleHash, request) {
  if (request.requestedModuleId !== request.nextModuleId) {
    syncModuleHash(request.nextModuleId, true, "", "");
  }
}

function applyInvalidPanelRedirect(syncModuleHash, request, panelApplied) {
  if (request.rawRequestedPanelTarget && (!request.requestedPanelTarget || !panelApplied)) {
    syncModuleHash(request.nextModuleId, true, "", request.requestedCommandId);
  }
}
