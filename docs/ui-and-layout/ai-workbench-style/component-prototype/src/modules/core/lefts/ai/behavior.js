import { searchInput } from "../../../../components/inputs/atoms.js";
import { listRows, moduleTree, panel } from "../../../shared/module-components.js";

export function behaviorLeft() {
  return [
    panel("Node Palette", `${searchInput("Search nodes...")}${listRows(["Selector", "Sequence", "Parallel", "Blackboard", "Cooldown", "Distance", "Attack", "Wait"], 0)}`),
    panel("AI Assets", moduleTree([
      ["Blackboards", "folder", false, 0],
      ["BB_Enemy", "grid", true, 1],
      ["Behavior Trees", "folder", false, 0],
      ["BT_Enemy", "component", true, 1],
      ["EQS", "folder", false, 0],
      ["EQS_Enemy_Search", "target", false, 1]
    ]))
  ];
}
