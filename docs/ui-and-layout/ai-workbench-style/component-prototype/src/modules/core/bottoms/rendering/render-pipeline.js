import { alerts } from "../../../../components/data/collections.js";
import { input, select } from "../../../../components/inputs/atoms.js";
import { moduleTable, settingsRows, tag } from "../../../shared/module-components.js";
import { coreBottomRouteOptions } from "../routes.js";

export function renderPipelineBottom() {
  const routeOptions = coreBottomRouteOptions("render-pipeline", "frame-capture-log");
  return `<div class="zr-module-output-grid">
    ${settingsRows([["Frame", input("", { value: "1234" })], ["Platform", select("Windows DX12")], ["FPS", select("30 fps")]])}
    ${moduleTable(["Event", "Pass", "Description", "GPU ms"], [
      { cells: [tag("Info", "blue"), "Frame Start", "Frame 1234 captured", "0.000"] },
      { cells: [tag("OK", "green"), "Lighting Pass", "2 Lighting Pass", "1.872"] },
      { cells: [tag("OK", "green"), "Post Process Pass", "5 Post Process Pass", "0.450"], selected: true },
      { cells: [tag("OK", "green"), "UI Composite Pass", "7 UI Composite Pass", "0.184"] }
    ], "82px 1fr 1.8fr 90px", routeOptions)}
    ${alerts([["success", "Pipeline compile succeeded"], ["warning", "3 resource transition warnings"], ["info", "0 errors"]])}
  </div>`;
}
