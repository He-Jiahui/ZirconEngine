import { searchInput, toggle } from "../../../../components/inputs/atoms.js";
import { listRows, moduleTree, panel, segmentButtons, settingsRows } from "../../../shared/module-components.js";

export function perceptionLeft() {
  return [
    panel("Sense Tools", `${segmentButtons(["Sight", "Hearing", "Damage", "Team"], 0)}${listRows(["Guard (Balanced)", "Sniper (Long Sight)", "Scout (Wide FOV)", "Berserker (Hearing)"], 0)}`),
    panel("Debug", settingsRows([
      ["Draw Senses", toggle("", true)],
      ["Draw LoS", toggle("", true)],
      ["Draw Hearing", toggle("", true)],
      ["Show Stimuli", toggle("", true)],
      ["Color By Team", toggle("", false)]
    ])),
    panel("AI Assets", `${searchInput("Search...")}${moduleTree([
      ["Perception Configs", "folder", false, 0],
      ["Guard_Perception", "eye", true, 1],
      ["Sniper_Perception", "eye", false, 1],
      ["Stimuli Sources", "folder", false, 0],
      ["GunShot_BP", "audio", false, 1],
      ["Explosion_BP", "sun", false, 1],
      ["Pawn Classes", "folder", false, 0],
      ["AI_Guard", "component", false, 1]
    ])}`)
  ];
}
