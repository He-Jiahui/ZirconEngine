import { abilityBottom } from "../../core-module-bottoms.js";
import { abilityCenter } from "../../core-module-centers.js";
import { abilityDetails } from "../../core-module-details.js";
import { abilityLeft } from "../../core-module-lefts.js";
import { bottomOutput } from "../../../shared/module-components.js";

export const gameplayAbilityCoreModule = {
  id: "gameplay-ability",
  label: "Gameplay Ability",
  shortLabel: "Ability",
  icon: "play",
  status: "GA_DashAttack ability graph open",
  actions: [
    ["save", "Save"],
    ["check", "Compile Ability"],
    ["history", "Diff"],
    ["search", "Find"],
    ["play", "Playtest"]
  ],
  left: () => abilityLeft(),
  center: () => abilityCenter(),
  right: () => abilityDetails(),
  bottom: () => bottomOutput("gameplay-ability", ["Timeline", "Compile Log", "Gameplay Event Log", "Simulation Console"], abilityBottom())
};
