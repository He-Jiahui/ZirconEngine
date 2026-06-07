import {
  commandIdFromLocation,
  moduleIdFromLocation,
  requestedModuleIdFromLocation,
  requestedPanelTargetFromLocation
} from "../../route-state.js";

export function locationStateRequest() {
  const requestedModuleId = requestedModuleIdFromLocation();
  const rawRequestedPanelTarget = requestedPanelTargetFromLocation();
  const requestedCommandId = commandIdFromLocation();
  const nextModuleId = moduleIdFromLocation();
  return {
    requestedModuleId,
    rawRequestedPanelTarget,
    requestedCommandId,
    nextModuleId,
    requestedPanelTarget: requestedModuleId === nextModuleId ? rawRequestedPanelTarget : ""
  };
}
