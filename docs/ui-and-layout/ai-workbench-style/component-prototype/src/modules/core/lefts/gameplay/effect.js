import { checkbox, input, searchInput, select, tabs } from "../../../../components/inputs/atoms.js";
import { listRows, moduleTree, panel, settingsRows } from "../../../shared/module-components.js";

export function gameplayLeft() {
  return [
    panel("Effect Tools", `${tabs(["Rules", "Stacking", "Tags"], 0)}${settingsRows([
      ["Duration Policy", select("Has Duration")],
      ["Duration", `${input("", { value: "10.0" })}<small>s</small>`],
      ["Period", `${input("", { value: "1.0" })}<small>s</small>`],
      ["Execute Periodic", checkbox("", false)],
      ["Stacking Type", select("Aggregate by Source")],
      ["Stack Limit Count", input("", { value: "5" })],
      ["Deny Overflow", checkbox("", false)]
    ])}`),
    panel("Tag Requirements", listRows(["Granted Tags", "Blocked Tags", "Source Tags", "Target Tags"], 3, ["1", "0", "0", "2"])),
    panel("Effect Assets", `${searchInput("Search assets...")}${moduleTree([
      ["Gameplay Effects", "folder", false, 0],
      ["GE_HealthRegen", "gear", true, 1],
      ["GE_DamageFire", "gear", false, 1],
      ["GE_Slow", "gear", false, 1],
      ["Curve Tables", "folder", false, 0],
      ["CT_Damage", "grid", false, 1]
    ])}`)
  ];
}
