import { checkbox, input, select } from "../../../../components/inputs/atoms.js";

export const productionRecipe = {
  detailTabs: ["Queue", "Rules", "History"],
  actions: (subject, shortLabel) => [["play", `Run ${shortLabel}`], ["check", `Validate ${shortLabel}`], ["save", `Publish ${shortLabel}`], ["history", `Review ${shortLabel}`]],
  tools: (subject) => ["Queue", "Validation Gate", "Artifact Set", "Change List", `${subject} Rules`, "Report Output"],
  metrics: () => [["Jobs", "12"], ["Queued", "4"], ["Warnings", "2", "warning"], ["Ready", "8"]],
  settings: (subject) => [["Profile", select(`${subject} Default`)], ["Target", select("Windows")], ["Version", input("", { value: "2026.06" })], ["Strict Mode", checkbox("", true)], ["Archive Output", checkbox("", true)]],
  table: (subject) => [[`${subject}_Validate`, "Running", "62"], [`${subject}_Cook`, "Queued", "0"], [`${subject}_Package`, "Ready", "100"], [`${subject}_Report`, "Queued", "0"]]
};
