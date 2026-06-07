import { blueprint, checkValue, graphPrimary, selectValue, tree } from "../helpers.js";

export const blendSpaceBlueprint = blueprint({
  status: "Blend samples and preview point selected",
  actions: [["play", "Preview Blend"], ["plus", "Add Sample"], ["target", "Move Sample"], ["check", "Validate Blend"]],
  tools: ["Axis Setup", "Sample Point", "Preview Cursor", "Triangulation", "Sync Group", "Curve Overlay"],
  assets: tree("Blend Spaces", "play", ["BS_Locomotion", "Walk_Fwd", "Run_Fwd", "Strafe_Left", "Sample_Grid"]),
  metrics: [["Samples", "12"], ["Axes", "2"], ["Warnings", "1", "warning"], ["Sync", "OK"]],
  detailTabs: ["Samples", "Axes", "Preview"],
  settings: [["Blend Space", selectValue("BS_Locomotion")], ["X Axis", selectValue("Speed")], ["Y Axis", selectValue("Direction")], ["Snap Samples", checkValue(true)], ["Show Triangles", checkValue(true)]],
  primary: graphPrimary("Blend Sample Map", [["Walk", "Speed 150", 14, 70, "green"], ["Jog", "Speed 320", 38, 48, "cyan"], ["Run", "Speed 600", 68, 28, "blue"], ["Strafe", "Dir -90", 32, 78, "orange"], ["Preview", "Cursor", 52, 56, "purple"]])
});
