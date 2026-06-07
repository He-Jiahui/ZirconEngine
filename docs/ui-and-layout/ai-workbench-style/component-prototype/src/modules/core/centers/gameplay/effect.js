import { cluster, grid } from "../../../../foundation/layout.js";
import { select } from "../../../../components/inputs/atoms.js";
import {
  actionButton,
  curvePanel,
  graphBoard,
  graphLink,
  moduleTable,
  node,
  panel,
  settingsRows,
  tag
} from "../../../shared/module-components.js";

export function gameplayCenter() {
  return grid({ className: "zr-module-editor-grid is-gameplay", children: [
    panel("Modifiers", `${cluster({ className: "zr-module-card-tools", justify: "end", children: [actionButton("Add Modifier", "plus"), actionButton("Duplicate", "file"), actionButton("Delete", "trash", { kind: "danger" })] })}${moduleTable(["#", "Name", "Attribute", "Modifier Op", "Magnitude", "Source", "Tags"], [
      { cells: ["1", "HealthRegen", "Health", "Additive", "Scalable Float", "Source", tag("Regen.Health", "cyan")], selected: true },
      { cells: ["2", "IncomingHealing", "Healing Received", "Multiplicative", "Scalable Float", "Target", tag("Regen.Bonus", "green")] },
      { cells: ["3", "MaxHealthCap", "Max Health", "Additive", "Captured Attribute", "Source", tag("Regen.Cap", "blue")] },
      { cells: ["4", "RegenPerStack", "Health", "Additive", "Scalable Float", "Source", tag("Regen.Stack", "orange")] }
    ], "38px 1.2fr 1.1fr 1.1fr 1fr 0.8fr 1fr")}`),
    panel("Dependency Graph", graphBoard("dependency", [
      node("HealthRegen", "Modifier", 44, 12, "cyan"),
      node("RegenPerStack", "Modifier", 70, 20, "cyan"),
      node("Health", "Attribute", 46, 42, "blue"),
      node("Max Health", "Attribute", 72, 42, "blue"),
      node("Clamp Health", "Execution", 47, 72, "orange"),
      node("Target Tags", "Require", 18, 72, "green"),
      node("Blocked Tags", "Block", 74, 72, "green")
    ], `${graphLink(52, 22, 22, 8)}${graphLink(50, 38, 16, -18)}${graphLink(52, 58, 18, 90)}${graphLink(32, 80, 18, 0)}${graphLink(60, 80, 18, 0)}`)),
    panel("Attribute Preview", `${settingsRows([["Preview at Level", select("1")]])}${curvePanel()}`)
  ] });
}
