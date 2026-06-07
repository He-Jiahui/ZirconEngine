import { blueprint, checkValue, graphPrimary, inputValue, selectValue, tree } from "../helpers.js";

export const controlRigBlueprint = blueprint({
  status: "Control rig solve graph selected",
  actions: [["play", "Preview Solve"], ["plus", "Add Control"], ["target", "Key Control"], ["check", "Validate Rig"]],
  tools: ["FK Control", "IK Chain", "Constraint", "Space Switch", "Pose Driver", "Solve Order"],
  assets: tree("Rig", "component", ["CR_Hero", "Spine_CTRL", "Hand_IK_L", "Foot_IK_R", "Space_World"]),
  metrics: [["Controls", "64"], ["Bones", "128"], ["Constraints", "18"], ["Warnings", "1", "warning"]],
  detailTabs: ["Controls", "Hierarchy", "Solve"],
  settings: [["Control", selectValue("Hand_IK_L")], ["Space", selectValue("World")], ["Weight", inputValue("1.0")], ["Mirror", checkValue(false)], ["Draw Axes", checkValue(true)]],
  primary: graphPrimary("Control Rig Solve Graph", [["Spine_CTRL", "FK", 12, 28, "cyan"], ["Arm_IK_L", "IK", 38, 18, "blue"], ["Hand_IK_L", "Selected", 60, 38, "green"], ["Foot_IK_R", "IK", 34, 66, "orange"], ["Output Pose", "Solve", 76, 56, "purple"]])
});
