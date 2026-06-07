import { checkbox, input, searchInput, select, slider, toggle } from "../../../../components/inputs/atoms.js";
import { moduleTree, panelGroup, settingsRows } from "../../../shared/module-components.js";
import { coreRightRouteOptions } from "../routes.js";

export function gameplayDetails() {
  return panelGroup("gameplay-right", [
    { label: "Effect Hierarchy", active: true, content: `${searchInput("Search hierarchy...")}${moduleTree([
      ["GE_HealthRegen", "gear", true, 0],
      ["Modifiers (4)", "folder", false, 1],
      ["HealthRegen (Additive)", "component", true, 2],
      ["IncomingHealing", "component", false, 2],
      ["Executions (1)", "folder", false, 1],
      ["Clamp Health", "play", false, 2],
      ["Granted Tags", "folder", false, 1],
      ["Regen.Health", "target", false, 2]
    ], coreRightRouteOptions("gameplay-right:effect-hierarchy"))}` },
    { label: "Details", content: `${settingsRows([
      ["Attribute", select("Health")],
      ["Modifier Op", select("Additive")],
      ["Magnitude Type", select("Scalable Float")],
      ["Magnitude", input("", { value: "10.0" })],
      ["Snapshot", checkbox("", false)],
      ["Replicate", toggle("", true)]
    ])}${slider("Magnitude Curve", 72, "10.0")}` }
  ]);
}
