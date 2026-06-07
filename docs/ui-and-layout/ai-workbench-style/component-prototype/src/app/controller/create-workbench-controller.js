import { createActivationHandlers } from "./activation.js";
import { createControllerState } from "./state.js";
import { createWorkbenchCommandHandler } from "./workbench/commands.js";
import { createWorkbenchLocationHandler } from "./workbench/location.js";
import { createWorkbenchRenderLoop } from "./workbench/render-loop.js";
import { createWorkbenchRouteSync } from "./workbench/route-sync.js";

export function createWorkbenchController(app) {
  const state = createControllerState();
  const { renderWorkbench, setStatus } = createWorkbenchRenderLoop(app, state);
  const { syncModuleHash, recordCommand } = createWorkbenchRouteSync(state);

  const { activateModule, activatePanelTarget, resetModulePanels } = createActivationHandlers({
    state,
    renderWorkbench,
    setStatus,
    syncModuleHash
  });

  const activateLocationModuleState = createWorkbenchLocationHandler({
    state,
    syncModuleHash,
    activatePanelTarget,
    resetModulePanels,
    setStatus
  });
  const applyCommandRoute = createWorkbenchCommandHandler({
    state,
    activateModule,
    activatePanelTarget,
    recordCommand,
    setStatus
  });

  return {
    get popup() {
      return state.popup;
    },
    activateLocationModuleState,
    activateModule,
    activatePanelTarget,
    applyCommandRoute,
    recordCommand,
    renderWorkbench,
    setStatus
  };
}
