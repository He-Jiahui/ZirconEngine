import { alerts } from "../../../../components/data/collections.js";
import { input, select } from "../../../../components/inputs/atoms.js";
import { moduleTable, settingsRows } from "../../../shared/module-components.js";
import { coreBottomRouteOptions } from "../routes.js";

export function gameplayBottom() {
  const routeOptions = coreBottomRouteOptions("gameplay-effect", "simulation-output");
  return `<div class="zr-module-output-grid">
    ${settingsRows([["Instigator", select("Player_01")], ["Target", select("Player_01")], ["Level", input("", { value: "1" })], ["Duration", input("", { value: "10.0" })]])}
    ${moduleTable(["Time", "Event", "Attribute", "Base", "Delta", "Final", "Source"], [
      { cells: ["0.00", "Apply GE", "Health", "100.00", "+0.00", "100.00", "Player_01"] },
      { cells: ["1.00", "Periodic Exec", "Health", "100.00", "+10.00", "110.00", "Player_01"], selected: true },
      { cells: ["2.00", "Periodic Exec", "Health", "110.00", "+10.00", "120.00", "Player_01"] }
    ], "70px 1.1fr 90px 70px 70px 70px 1fr", routeOptions)}
    ${alerts([["success", "All Good"], ["warning", "0 Warnings"], ["info", "Compile successful"]])}
  </div>`;
}
