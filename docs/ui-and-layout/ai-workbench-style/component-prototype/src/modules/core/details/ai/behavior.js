import { checkbox, searchInput } from "../../../../components/inputs/atoms.js";
import { listRows, moduleTree, panelGroup, settingsRows, tag } from "../../../shared/module-components.js";
import { coreRightRouteOptions } from "../routes.js";

export function behaviorDetails() {
  return panelGroup("behavior-right", [
    { label: "BT Outline", content: `${searchInput("Search...")}${moduleTree([
      ["ROOT", "target", false, 0],
      ["Selector", "component", true, 1],
      ["Sequence", "list", false, 2],
      ["Chase Target", "target", false, 3],
      ["Attack", "play", true, 3]
    ], coreRightRouteOptions("behavior-right:bt-outline"))}` },
    { label: "Execution", active: true, content: `${listRows(["Selector - Running", "Sequence - Running", "Chase Target - Success", "Attack - Running", "Patrol - Inactive"], 3, [], coreRightRouteOptions("behavior-right:execution"))}${settingsRows([["Status", tag("Running", "cyan")], ["Elapsed", "1.45s"], ["Last Result", "In Progress"]])}` },
    { label: "Blackboard", content: settingsRows([["TargetActor", "Player_01"], ["LastKnownLocation", "128, 64, -12"], ["CanAttack", checkbox("", true)]]) }
  ]);
}
