import { findPopupLayer, renderWorkbenchShell } from "../rendering.js";
import { updateStatusMessage } from "../status.js";

export function createWorkbenchRenderLoop(app, state) {
  function renderWorkbench() {
    renderWorkbenchShell(app, state.activeModuleId, state.latestStatus);
    state.popup = findPopupLayer();
  }

  function setStatus(message) {
    state.latestStatus = message;
    updateStatusMessage(message);
  }

  return { renderWorkbench, setStatus };
}
