import { checkbox, input, select } from "../../../../components/inputs/atoms.js";

export const animationRecipe = {
  detailTabs: ["Tracks", "Curves", "Validation"],
  actions: (subject, shortLabel) => [["play", `Preview ${shortLabel}`], ["plus", `Add ${shortLabel}`], ["target", `Key ${shortLabel}`], ["check", `Validate ${shortLabel}`]],
  tools: (subject) => ["Pose Track", "Notify Track", "Blend Region", "Root Motion", `${subject} Curves`, "Sync Marker"],
  metrics: () => [["Frames", "240"], ["Tracks", "18"], ["Keys", "284"], ["Sync", "OK"]],
  settings: (subject) => [["Clip", select(`${subject}_Main`)], ["Frame Rate", select("60 fps")], ["Work Range", input("", { value: "0100-0240" })], ["Snap", checkbox("", true)], ["Auto Key", checkbox("", false)]],
  table: () => [["Base Pose", "0000-0060", "Ready"], ["Transition", "0060-0120", "Blending"], ["Notify Window", "0120-0160", "Selected"], ["Recovery", "0160-0240", "Ready"]]
};
