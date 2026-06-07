import { cluster } from "../../../../foundation/layout.js";
import { checkbox, searchInput, select } from "../../../../components/inputs/atoms.js";
import { compactStats, moduleTable, panel, tag } from "../../../shared/module-components.js";

export function tagsCenter() {
  return `<div class="zr-module-editor-grid is-tags">
    ${panel("Gameplay Tag Registry", `${cluster({ className: "zr-module-filterbar", children: [searchInput("Search tags..."), checkbox("Show Inherited", true), select("View Options")] })}${moduleTable(["Tag", "Namespace", "References", "Status", "Source"], [
      { cells: ["Ability.Activate", "Game", "128", tag("Valid", "green"), "DefaultGameplayTags.ini"] },
      { cells: ["Ability.Cancel", "Game", "32", tag("Valid", "green"), "DefaultGameplayTags.ini"] },
      { cells: ["Character.State.Alive", "Game", "68", tag("Valid", "green"), "DefaultGameplayTags.ini"] },
      { cells: ["Character.State.Stunned", "Game", "36", tag("Valid", "green"), "DefaultGameplayTags.ini"], selected: true },
      { cells: ["Character.Type.Player", "Game", "24", tag("Deprecated", "orange"), "DefaultGameplayTags.ini"] },
      { cells: ["Combat.Damage.Physical", "Game", "36", tag("Valid", "green"), "CombatTags.ini"] }
    ], "1.5fr 0.75fr 0.65fr 0.75fr 1.4fr")}`)}
    ${panel("Reference Summary", compactStats([["Direct", "6"], ["Indirect", "30"], ["Owners", "12"], ["Conflicts", "1", "warning"]]))}
  </div>`;
}
