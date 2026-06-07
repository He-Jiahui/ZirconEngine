import { actionRouteKey, normalizeActionId } from "../../foundation/action-paths.js";
import { moduleRouteMap } from "./module-targets.js";
import { panelRouteMap, resolvePanelTarget } from "./panel-targets.js";
import { moduleScopedRouteMap } from "./scoped-targets.js";
import { extensionRouteForCommand } from "./extension-targets.js";
import { routeLabel } from "./labels.js";

export function routeForCommand(command, activeModuleId) {
  const normalized = normalizeCommand(command);
  const actionId = normalizeActionId(command);
  const scopedRoute = moduleScopedRouteMap.get(`${activeModuleId}:${normalized}`)
    ?? extensionRouteForCommand(normalized, activeModuleId);
  const nextModuleId = scopedRoute?.moduleId ?? moduleRouteMap.get(normalized);
  const panelTarget = resolvePanelTarget(
    scopedRoute?.panelTarget ?? panelRouteMap.get(normalized),
    nextModuleId ?? activeModuleId
  );
  if (!nextModuleId && !panelTarget) return null;
  return {
    command: actionId,
    moduleId: nextModuleId ?? activeModuleId,
    panelTarget,
    label: routeLabel(normalized, nextModuleId ?? activeModuleId, panelTarget)
  };
}

export function normalizeCommand(value) {
  return actionRouteKey(value);
}
