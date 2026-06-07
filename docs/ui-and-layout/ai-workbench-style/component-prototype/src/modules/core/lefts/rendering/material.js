import { searchInput } from "../../../../components/inputs/atoms.js";
import { listRows, moduleTree, panel, previewTile } from "../../../shared/module-components.js";

export function materialLeft() {
  return [
    panel("Node Palette", `${searchInput("Search nodes...")}${listRows(["Texture Sample", "Multiply", "Lerp", "Scalar Parameter", "Vector Parameter", "Roughness"], 0)}`),
    panel("Material Preview", previewTile("material")),
    panel("Assets", `${searchInput("Search assets...")}${moduleTree([
      ["Game/Materials", "folder", false, 0],
      ["Environment", "folder", false, 1],
      ["M_Rock_Cliff", "material", true, 2],
      ["M_Wet_Rock", "material", false, 2],
      ["Functions", "folder", false, 1]
    ])}`)
  ];
}
