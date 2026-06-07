import { checkbox, input, select } from "../../../components/inputs/atoms.js";
import { actionButton, listRows, panelGroup, settingsRows } from "../../shared/module-components.js";
import { coreRightRouteOptions } from "./routes.js";

export function sceneDetails() {
  return panelGroup("scene-right", [
    { label: "Inspector", active: true, content: `${settingsRows([
      ["Object", select("Props")],
      ["Tag", select("Untagged")],
      ["Position", input("", { value: "128.4, 64.2, -32.7" })],
      ["Rotation", input("", { value: "0, 90, 0" })],
      ["Scale", input("", { value: "1, 1, 1" })],
      ["Static", checkbox("", false)]
    ])}${actionButton("Add Component", "plus", coreRightRouteOptions("scene-right:inspector"))}` },
    { label: "History", content: listRows(["Selected Props", "Moved Box_01", "Updated Material", "Saved Scene"], 0, [], coreRightRouteOptions("scene-right:history")) }
  ]);
}
