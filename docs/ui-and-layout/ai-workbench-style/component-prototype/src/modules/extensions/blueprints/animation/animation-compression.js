import { blueprint, checkValue, inputValue, selectValue, timelinePrimary, tree } from "../helpers.js";

export const animationCompressionBlueprint = blueprint({
  status: "Animation compression batch compared",
  actions: [["check", "Compress Batch"], ["target", "Compare Error"], ["history", "Review Curves"], ["save", "Save Profile"]],
  tools: ["Compression Preset", "Error Metric", "Track Filter", "Memory Report", "Curve Trim", "Batch Queue"],
  assets: tree("Animation", "history", ["CMP_Humanoid", "Run_Fwd", "Jump_Land", "Curve_Facial", "Batch_Player"]),
  metrics: [["Clips", "38"], ["Saved", "42 MB"], ["Error", "0.18"], ["Warnings", "2", "warning"]],
  detailTabs: ["Tracks", "Error", "Memory"],
  settings: [["Profile", selectValue("Humanoid High")], ["Max Error", inputValue("0.18")], ["Key Reduction", selectValue("Adaptive")], ["Preserve Curves", checkValue(true)], ["Batch Mode", checkValue(true)]],
  primary: timelinePrimary("Compression Error Timeline", ["Clip", "Error", "Memory"], [["Run_Fwd", "0.12", "1.8 MB"], ["Jump_Land", "0.18", "1.2 MB"], ["Turn_90", "0.09", "0.9 MB"], ["Facial_A", "0.24", "Warning"]])
});
