import { blueprint, checkValue, graphPrimary, inputValue, selectValue, tree } from "../helpers.js";

export const motionMatchingBlueprint = blueprint({
  status: "Motion matching query and pose cluster active",
  actions: [["play", "Preview Match"], ["target", "Query Pose"], ["check", "Validate Database"], ["history", "Review Match"]],
  tools: ["Pose Query", "Trajectory", "Feature Vector", "Pose Cluster", "Cost Debug", "Database"],
  assets: tree("Motion", "play", ["MM_Locomotion", "Pose_Run_42", "Pose_Stop_08", "Trajectory_A", "Cluster_Turn"]),
  metrics: [["Poses", "12K"], ["Clusters", "86"], ["Cost", "0.14"], ["Warnings", "1", "warning"]],
  detailTabs: ["Query", "Clusters", "Timeline"],
  settings: [["Database", selectValue("MM_Locomotion")], ["Trajectory", selectValue("2D Future")], ["Cost Bias", inputValue("0.42")], ["Mirror", checkValue(true)], ["Debug Draw", checkValue(true)]],
  primary: graphPrimary("Motion Matching Query", [["Current Pose", "Input", 12, 44, "cyan"], ["Trajectory", "Feature", 34, 22, "blue"], ["Pose Cluster", "Search", 58, 34, "green"], ["Best Match", "Pose", 76, 52, "orange"], ["Cost Curve", "Debug", 42, 68, "purple"]])
});
