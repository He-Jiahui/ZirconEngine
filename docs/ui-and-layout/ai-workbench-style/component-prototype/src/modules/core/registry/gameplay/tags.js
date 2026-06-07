import { tagsBottom } from "../../core-module-bottoms.js";
import { tagsCenter } from "../../core-module-centers.js";
import { tagsDetails } from "../../core-module-details.js";
import { tagsLeft } from "../../core-module-lefts.js";
import { bottomOutput } from "../../../shared/module-components.js";

export const gameplayTagsCoreModule = {
  id: "gameplay-tags",
  label: "Gameplay Tags",
  shortLabel: "Tags",
  icon: "target",
  status: "Character.State.Stunned selected",
  actions: [
    ["plus", "Add Tag"],
    ["file", "Rename"],
    ["move", "Move"],
    ["trash", "Delete"],
    ["check", "Validate Tags"]
  ],
  left: () => tagsLeft(),
  center: () => tagsCenter(),
  right: () => tagsDetails(),
  bottom: () => bottomOutput("gameplay-tags", ["Validation Log", "Reference Scan", "Migration Preview", "Compile Log"], tagsBottom())
};
