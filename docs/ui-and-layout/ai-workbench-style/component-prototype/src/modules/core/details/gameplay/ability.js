import { input, searchInput, select, slider } from "../../../../components/inputs/atoms.js";
import { alerts } from "../../../../components/data/collections.js";
import { actionButton, moduleTree, panelGroup, settingsRows, tag } from "../../../shared/module-components.js";
import { coreRightRouteOptions } from "../routes.js";

export function abilityDetails() {
  return panelGroup("ability-right", [
    { label: "Graph Outline", active: true, content: `${searchInput("Search outline...")}${moduleTree([
      ["GA_DashAttack", "play", true, 0],
      ["Phases", "folder", false, 1],
      ["Activation", "target", false, 2],
      ["Tasks", "list", false, 2],
      ["Play Montage", "history", true, 3],
      ["Wait Gameplay Event", "component", false, 3],
      ["Dependencies", "folder", false, 1],
      ["GE_DashAttack_Damage", "gear", false, 2]
    ], coreRightRouteOptions("ability-right:graph-outline"))}` },
    { label: "Task Properties", content: `${settingsRows([
      ["Task", tag("Play Montage", "cyan")],
      ["Montage", select("AM_DashAttack_Montage")],
      ["Play Rate", input("", { value: "1.0" })],
      ["Start Section", select("Default")],
      ["Targeting Mode", select("Self")],
      ["Prediction Key", select("Use Ability Key")],
      ["Replication", select("Server Initiated")]
    ])}${slider("Blend Weight", 80, "1.0")}` },
    { label: "Validation", content: `${alerts([["success", "Compile succeeded"], ["warning", "Montage has no default slot"], ["info", "Prediction path is simulated"]])}${actionButton("Fix Montage Slot", "check", coreRightRouteOptions("ability-right:validation"))}` }
  ]);
}
