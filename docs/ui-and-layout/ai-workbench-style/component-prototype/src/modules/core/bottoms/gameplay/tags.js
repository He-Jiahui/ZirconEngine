import { alerts } from "../../../../components/data/collections.js";
import { select, toggle } from "../../../../components/inputs/atoms.js";
import { moduleTable, settingsRows, tag } from "../../../shared/module-components.js";
import { coreBottomRouteOptions } from "../routes.js";

export function tagsBottom() {
  const routeOptions = coreBottomRouteOptions("gameplay-tags", "validation-log");
  return `<div class="zr-module-output-grid">
    ${alerts([["error", "2 errors"], ["warning", "8 warnings"], ["info", "6 infos"]])}
    ${moduleTable(["Severity", "Tag", "Message", "Source"], [
      { cells: [tag("Error", "orange"), "Character.State.Stunned", "Redirect conflict: also redirected from Character.State.Stun", "DefaultGameplayTags.ini:42"], selected: true },
      { cells: [tag("Error", "orange"), "Ability.Unknown", "Invalid tag name", "DefaultGameplayTags.ini:113"] },
      { cells: [tag("Warning", "orange"), "Character.Type", "Deprecated tag used 62 times", "DefaultGameplayTags.ini:78"] },
      { cells: [tag("Info", "blue"), "Combat.Heal", "Tag is valid", "CombatTags.ini:55"] }
    ], "98px 1fr 2fr 1.2fr", routeOptions)}
    ${settingsRows([["Export", select("CSV")], ["Filter", select("All")], ["Auto Fix", toggle("", false)]])}
  </div>`;
}
