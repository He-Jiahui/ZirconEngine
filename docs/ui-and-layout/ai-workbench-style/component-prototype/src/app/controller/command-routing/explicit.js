import { normalizeActionId } from "../../../foundation/action-paths.js";
import { modules } from "../../../modules/modules.js";
import { commandLabel } from "../../labels.js";
import { explicitRouteLabel } from "./label.js";

export function explicitRouteForTarget(target, activeModuleId) {
  const panelTarget = target.dataset.routePanel ?? "";
  const requestedModuleId = target.dataset.routeModule ?? "";
  if (!panelTarget && !requestedModuleId) return null;
  const moduleId = modules.some((module) => module.id === requestedModuleId)
    ? requestedModuleId
    : activeModuleId;
  const command = normalizeActionId(target.dataset.action || commandLabel(target));
  return {
    command,
    moduleId,
    panelTarget,
    label: explicitRouteLabel(command, moduleId, panelTarget)
  };
}
