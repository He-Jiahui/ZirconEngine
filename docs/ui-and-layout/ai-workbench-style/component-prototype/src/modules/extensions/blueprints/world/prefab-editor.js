import { blueprint, checkValue, graphPrimary, inputValue, selectValue, tree } from "../helpers.js";

export const prefabEditorBlueprint = blueprint({
  status: "Prefab nested hierarchy and validation active",
  actions: [["plus", "Add Child"], ["target", "Apply Override"], ["check", "Validate Prefab"], ["save", "Save Prefab"]],
  tools: ["Component Add", "Nested Prefab", "Override Diff", "Variant", "Validation", "Placement"],
  assets: tree("Prefabs", "cube", ["PF_Chest", "Mesh_Chest", "LootSocket", "Light_Glow", "Override_Open"]),
  metrics: [["Children", "18"], ["Overrides", "6"], ["Refs", "32"], ["Warnings", "2", "warning"]],
  detailTabs: ["Hierarchy", "Overrides", "Validation"],
  settings: [["Prefab", selectValue("PF_Chest")], ["Variant", selectValue("Default")], ["Instance ID", inputValue("Chest_04")], ["Propagate", checkValue(true)], ["Lock Root", checkValue(false)]],
  primary: graphPrimary("Prefab Composition", [["PF_Chest", "Root", 12, 30, "cyan"], ["Mesh", "Component", 34, 20, "blue"], ["LootSocket", "Socket", 54, 44, "green"], ["Light", "Component", 34, 68, "orange"], ["Override", "Instance", 74, 58, "purple"]])
});
