import { titleWord } from "../../extension-configs.js";
import { panel, settingsRows, tag } from "../../../shared/module-components.js";

export function extensionReferencePanel(config) {
  return panel("Reference", settingsRows([
    ["AI Sample", tag(config.source.replace(/^ai-|-layout\.png$/g, "").replace(/-/g, " "), "cyan")],
    ["Category", tag(config.category, "blue")],
    ["Layout", tag(`${titleWord(config.layoutKind)} Workbench`, "green")],
    ["Native Contract", tag("Prototype Only", "orange")]
  ]));
}
