import { gameplayBottom } from "../../core-module-bottoms.js";
import { gameplayCenter } from "../../core-module-centers.js";
import { gameplayDetails } from "../../core-module-details.js";
import { gameplayLeft } from "../../core-module-lefts.js";
import { bottomOutput } from "../../../shared/module-components.js";

export const gameplayEffectCoreModule = {
  id: "gameplay-effect",
  label: "Gameplay Effect",
  shortLabel: "Effect",
  icon: "component",
  status: "GE_HealthRegen selected",
  actions: [
    ["save", "Save"],
    ["folder", "Browse"],
    ["check", "Compile"],
    ["history", "Diff"],
    ["play", "Simulation"]
  ],
  left: () => gameplayLeft(),
  center: () => gameplayCenter(),
  right: () => gameplayDetails(),
  bottom: () => bottomOutput("gameplay-effect", ["Simulation Output", "Attribute Delta", "Validation", "Compile Log"], gameplayBottom())
};
