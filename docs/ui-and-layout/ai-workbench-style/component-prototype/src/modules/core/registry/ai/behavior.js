import { behaviorBottom } from "../../core-module-bottoms.js";
import { behaviorCenter } from "../../core-module-centers.js";
import { behaviorDetails } from "../../core-module-details.js";
import { behaviorLeft } from "../../core-module-lefts.js";
import { bottomOutput } from "../../../shared/module-components.js";

export const behaviorTreeCoreModule = {
  id: "behavior-tree",
  label: "Behavior Tree",
  shortLabel: "Behavior",
  icon: "component",
  status: "BT_Enemy running in preview",
  actions: [
    ["save", "Save"],
    ["undo", "Undo"],
    ["play", "Play"],
    ["target", "Debug"],
    ["check", "Validate"]
  ],
  left: () => behaviorLeft(),
  center: () => behaviorCenter(),
  right: () => behaviorDetails(),
  bottom: () => bottomOutput("behavior-tree", ["AI Debug Log", "Runtime Trace", "Breakpoint Output", "Validation Issues"], behaviorBottom())
};
