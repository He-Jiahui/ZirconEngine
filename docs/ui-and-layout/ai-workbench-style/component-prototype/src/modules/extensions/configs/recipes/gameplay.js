import { checkbox, input, select } from "../../../../components/inputs/atoms.js";

export const gameplayRecipe = {
  detailTabs: ["Rules", "State", "Validation"],
  actions: (subject, shortLabel) => [["plus", `Add ${shortLabel}`], ["play", `Simulate ${shortLabel}`], ["target", `Inspect ${shortLabel}`], ["check", `Validate ${shortLabel}`]],
  tools: (subject) => ["Rule Stack", "State Graph", "Tag Filters", "Spawn Probe", `${subject} Preview`, "Conflict Check"],
  metrics: () => [["Rules", "18"], ["States", "12"], ["Refs", "36"], ["Conflicts", "1", "warning"]],
  settings: (subject) => [["Rule Set", select(subject)], ["Authority", select("Server")], ["Seed", input("", { value: "2026" })], ["Live Preview", checkbox("", true)], ["Strict Tags", checkbox("", true)]],
  table: (subject) => [[`${subject}_Rule_A`, "Ready", "High"], [`${subject}_Rule_B`, "Selected", "Medium"], [`${subject}_State_C`, "Queued", "Low"], [`${subject}_Conflict`, "Warning", "Tags"]]
};
