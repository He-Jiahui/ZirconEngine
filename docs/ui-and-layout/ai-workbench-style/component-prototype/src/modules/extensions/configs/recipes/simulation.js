import { checkbox, input, select } from "../../../../components/inputs/atoms.js";

export const simulationRecipe = {
  detailTabs: ["Bodies", "Materials", "Contacts"],
  actions: (subject, shortLabel) => [["plus", `Add ${shortLabel}`], ["check", `Validate ${shortLabel}`], ["grid", `Bake ${shortLabel}`], ["play", `Run ${shortLabel}`]],
  tools: (subject) => ["Body Setup", "Proxy Hull", "Material Pair", "Contact Debug", `${subject} Bake`, "Mass Preview"],
  metrics: () => [["Bodies", "12"], ["Hull Verts", "96"], ["Mass", "48 kg"], ["Errors", "1", "warning"]],
  settings: (subject) => [["Preset", select(subject)], ["Mass", input("", { value: "48.0" })], ["Friction", input("", { value: "0.62" })], ["Hit Events", checkbox("", true)], ["CCD", checkbox("", false)]],
  table: () => [["Hull_00", "Convex", "32 verts"], ["Hull_01", "Box", "8 verts"], ["Hull_02", "Convex", "56 verts"], ["Hull_03", "Invalid", "Non-manifold"]]
};
