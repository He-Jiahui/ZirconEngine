import { actionSegment } from "../../../foundation/action-paths.js";

export function coreRightRouteOptions(panel) {
  const [group, tab = "details"] = String(panel).split(":");
  return {
    actionScope: `workbench.module.right.${actionSegment(group)}.${actionSegment(tab)}`,
    routePanel: panel
  };
}
