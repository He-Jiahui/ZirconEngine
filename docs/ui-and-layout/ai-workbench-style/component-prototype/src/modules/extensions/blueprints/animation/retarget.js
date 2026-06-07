import { blueprint, checkValue, selectValue, tablePrimary, tree } from "../helpers.js";

export const retargetBlueprint = blueprint({
  status: "Retarget chain mapping selected",
  actions: [["target", "Map Chain"], ["play", "Preview Retarget"], ["check", "Validate Skeletons"], ["save", "Export Retarget"]],
  tools: ["Chain Map", "Source Pose", "Target Pose", "Root Scale", "IK Goal", "Export Queue"],
  assets: tree("Retarget", "target", ["RTG_HeroToNPC", "SK_Hero", "SK_NPC", "Chain_Arm_L", "Pose_A"]),
  metrics: [["Chains", "18"], ["Mapped", "17"], ["Errors", "1", "warning"], ["Clips", "42"]],
  detailTabs: ["Chains", "Pose", "Export"],
  settings: [["Rig", selectValue("HeroToNPC")], ["Source", selectValue("SK_Hero")], ["Target", selectValue("SK_NPC")], ["Retarget Root", checkValue(true)], ["Preview Motion", checkValue(true)]],
  primary: tablePrimary("Retarget Chain Map", ["Source", "Target", "Mode", "State"], [["Arm_L", "Arm_L", "FK/IK", "Selected"], ["Arm_R", "Arm_R", "FK/IK", "Ready"], ["Spine", "Spine", "Root", "Ready"], ["Tail", "-", "Missing", "Warning"]], "1fr 1fr 0.8fr 0.8fr")
});
