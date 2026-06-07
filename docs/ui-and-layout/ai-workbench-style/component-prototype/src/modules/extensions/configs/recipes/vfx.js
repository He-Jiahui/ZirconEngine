import { checkbox, input, select } from "../../../../components/inputs/atoms.js";

export const vfxRecipe = {
  detailTabs: ["Emitters", "Curves", "Compile"],
  actions: (subject, shortLabel) => [["play", `Simulate ${shortLabel}`], ["plus", `Add ${shortLabel}`], ["check", `Compile ${shortLabel}`], ["target", `Capture ${shortLabel}`]],
  tools: (subject) => ["Emitter Stack", "Spawn Rate", "GPU Sim", "Curve Track", `${subject} Preview`, "Bounds Debug"],
  metrics: () => [["Emitters", "5"], ["Particles", "42K"], ["GPU", "0.8 ms"], ["Warnings", "2", "warning"]],
  settings: (subject) => [["Emitter", select(subject)], ["FPS", select("60 fps")], ["Duration", input("", { value: "2.0" })], ["Loop", checkbox("", true)], ["Fixed Bounds", checkbox("", false)]],
  table: () => [["Spawn", "Ready", "120/s"], ["Velocity", "Ready", "Curve"], ["Color", "Selected", "Gradient"], ["GPU Sort", "Warning", "Cost"]]
};
