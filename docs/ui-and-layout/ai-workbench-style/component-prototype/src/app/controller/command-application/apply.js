import { commandRouteForTarget } from "../command-routing.js";
import { applyModuleCommandRoute } from "./module.js";
import { applyPanelCommandRoute } from "./panel.js";
import { recordPlainCommandRoute } from "./record.js";
import { commandRouteStatusMessage } from "./status.js";

export function applyCommandRouteForTarget(target, { state, activateModule, activatePanelTarget, recordCommand, setStatus }) {
  const route = commandRouteForTarget(target, state.activeModuleId);
  if (!route) return false;
  if (route.moduleId !== state.activeModuleId) {
    applyModuleCommandRoute(route, { activateModule });
  } else if (route.panelTarget) {
    applyPanelCommandRoute(route, { activatePanelTarget });
  } else {
    recordPlainCommandRoute(route, { activatePanelTarget, recordCommand });
  }
  setStatus(commandRouteStatusMessage(route));
  return true;
}
