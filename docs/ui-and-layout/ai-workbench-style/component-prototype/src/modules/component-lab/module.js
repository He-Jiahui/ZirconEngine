import { bottomOutput } from "../shared/module-components.js";
import { componentLabBottom } from "./bottom.js";
import { componentLabCenter } from "./center.js";
import { componentLabDetails } from "./details.js";
import { componentLabLeft } from "./left.js";

export const componentLabModule = {
  id: "component-lab",
  label: "Component Lab",
  shortLabel: "Components",
  icon: "component",
  status: "Bottom-up component taxonomy audit",
  webOnly: true,
  actions: [
    ["check", "Audit Inputs"],
    ["list", "Audit Collections"],
    ["columns", "Audit Surfaces"],
    ["grid", "Responsive"],
    ["component", "Native Handoff"]
  ],
  left: () => componentLabLeft(),
  center: () => componentLabCenter(),
  right: () => componentLabDetails(),
  bottom: () => bottomOutput("component-lab", ["Audit Log", "Responsive", "Routes", "Native Handoff"], componentLabBottom())
};
