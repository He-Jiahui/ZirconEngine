import { checkbox, input, select, slider } from "../../../../components/inputs/atoms.js";
import { alerts } from "../../../../components/data/collections.js";
import { listRows, moduleTree, panelGroup, settingsRows } from "../../../shared/module-components.js";
import { coreRightRouteOptions } from "../routes.js";

export function vfxDetails() {
  return panelGroup("vfx-right", [
    { label: "System Overview", active: true, content: `${moduleTree([
      ["P_Bolt_01", "sun", true, 0],
      ["E_Bolt", "component", true, 1],
      ["E_Bolt_Light", "sun", false, 1],
      ["E_Bolt_Sparks", "sun", false, 1]
    ], coreRightRouteOptions("vfx-right:system-overview"))}${listRows(["Spawn", "Update", "Post Update", "Render"], 1, ["10", "22", "6", "5"], coreRightRouteOptions("vfx-right:system-overview"))}` },
    { label: "Stages", content: listRows(["Stage 0 Spawn", "Stage 1 Update", "Stage 2 Post Update", "Stage 3 Render"], 1, [], coreRightRouteOptions("vfx-right:stages")) },
    { label: "Details", content: `${settingsRows([
      ["Curl Noise", checkbox("", true)],
      ["Noise Strength", input("", { value: "75.0" })],
      ["Frequency", input("", { value: "2.5" })],
      ["Octaves", select("3")],
      ["Noise Type", select("Curl")],
      ["Space", select("World")]
    ])}${slider("Mask", 68, "None")}` },
    { label: "Compile", content: alerts([["success", "E_Bolt compile success"], ["warning", "Warnings (2)"], ["info", "Infos (3)"]]) }
  ]);
}
