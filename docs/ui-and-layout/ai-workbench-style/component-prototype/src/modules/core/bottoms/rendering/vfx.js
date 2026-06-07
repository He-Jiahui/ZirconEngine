import { moduleTable, timeline } from "../../../shared/module-components.js";
import { coreBottomRouteOptions } from "../routes.js";

export function vfxBottom() {
  const routeOptions = coreBottomRouteOptions("vfx", "timeline");
  return `<div class="zr-module-output-grid is-vfx-bottom">${timeline("vfx")}${moduleTable(["Time", "System", "Emitter", "Event"], [
    { cells: ["00:00.00", "P_Bolt_01", "E_Bolt", "Activated"] },
    { cells: ["00:00.01", "P_Bolt_01", "E_Bolt", "Spawn Burst 20"] },
    { cells: ["00:00.45", "P_Bolt_01", "E_Bolt", "Collision 15"], selected: true }
  ], "90px 1fr 1fr 1.4fr", routeOptions)}</div>`;
}
