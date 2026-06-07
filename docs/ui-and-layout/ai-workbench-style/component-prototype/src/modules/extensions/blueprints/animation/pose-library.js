import { blueprint, checkValue, inputValue, selectValue, tablePrimary, tree } from "../helpers.js";

export const poseLibraryBlueprint = blueprint({
  status: "Pose set and capture metadata selected",
  actions: [["plus", "Capture Pose"], ["play", "Preview Pose"], ["check", "Validate Pose"], ["save", "Export Pose"]],
  tools: ["Pose Capture", "Pose Set", "Mirror Pose", "Metadata", "Batch Apply", "Thumbnail"],
  assets: tree("Poses", "history", ["Pose_Combat", "Idle_A", "Aim_Offset", "Crouch_Start", "Pose_Metadata"]),
  metrics: [["Poses", "184"], ["Sets", "12"], ["Mirrored", "86"], ["Warnings", "1", "warning"]],
  detailTabs: ["Pose", "Metadata", "Batch"],
  settings: [["Pose", selectValue("Aim_Offset")], ["Set", selectValue("Combat")], ["Blend", inputValue("0.25")], ["Mirror", checkValue(false)], ["Apply Additive", checkValue(true)]],
  primary: tablePrimary("Pose Library", ["Pose", "Set", "Tags", "State"], [["Aim_Offset", "Combat", "UpperBody", "Selected"], ["Idle_A", "Base", "Loop", "Ready"], ["Crouch_Start", "Movement", "Start", "Ready"], ["Deprecated_Pose", "Legacy", "Old", "Warning"]], "1fr 0.8fr 1fr 0.8fr")
});
