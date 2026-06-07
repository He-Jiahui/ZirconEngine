import { checkbox, input, select } from "../../../components/inputs/atoms.js";
import { actionPath } from "../../../foundation/action-paths.js";
import { compactStats, panel, settingsRows } from "../../shared/module-components.js";

export function hudCenter() {
  return `<div class="zr-module-editor-grid is-hud">
    ${panel("Gameplay_HUD Canvas", hudCanvas())}
    ${panel("Responsive Layout", `${settingsRows([
      ["Device", select("iPhone 15 Pro")],
      ["Aspect", select("19.5:9")],
      ["DPI Scale", input("", { value: "1.00" })],
      ["Safe Zone", checkbox("", true)]
    ])}${compactStats([["Widgets", "18"], ["Bindings", "12"], ["Warnings", "3", "warning"], ["Errors", "1", "warning"]])}`)}
  </div>`;
}

function hudCanvas() {
  return `<div class="zr-module-hud-canvas">
    <button class="zr-hud-widget is-minimap" type="button" data-action="${actionPath("workbench.module.hud.widget", "minimap")}">Minimap</button>
    <button class="zr-hud-widget is-score-left" type="button" data-action="${actionPath("workbench.module.hud.score", "team_score_left")}">12</button>
    <button class="zr-hud-widget is-score-right" type="button" data-action="${actionPath("workbench.module.hud.score", "team_score_right")}">08</button>
    <button class="zr-hud-widget is-weapon is-selected" type="button" data-action="${actionPath("workbench.module.hud.widget", "weapon_panel")}"><strong>30</strong><small>/120</small></button>
    <button class="zr-hud-widget is-status" type="button" data-action="${actionPath("workbench.module.hud.widget", "player_status")}"><strong>Ranger_7</strong><span></span></button>
    <button class="zr-hud-widget is-ability a1" type="button" data-action="${actionPath("workbench.module.hud.ability", "slot_1")}">Q</button>
    <button class="zr-hud-widget is-ability a2" type="button" data-action="${actionPath("workbench.module.hud.ability", "slot_2")}">E</button>
    <button class="zr-hud-widget is-ability a3" type="button" data-action="${actionPath("workbench.module.hud.ability", "slot_3")}">R</button>
    <button class="zr-hud-widget is-action b1" type="button" data-action="${actionPath("workbench.module.hud.action", "sprint")}">Run</button>
    <button class="zr-hud-widget is-action b2" type="button" data-action="${actionPath("workbench.module.hud.action", "crosshair")}">Aim</button>
    <span class="zr-hud-crosshair"></span>
  </div>`;
}
