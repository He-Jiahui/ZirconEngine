import { materialBottom } from "../../core-module-bottoms.js";
import { materialCenter } from "../../core-module-centers.js";
import { materialDetails } from "../../core-module-details.js";
import { materialLeft } from "../../core-module-lefts.js";
import { bottomOutput } from "../../../shared/module-components.js";

export const materialCoreModule = {
  id: "material",
  label: "Material",
  icon: "material",
  status: "M_Rock_Cliff graph open",
  actions: [
    ["save", "Save"],
    ["undo", "Undo"],
    ["check", "Compile"],
    ["play", "Preview"],
    ["cube", "Build"]
  ],
  left: () => materialLeft(),
  center: () => materialCenter(),
  right: () => materialDetails(),
  bottom: () => bottomOutput("material", ["Shader Output", "Preview Variants", "Warnings"], materialBottom())
};
