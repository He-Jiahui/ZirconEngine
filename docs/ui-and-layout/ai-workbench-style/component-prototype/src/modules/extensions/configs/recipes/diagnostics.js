import { checkbox, input, select } from "../../../../components/inputs/atoms.js";

export const diagnosticsRecipe = {
  detailTabs: ["Live Log", "Counters", "Report"],
  actions: (subject, shortLabel) => [["search", `Filter ${shortLabel}`], ["trash", `Clear ${shortLabel}`], ["save", `Export ${shortLabel}`], ["check", `Open ${shortLabel}`]],
  tools: (subject) => ["Log Filter", "Counters", "Trace Events", "Warning Buckets", `${subject} Report`, "Session Diff"],
  metrics: () => [["FPS", "58"], ["Warnings", "24", "warning"], ["Errors", "1", "warning"], ["Marks", "82"]],
  settings: (subject) => [["Subsystem", select(subject)], ["Severity", select("Warnings+")], ["Regex", input("filter...")], ["Collapse Repeats", checkbox("", true)], ["Follow Tail", checkbox("", true)]],
  table: () => [["12:10:11", "Renderer", "Warning"], ["12:10:13", "Asset", "Info"], ["12:10:18", "Gameplay", "Warning"], ["12:10:21", "Runtime", "Error"]]
};
