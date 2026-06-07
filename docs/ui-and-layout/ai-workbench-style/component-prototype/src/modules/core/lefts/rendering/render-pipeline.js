import { searchInput } from "../../../../components/inputs/atoms.js";
import { listRows, moduleTree, panel } from "../../../shared/module-components.js";

export function renderPipelineLeft() {
  return [
    panel("Pass Palette", `${searchInput("Search passes...")}${listRows(["Render Pass", "Compute Pass", "Copy Pass", "Clear Pass", "Shadow Pass", "Lighting Pass", "Reflection Pass", "Bloom Pass", "Tone Map Pass", "Debug Pass"], 5)}`),
    panel("Pipeline Assets", `${searchInput("Search assets...")}${moduleTree([
      ["Pipelines", "folder", false, 0],
      ["MainPipeline.rp", "renderer", true, 1],
      ["MobilePipeline.rp", "renderer", false, 1],
      ["Passes", "folder", false, 0],
      ["Lighting", "folder", false, 1],
      ["PostProcess", "folder", false, 1],
      ["Shaders", "folder", false, 0],
      ["Textures", "folder", false, 0]
    ])}`)
  ];
}
