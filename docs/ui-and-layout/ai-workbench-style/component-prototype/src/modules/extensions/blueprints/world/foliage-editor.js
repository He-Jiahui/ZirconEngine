import { blueprint, checkValue, inputValue, selectValue, tablePrimary, tree } from "../helpers.js";

export const foliageEditorBlueprint = blueprint({
  status: "Foliage brush and biome clusters visible",
  actions: [["plus", "Add Foliage"], ["grid", "Paint Foliage"], ["check", "Build Clusters"], ["play", "Preview Density"]],
  tools: ["Paint Brush", "Erase Brush", "Density Mask", "Biome Rule", "Cluster Bake", "Scatter Preview"],
  assets: tree("Foliage", "globe", ["FOL_Forest", "Oak_Tall", "Fern_A", "Grass_Clump", "Biome_Riverbank"]),
  metrics: [["Instances", "84K"], ["Types", "12"], ["Clusters", "128"], ["Warnings", "2", "warning"]],
  detailTabs: ["Brush", "Types", "Clusters"],
  settings: [["Foliage Type", selectValue("Oak_Tall")], ["Density", inputValue("0.72")], ["Radius", inputValue("480")], ["Align Normal", checkValue(true)], ["Cast Shadows", checkValue(true)]],
  primary: tablePrimary("Foliage Cluster Workspace", ["Cluster", "Type", "Density", "State"], [["Forest_A12", "Oak", "0.72", "Ready"], ["Forest_A13", "Fern", "0.58", "Selected"], ["River_02", "Grass", "0.81", "Queued"], ["Cliff_01", "Shrub", "0.24", "Warning"]], "1fr 0.8fr 0.8fr 0.8fr")
});
