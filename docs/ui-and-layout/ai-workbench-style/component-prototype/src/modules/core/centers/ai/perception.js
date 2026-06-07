import { input, select, slider } from "../../../../components/inputs/atoms.js";
import { actionPath } from "../../../../foundation/action-paths.js";
import { compactStats, panel, settingsRows } from "../../../shared/module-components.js";

export function perceptionCenter() {
  return `<div class="zr-module-editor-grid is-perception">
    ${panel("World Perception Map", perceptionMap())}
    ${panel("Stimuli Channels", compactStats([["Agents", "3"], ["Actors", "4"], ["Events", "6"], ["Lost Sight", "2", "warning"]]))}
    ${panel("Sense Profile", `${settingsRows([
      ["Preset", select("Guard Balanced")],
      ["Sight Radius", input("", { value: "2000.0" })],
      ["Lose Radius", input("", { value: "2500.0" })],
      ["Peripheral Angle", input("", { value: "120deg" })]
    ])}${slider("Max Age", 52, "5.0s")}`)}
  </div>`;
}

function perceptionMap() {
  return `<div class="zr-module-map is-perception-map">
    <span class="zr-map-wall is-1"></span><span class="zr-map-wall is-2"></span><span class="zr-map-wall is-3"></span><span class="zr-map-wall is-4"></span>
    <button class="zr-map-point is-agent is-1" type="button" data-action="${actionPath("workbench.module.perception.agent", "ai_guard_01")}"><span>AI_Guard_01</span></button>
    <button class="zr-map-point is-agent is-2" type="button" data-action="${actionPath("workbench.module.perception.agent", "ai_guard_02")}"><span>AI_Guard_02</span></button>
    <button class="zr-map-point is-agent is-3" type="button" data-action="${actionPath("workbench.module.perception.agent", "ai_guard_03")}"><span>AI_Guard_03</span></button>
    <button class="zr-map-point is-hostile" type="button" data-action="${actionPath("workbench.module.perception.hostile", "enemy_01")}"><span>Enemy_01</span></button>
    <button class="zr-map-point is-sound" type="button" data-action="${actionPath("workbench.module.perception.sound", "noise_maker_bp")}"><span>Noise_Maker_BP</span></button>
    <span class="zr-map-cone is-cyan is-1"></span><span class="zr-map-cone is-cyan is-2"></span><span class="zr-map-cone is-purple"></span>
    <span class="zr-map-path is-1"></span><span class="zr-map-path is-2"></span><span class="zr-map-path is-3"></span>
  </div>`;
}
