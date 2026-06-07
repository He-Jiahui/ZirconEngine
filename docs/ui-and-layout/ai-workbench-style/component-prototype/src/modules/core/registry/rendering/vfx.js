import { vfxBottom } from "../../core-module-bottoms.js";
import { vfxCenter } from "../../core-module-centers.js";
import { vfxDetails } from "../../core-module-details.js";
import { vfxLeft } from "../../core-module-lefts.js";
import { bottomOutput } from "../../../shared/module-components.js";

export const vfxCoreModule = {
  id: "vfx",
  label: "VFX",
  icon: "sun",
  status: "P_Bolt_01 previewing at 60 fps",
  actions: [
    ["save", "Save"],
    ["save", "Save All"],
    ["undo", "Undo"],
    ["play", "Simulate"],
    ["check", "Compile"]
  ],
  left: () => vfxLeft(),
  center: () => vfxCenter(),
  right: () => vfxDetails(),
  bottom: () => bottomOutput("vfx", ["Timeline", "Curves", "Niagara Log", "Compile Output", "Event Log"], vfxBottom())
};
