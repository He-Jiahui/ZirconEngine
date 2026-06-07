import { blueprint, checkValue, inputValue, selectValue, tablePrimary, tree } from "../helpers.js";

export const levelVariantBlueprint = blueprint({
  status: "Level variant override stack selected",
  actions: [["plus", "Add Variant"], ["target", "Apply Variant"], ["check", "Validate Overrides"], ["history", "Review Diff"]],
  tools: ["Variant Set", "Actor Override", "Material Swap", "Visibility Override", "Property Capture", "Diff"],
  assets: tree("Variants", "columns", ["Vehicle_Showcase", "Variant_Red", "Variant_Blue", "Override_Material", "Actor_CarBody"]),
  metrics: [["Variants", "18"], ["Overrides", "124"], ["Conflicts", "2", "warning"], ["Actors", "42"]],
  detailTabs: ["Variant", "Overrides", "Diff"],
  settings: [["Variant", selectValue("Variant_Red")], ["Set", selectValue("Vehicle Showcase")], ["Capture Mode", selectValue("Selected Props")], ["Auto Apply", checkValue(false)], ["Record Diff", checkValue(true)]],
  primary: tablePrimary("Variant Overrides", ["Actor", "Property", "Value", "State"], [["CarBody", "Material", "M_RedPaint", "Selected"], ["Wheel_FL", "Visible", "true", "Ready"], ["Light_Rig", "Intensity", "4.2", "Ready"], ["Door_L", "Transform", "Conflict", "Warning"]], "1fr 1fr 1fr 0.8fr")
});
