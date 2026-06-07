import { alerts } from "../../../components/data/collections.js";
import { moduleTable } from "../../shared/module-components.js";
import { coreBottomRouteOptions } from "./routes.js";

export function sceneBottom() {
  const routeOptions = coreBottomRouteOptions("scene", "selection");
  return `${moduleTable(["Name", "Type", "Size", "Modified"], [
    { cells: ["Item_01", "Mesh", "2.4 MB", "2m ago"] },
    { cells: ["Item_02", "Material", "512 KB", "10m ago"], selected: true },
    { cells: ["Item_03", "Texture", "1.20 MB", "1m ago"] }
  ], "minmax(120px,1.2fr) 110px 90px 120px", routeOptions)}${alerts([["info", "Scene selection ready"], ["success", "No runtime errors"], ["warning", "2 layout warnings"]])}`;
}
