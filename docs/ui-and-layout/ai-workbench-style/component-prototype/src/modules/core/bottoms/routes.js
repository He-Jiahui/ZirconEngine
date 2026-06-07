import { actionSegment } from "../../../foundation/action-paths.js";

export function coreBottomRouteOptions(moduleId, panelKey) {
  return {
    actionScope: `workbench.module.bottom.${actionSegment(moduleId)}.${actionSegment(panelKey)}`,
    routePanel: `module-bottom-${moduleId}:${panelKey}`
  };
}
