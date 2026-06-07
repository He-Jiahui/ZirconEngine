import { checkbox, input, select } from "../../../../components/inputs/atoms.js";

export const worldRecipe = {
  detailTabs: ["Brush", "Layers", "Streaming"],
  actions: (subject, shortLabel) => [["plus", `Add ${shortLabel}`], ["grid", `Paint ${shortLabel}`], ["check", `Build ${shortLabel}`], ["play", `Preview ${shortLabel}`]],
  tools: (subject) => ["Sculpt", "Paint Layer", "Spline Tool", "Scatter Mask", `${subject} Preview`, "Streaming Cell"],
  metrics: () => [["Tiles", "64"], ["LOD", "5"], ["Layers", "7"], ["Warnings", "2", "warning"]],
  settings: (subject) => [["Brush", select(`${subject} Brush`)], ["Radius", input("", { value: "512" })], ["Strength", input("", { value: "0.38" })], ["Falloff", select("Smooth")], ["Live Preview", checkbox("", true)]],
  table: (subject) => [[`${subject}_Tile_12_08`, "Loaded", "1.2 ms"], [`${subject}_Tile_12_09`, "Loaded", "1.4 ms"], [`${subject}_Layer_Rock`, "Dirty", "Queued"], [`${subject}_Cell_A`, "Visible", "High"]]
};
