import { searchInput } from "../../../components/inputs/atoms.js";
import { listRows, moduleTree, panel } from "../../shared/module-components.js";

export function sceneLeft() {
  return [
    panel("Hierarchy", `${searchInput("Search scene...")}${moduleTree([
      ["Root", "cube", true, 0],
      ["Environment", "folder", false, 1],
      ["Lighting", "sun", false, 2],
      ["Level Geometry", "grid", false, 1],
      ["Props", "cube", true, 2],
      ["PlayerStart", "target", false, 1]
    ])}`),
    panel("Layers", listRows(["Gameplay", "Environment", "Lighting", "Audio", "Debug"], 0))
  ];
}
