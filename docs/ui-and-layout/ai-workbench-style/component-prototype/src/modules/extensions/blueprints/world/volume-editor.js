import { blueprint, checkValue, graphPrimary, inputValue, selectValue, tree } from "../helpers.js";

export const volumeEditorBlueprint = blueprint({
  status: "Volume overlap and bounds details selected",
  actions: [["plus", "Add Volume"], ["target", "Inspect Overlap"], ["check", "Validate Volume"], ["play", "Preview Volume"]],
  tools: ["Box Volume", "Sphere Volume", "Bounds Edit", "Overlap Rule", "Profile", "Event Output"],
  assets: tree("Volumes", "cube", ["VOL_DamageZone", "Profile_Default", "Overlap_Player", "Event_OnEnter", "Bounds_A"]),
  metrics: [["Volumes", "24"], ["Overlaps", "12"], ["Events", "8"], ["Warnings", "1", "warning"]],
  detailTabs: ["Bounds", "Overlaps", "Events"],
  settings: [["Volume", selectValue("VOL_DamageZone")], ["Profile", selectValue("Damage")], ["Priority", inputValue("10")], ["Generate Events", checkValue(true)], ["Draw Bounds", checkValue(true)]],
  primary: graphPrimary("Volume Overlap Workspace", [["Volume", "Bounds", 14, 34, "cyan"], ["Player", "Overlap", 38, 24, "blue"], ["Damage Rule", "Effect", 60, 42, "green"], ["OnEnter", "Event", 42, 68, "orange"], ["OnExit", "Event", 78, 60, "purple"]])
});
