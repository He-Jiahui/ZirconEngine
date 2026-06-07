import { input, select } from "../../../../components/inputs/atoms.js";
import { moduleTable, settingsRows, tag, timeline } from "../../../shared/module-components.js";
import { coreBottomRouteOptions } from "../routes.js";

export function abilityBottom() {
  const routeOptions = coreBottomRouteOptions("gameplay-ability", "timeline");
  return `<div class="zr-module-output-grid">
    ${settingsRows([["Speed", select("1.0x")], ["Duration", input("", { value: "4.00s" })], ["Playhead", input("", { value: "1.25s" })]])}
    ${timeline("ability")}
    ${moduleTable(["Time", "Event", "Result", "Asset"], [
      { cells: ["1.22s", "Ability Activated", tag("OK", "green"), "GA_DashAttack"], selected: true },
      { cells: ["1.30s", "Cost Applied", tag("OK", "green"), "GE_DashAttack_Cost"] },
      { cells: ["2.45s", "Gameplay Event", tag("Received", "cyan"), "Event.Data.Hit"] },
      { cells: ["2.48s", "Effect Applied", tag("OK", "green"), "GE_DashAttack_Damage"] }
    ], "70px 1.2fr 92px 1.4fr", routeOptions)}
  </div>`;
}
