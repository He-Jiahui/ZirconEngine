import { checkbox, input, select } from "../../../../components/inputs/atoms.js";

export const defaultRecipe = {
  detailTabs: ["Details", "Rules", "Validation"],
  actions: (subject, shortLabel) => [["search", `Find ${shortLabel}`], ["plus", `Add ${shortLabel}`], ["check", `Validate ${shortLabel}`], ["play", `Preview ${shortLabel}`]],
  tools: (subject) => [`${subject} Overview`, `${subject} Assets`, `${subject} Rules`, `${subject} Preview`, `${subject} Validation`, `${subject} Output`],
  metrics: () => [["Refs", "1"], ["Controls", "24"], ["Panels", "4"], ["Status", "Ready"]],
  settings: (subject) => [["Preset", select(`${subject} Default`)], ["Filter", input("Filter...")], ["Live Preview", checkbox("", true)], ["Auto Validate", checkbox("", true)], ["Density", select("Workbench Compact")]],
  table: (subject) => [[`${subject}_Primary`, "Ready", "Panel"], [`${subject}_Secondary`, "Ready", "Details"], [`${subject}_Validation`, "Queued", "Check"], [`${subject}_Output`, "Idle", "Log"]]
};
