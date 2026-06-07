import { checkbox, input, select } from "../../../../components/inputs/atoms.js";

export const renderingRecipe = {
  detailTabs: ["Passes", "Resources", "Issues"],
  actions: (subject, shortLabel) => [["play", `Preview ${shortLabel}`], ["check", `Compile ${shortLabel}`], ["target", `Capture ${shortLabel}`], ["save", `Save ${shortLabel}`]],
  tools: (subject) => [`${subject} Stack`, "Shader Pass", "Frame Capture", "Resource View", "Permutation Set", "Warnings"],
  metrics: () => [["GPU", "1.28 ms"], ["Passes", "9"], ["Textures", "24"], ["Warnings", "3", "warning"]],
  settings: (subject) => [["Preview", select("SM5")], ["Quality", select("High")], ["Frame", input("", { value: "1234" })], ["Capture Resources", checkbox("", true)], ["Live Compile", checkbox("", true)]],
  table: (subject) => [["GBuffer", "Ready", "0.42 ms"], ["Lighting", "Ready", "0.68 ms"], [subject, "Compiling", "0.18 ms"], ["Post Process", "Warning", "0.31 ms"]]
};
