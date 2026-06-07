import { hudBottom } from "../core-module-bottoms.js";
import { hudCenter } from "../core-module-centers.js";
import { hudDetails } from "../core-module-details.js";
import { hudLeft } from "../core-module-lefts.js";
import { bottomOutput } from "../../shared/module-components.js";

export const hudEditorCoreModule = {
  id: "hud-editor",
  label: "HUD Editor",
  shortLabel: "HUD",
  icon: "image",
  status: "WeaponPanel selected in Gameplay_HUD",
  actions: [
    ["save", "Save All"],
    ["undo", "Undo"],
    ["play", "Preview HUD"],
    ["check", "Validate UI"],
    ["cube", "Build UI"]
  ],
  left: () => hudLeft(),
  center: () => hudCenter(),
  right: () => hudDetails(),
  bottom: () => bottomOutput("hud-editor", ["Validation", "Binding Errors", "Preview Log", "Performance", "Compile Log"], hudBottom())
};

export const hudCoreModules = [hudEditorCoreModule];
