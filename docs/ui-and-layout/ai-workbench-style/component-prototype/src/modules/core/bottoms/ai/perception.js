import { checkbox, select } from "../../../../components/inputs/atoms.js";
import { moduleTable, settingsRows, tag, timeline } from "../../../shared/module-components.js";
import { coreBottomRouteOptions } from "../routes.js";

export function perceptionBottom() {
  const routeOptions = coreBottomRouteOptions("ai-perception", "perception-timeline");
  return `<div class="zr-module-output-grid">
    ${settingsRows([["Agents", select("All Agents")], ["Show Lost", checkbox("", true)], ["Speed", select("1.0x")]])}
    ${timeline("perception")}
    ${moduleTable(["Time", "Agent", "Event", "Sense"], [
      { cells: ["00:11.8", "AI_Guard_01", "Hearing stimulus", tag("Hearing", "purple")] },
      { cells: ["00:13.1", "AI_Guard_02", "Enemy_01 seen", tag("Sight", "cyan")], selected: true },
      { cells: ["00:14.0", "AI_Guard_02", "Lost sight", tag("Warning", "orange")] }
    ], "80px 1fr 1.4fr 92px", routeOptions)}
  </div>`;
}
