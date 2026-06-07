import {
  graphBoard,
  graphLink,
  moduleTable,
  node,
  panel,
  tag,
  timeline
} from "../../../shared/module-components.js";

export function abilityCenter() {
  return `<div class="zr-module-editor-grid is-ability">
    ${panel("Ability Graph", graphBoard("ability", [
      node("Activate Ability", "Authority", 8, 34, "green"),
      node("Apply Cost", "Cost Effect", 25, 24, "orange"),
      node("Apply Cooldown", "Cooldown Effect", 25, 56, "orange"),
      node("Play Montage", "AM_DashAttack", 44, 35, "cyan"),
      node("Wait Gameplay Event", "Event.Data.Hit", 64, 24, "purple"),
      node("Apply Effect", "GE_DashAttack_Damage", 64, 58, "purple"),
      node("End Ability", "Success", 84, 35, "neutral")
    ], `${graphLink(19, 42, 10)}${graphLink(36, 32, 9)}${graphLink(36, 64, 11, -12)}${graphLink(56, 42, 10)}${graphLink(70, 42, 10, 90)}${graphLink(76, 66, 8, -28)}`))}
    ${panel("Ability Phase Matrix", moduleTable(["Phase", "Task", "Asset", "Net Role", "Status"], [
      { cells: ["Activation", "Activate Ability", "Authority Gate", "Server", tag("Ready", "green")] },
      { cells: ["Tasks", "Play Montage", "AM_DashAttack_Montage", "Predicted", tag("Selected", "cyan")], selected: true },
      { cells: ["Event Response", "Apply Effect", "GE_DashAttack_Damage", "Server", tag("Valid", "green")] },
      { cells: ["End", "End Ability", "Success", "Server", tag("Ready", "green")] }
    ], "0.9fr 1.2fr 1.4fr 0.9fr 86px"))}
    ${panel("Ability Timing", timeline("ability"))}
  </div>`;
}
