import { blueprint, checkValue, graphPrimary, inputValue, selectValue, tree } from "../helpers.js";

export const scatterEditorBlueprint = blueprint({
  status: "Procedural scatter rule stack selected",
  actions: [["plus", "Add Rule"], ["grid", "Generate Scatter"], ["check", "Validate Scatter"], ["play", "Preview Scatter"]],
  tools: ["Spawn Rule", "Density Map", "Slope Filter", "Biome Mask", "Collision Test", "Seed Preview"],
  assets: tree("Scatter", "globe", ["SC_Forest", "Rule_Rocks", "Rule_Ferns", "Mask_Slope", "Seed_2026"]),
  metrics: [["Rules", "18"], ["Instances", "64K"], ["Conflicts", "1", "warning"], ["Seed", "2026"]],
  detailTabs: ["Rules", "Constraints", "Output"],
  settings: [["Rule Set", selectValue("SC_Forest")], ["Seed", inputValue("2026")], ["Density", inputValue("0.64")], ["Avoid Collisions", checkValue(true)], ["Strict Bounds", checkValue(true)]],
  primary: graphPrimary("Scatter Rule Graph", [["Biome Mask", "Input", 12, 30, "cyan"], ["Slope Filter", "Constraint", 34, 22, "blue"], ["Spawn Rule", "Rule", 56, 42, "green"], ["Collision Test", "Validation", 38, 66, "orange"], ["Output Set", "Instances", 76, 58, "purple"]])
});
