import { routeForCommand } from "../../../routing/routes.js";
import { commandLabel } from "../../labels.js";

export function fallbackCommandRoute(target, activeModuleId) {
  return routeForCommand(target.dataset.action || commandLabel(target), activeModuleId);
}
