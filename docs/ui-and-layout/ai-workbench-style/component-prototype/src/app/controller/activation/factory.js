import { createModuleActivation } from "./module.js";
import { createPanelActivation } from "./panel.js";
import { createPanelReset } from "./reset.js";

export function createActivationHandlers({ state, renderWorkbench, setStatus, syncModuleHash }) {
  const activatePanelTarget = createPanelActivation({ state, setStatus, syncModuleHash });
  const resetModulePanels = createPanelReset({ state, renderWorkbench });
  const activateModule = createModuleActivation({
    state,
    renderWorkbench,
    setStatus,
    syncModuleHash,
    activatePanelTarget
  });

  return { activateModule, activatePanelTarget, resetModulePanels };
}
