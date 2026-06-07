import { cluster } from "../../../../foundation/layout.js";
import { checkbox } from "../../../../components/inputs/atoms.js";
import { actionButton, moduleTree, panel, panelGroup, settingsRows } from "../../../shared/module-components.js";

export function tagsLeft() {
  return [
    panel("Tag Actions", cluster({ className: "zr-module-card-tools", wrap: true, children: [actionButton("Add", "plus"), actionButton("Rename", "file"), actionButton("Move", "move"), actionButton("Duplicate", "file")] })),
    panel("Validation Filters", settingsRows([
      ["Show Invalid", checkbox("", true)],
      ["Show Deprecated", checkbox("", true)],
      ["Show Redirects", checkbox("", true)],
      ["Show Conflicts", checkbox("", true)],
      ["Show Unused", checkbox("", false)]
    ])),
    panel("Sources", panelGroup("tag-sources", [
      { label: "Sources", active: true, content: moduleTree([
        ["Project", "folder", false, 0],
        ["DefaultGameplayTags.ini", "file", true, 1],
        ["Plugins", "folder", false, 0],
        ["GameplayAbilitiesTags.ini", "file", false, 1],
        ["CombatTags.ini", "file", false, 1],
        ["Native Tag Sets", "folder", false, 0],
        ["CoreGameplayTags.ini", "file", false, 1]
      ]) },
      { label: "Plugins", content: moduleTree([
        ["Plugins", "folder", true, 0],
        ["GameplayAbilitiesTags.ini", "file", true, 1],
        ["CombatTags.ini", "file", false, 1]
      ]) },
      { label: "Native Sets", content: moduleTree([
        ["Native Tag Sets", "folder", true, 0],
        ["CoreGameplayTags.ini", "file", true, 1]
      ]) }
    ], { className: "is-card-panel" }))
  ];
}
