import { moduleTable, progress } from "../../shared/module-components.js";
import { coreBottomRouteOptions } from "./routes.js";

export function assetBottom() {
  const routeOptions = coreBottomRouteOptions("asset-browser", "queue");
  return `<div class="zr-module-output-grid">
    ${moduleTable(["ID", "Task", "Path", "Status", "Progress"], [
      { cells: ["IMP-1021", "Import FBX", "/Game/Forest/SM_Cliff_Rock_02.fbx", "Importing", progress(62)] },
      { cells: ["IMP-1022", "Import Textures", "/Game/Textures/T_Forest_Rock_01.*", "Queued", progress(0)] },
      { cells: ["VAL-2041", "Validate Assets", "/Game/Environments/Forest/*", "Queued", progress(0)] }
    ], "76px 140px 1.6fr 100px 130px", routeOptions)}
    <div class="zr-module-log"><p>10:20:11 Import started: SM_Cliff_Rock_02.fbx</p><p class="is-warning">10:20:12 2 warnings</p><p class="is-error">10:20:15 Error: invalid collision</p></div>
  </div>`;
}
