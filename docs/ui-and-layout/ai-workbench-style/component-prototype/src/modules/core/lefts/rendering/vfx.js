import { searchInput } from "../../../../components/inputs/atoms.js";
import { assetStrip, listRows, moduleTree, panel, segmentButtons } from "../../../shared/module-components.js";

export function vfxLeft() {
  return [
    panel("Emitter Library", `${segmentButtons(["Emitters", "Modules", "Tools"], 0)}${searchInput("Search emitters...")}${listRows(["Point", "Box", "Sphere", "Cylinder", "Mesh", "Force", "Velocity", "Curl Noise"], 0)}`),
    panel("Content Browser", `${searchInput("Search assets...")}${moduleTree([
      ["VFX", "folder", false, 0],
      ["Systems", "folder", false, 1],
      ["P_Bolt_01", "sun", true, 2],
      ["P_RailTrail", "sun", false, 2],
      ["Textures", "folder", false, 1]
    ])}`),
    panel("Source", assetStrip(["T_Bolt_01", "M_Bolt_01", "T_Noise_01"]))
  ];
}
