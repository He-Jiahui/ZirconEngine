import { checkbox, input, select } from "../../../../components/inputs/atoms.js";

export const dataRecipe = {
  detailTabs: ["Rows", "Schema", "Validation"],
  actions: (subject, shortLabel) => [["plus", `Add ${shortLabel}`], ["folder", `Import ${shortLabel}`], ["check", `Validate ${shortLabel}`], ["save", `Save ${shortLabel}`]],
  tools: (subject) => ["Schema", "CSV Import", "Diff Rows", "Validation", `${subject} References`, "Bulk Edit"],
  metrics: () => [["Rows", "128"], ["Columns", "14"], ["Invalid", "2", "warning"], ["Refs", "512"]],
  settings: (subject) => [["Row Name", input("", { value: `${subject}_Primary` })], ["Type", select("Gameplay")], ["Version", input("", { value: "12" })], ["Localized", checkbox("", true)], ["Deprecated", checkbox("", false)]],
  table: (subject) => [[`${subject}_Tier01`, "Ready", "42"], [`${subject}_Tier02`, "Selected", "68"], [`${subject}_Fallback`, "Warning", "25"], [`${subject}_Debug`, "Ready", "58"]]
};
