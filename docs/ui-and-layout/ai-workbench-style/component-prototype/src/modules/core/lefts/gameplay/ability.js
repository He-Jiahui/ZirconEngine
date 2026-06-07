import { searchInput, select, toggle } from "../../../../components/inputs/atoms.js";
import { listRows, moduleTree, panel, settingsRows, tag } from "../../../shared/module-components.js";

export function abilityLeft() {
  return [
    panel("Ability Task Palette", `${searchInput("Search tasks...")}${listRows(["Activate Ability", "Check State", "Wait Gameplay Event", "Apply Cost", "Apply Cooldown", "Play Montage", "Apply Effect", "End Ability"], 5)}`),
    panel("Ability Assets", `${searchInput("Search assets...")}${moduleTree([
      ["Abilities", "folder", false, 0],
      ["GA_DashAttack", "play", true, 1],
      ["GA_Jump", "play", false, 1],
      ["Animation Montages", "folder", false, 0],
      ["AM_DashAttack_Montage", "history", true, 1],
      ["Effects", "folder", false, 0],
      ["GE_DashAttack_Damage", "gear", false, 1],
      ["GE_DashAttack_Cost", "gear", false, 1]
    ])}`),
    panel("Ability Debug", settingsRows([
      ["Debug Object", select("None")],
      ["Authority", tag("Server", "cyan")],
      ["Prediction", toggle("", true)]
    ]))
  ];
}
