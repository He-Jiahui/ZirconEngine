import { checkbox, select } from "../../../../components/inputs/atoms.js";

export const uiRecipe = {
  detailTabs: ["Hierarchy", "Bindings", "Accessibility"],
  actions: (subject, shortLabel) => [["plus", `Add ${shortLabel}`], ["play", `Preview ${shortLabel}`], ["check", `Validate ${shortLabel}`], ["save", `Export ${shortLabel}`]],
  tools: (subject) => ["Widget Tree", "Responsive Rules", "Binding Graph", "Token Swatches", `${subject} Preview`, "Accessibility Audit"],
  metrics: () => [["Widgets", "42"], ["Bindings", "18"], ["Breakpoints", "4"], ["Issues", "3", "warning"]],
  settings: (subject) => [["Screen", select(`${subject} Screen`)], ["Breakpoint", select("Desktop")], ["Theme", select("Workbench Dark")], ["Show Bounds", checkbox("", true)], ["Auto Layout", checkbox("", true)]],
  table: () => [["Header", "Container", "Bound"], ["Primary Button", "Action", "Ready"], ["Status Text", "Text", "Warning"], ["Icon Grid", "List", "Ready"]]
};
