import { cluster } from "../../../../foundation/layout.js";
import { input, select } from "../../../../components/inputs/atoms.js";
import { actionIcon, graphBoard, graphLink, node, panel, previewTile, timeline } from "../../../shared/module-components.js";

export function vfxCenter() {
  return `<div class="zr-module-editor-grid is-vfx">
    ${panel("Preview", `${previewTile("vfx")}${cluster({ className: "zr-module-playbar", children: [actionIcon("Play", "play"), actionIcon("Pause", "more"), actionIcon("Record", "target"), select("60 fps"), input("", { value: "00:01.23" })] })}`)}
    ${panel("Emitter Stack", graphBoard("vfx", [
      node("Spawn", "Rate / Burst", 16, 34, "green"),
      node("Update", "Force / Curl Noise", 44, 28, "blue"),
      node("Output", "Sprite Renderer", 72, 36, "cyan")
    ], `${graphLink(27, 40, 20, 0)}${graphLink(54, 40, 16, 0)}`))}
    ${panel("Timeline", timeline("vfx"))}
  </div>`;
}
