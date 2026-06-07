import { perceptionBottom } from "../../core-module-bottoms.js";
import { perceptionCenter } from "../../core-module-centers.js";
import { perceptionDetails } from "../../core-module-details.js";
import { perceptionLeft } from "../../core-module-lefts.js";
import { bottomOutput } from "../../../shared/module-components.js";

export const aiPerceptionCoreModule = {
  id: "ai-perception",
  label: "AI Perception",
  shortLabel: "Perception",
  icon: "eye",
  status: "Guard_Perception drawing sight and hearing stimuli",
  actions: [
    ["play", "Simulate Perception"],
    ["target", "Focus"],
    ["grid", "2D View"],
    ["cube", "3D View"],
    ["check", "Validate Query"]
  ],
  left: () => perceptionLeft(),
  center: () => perceptionCenter(),
  right: () => perceptionDetails(),
  bottom: () => bottomOutput("ai-perception", ["Perception Timeline", "Debug Log", "Query Output", "Validation", "Compile Log"], perceptionBottom())
};
