import { ignoredClick, stoppedClick } from "../utils.js";
import { activateModuleNavigation } from "./activate.js";
import { moduleNavigationTarget } from "./target.js";

export function handleModuleNavigation(event, controller) {
  const moduleButton = moduleNavigationTarget(event);
  if (!moduleButton) return ignoredClick;
  activateModuleNavigation(controller, moduleButton);
  return stoppedClick;
}
