import { blueprint, checkValue, inputValue, selectValue, tablePrimary, tree } from "../helpers.js";

export const terrainEditorBlueprint = blueprint({
  status: "Terrain sculpt layers and world cells staged",
  actions: [["plus", "Add Layer"], ["grid", "Sculpt Terrain"], ["check", "Build Terrain"], ["play", "Preview Erosion"]],
  tools: ["Sculpt Brush", "Paint Material", "Flatten", "Ramp", "Erosion Mask", "World Cell"],
  assets: tree("Terrain", "globe", ["Landscape_Main", "Heightfield_Ridge", "Layer_Rock", "Layer_Grass", "WorldPartition_A12"]),
  metrics: [["Cells", "64"], ["Layers", "7"], ["Brush", "512"], ["Warnings", "2", "warning"]],
  detailTabs: ["Brush", "Layers", "Streaming"],
  settings: [["Brush Preset", selectValue("Sculpt Soft")], ["Radius", inputValue("512")], ["Strength", inputValue("0.38")], ["Falloff", selectValue("Smooth")], ["Live Preview", checkValue(true)]],
  primary: tablePrimary("Terrain Cell Workspace", ["Cell", "Layer", "State", "LOD"], [["A12_08", "Rock", "Loaded", "3"], ["A12_09", "Grass", "Dirty", "3"], ["A13_08", "Mud", "Queued", "2"], ["Spline_Road_04", "Road", "Ready", "1"]], "1fr 1fr 0.8fr 0.6fr")
});
