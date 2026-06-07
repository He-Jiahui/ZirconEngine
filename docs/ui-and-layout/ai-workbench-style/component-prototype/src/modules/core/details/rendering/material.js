import { checkbox, input, searchInput, select } from "../../../../components/inputs/atoms.js";
import { moduleTree, panelGroup, settingsRows } from "../../../shared/module-components.js";
import { coreRightRouteOptions } from "../routes.js";

export function materialDetails() {
  return panelGroup("material-right", [
    { label: "Graph Outline", active: true, content: `${searchInput("Search...")}${moduleTree([
      ["M_Rock_Cliff", "material", false, 0],
      ["Texture Sample", "image", true, 1],
      ["Moss Mask", "image", false, 1],
      ["Multiply", "component", false, 1],
      ["Lerp", "component", false, 1],
      ["Roughness", "component", false, 1]
    ], coreRightRouteOptions("material-right:graph-outline"))}` },
    { label: "Parameters", content: settingsRows([
      ["Tiling", input("", { value: "4.0" })],
      ["Use Moss", checkbox("", true)],
      ["Tint", select("Olive")],
      ["Moss Color", select("Green")],
      ["Roughness", input("", { value: "0.65" })]
    ]) },
    { label: "Node Details", content: settingsRows([
      ["Node Name", input("", { value: "TextureSample_0" })],
      ["Texture", select("T_Rock_Cliff_Albedo")],
      ["Sampler Source", select("From Texture Asset")],
      ["Mip Value Mode", select("None")]
    ]) }
  ]);
}
