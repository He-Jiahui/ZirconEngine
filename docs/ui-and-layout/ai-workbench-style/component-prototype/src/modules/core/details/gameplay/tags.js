import { checkbox, input, searchInput, select } from "../../../../components/inputs/atoms.js";
import { alerts } from "../../../../components/data/collections.js";
import { actionButton, listRows, moduleTable, moduleTree, panelGroup, settingsRows, tag } from "../../../shared/module-components.js";
import { coreRightRouteOptions } from "../routes.js";

export function tagsDetails() {
  return panelGroup("tags-right", [
    { label: "Hierarchy", active: true, content: `${searchInput("Search hierarchy...")}${moduleTree([
      ["Ability", "folder", false, 0],
      ["Character", "folder", false, 0],
      ["Character.State", "folder", false, 1],
      ["Character.State.Alive", "target", false, 2],
      ["Character.State.Dead", "target", false, 2],
      ["Character.State.Stunned", "target", true, 2],
      ["Combat", "folder", false, 0],
      ["UI", "folder", false, 0]
    ], coreRightRouteOptions("tags-right:hierarchy"))}${settingsRows([
      ["Tag", input("", { value: "Character.State.Stunned" })],
      ["Namespace", select("Game")],
      ["Source", "DefaultGameplayTags.ini"],
      ["Status", tag("Valid", "green")],
      ["Deprecated", checkbox("", false)]
    ])}` },
    { label: "References", content: moduleTable(["Owner", "Type", "Path"], [
      { cells: ["GA_DashAttack", "Ability", "/Game/Abilities"] },
      { cells: ["BT_Enemy", "AI", "/Game/AI"] },
      { cells: ["WBP_Status", "UI", "/Game/UI"] }
    ], "1fr 0.8fr 1.3fr", coreRightRouteOptions("tags-right:references")) },
    { label: "Owners", content: listRows(["DefaultGameplayTags.ini", "CombatTags.ini", "Native Sets"], 0, ["36", "4", "2"], coreRightRouteOptions("tags-right:owners")) },
    { label: "Redirects", content: `${alerts([["error", "Redirect conflict from Character.State.Stun"]])}${actionButton("Resolve Redirect", "check", coreRightRouteOptions("tags-right:redirects"))}` }
  ]);
}
