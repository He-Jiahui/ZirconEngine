import { sceneBottom } from "../core-module-bottoms.js";
import { sceneCenter } from "../core-module-centers.js";
import { sceneDetails } from "../core-module-details.js";
import { sceneLeft } from "../core-module-lefts.js";
import { bottomOutput } from "../../shared/module-components.js";

export const sceneCoreModule = {
  id: "scene",
  label: "Scene",
  icon: "cube",
  status: "Scene workbench ready",
  actions: [
    ["save", "Save"],
    ["folder", "Browse"],
    ["grid", "Snap"],
    ["play", "Preview"]
  ],
  left: () => sceneLeft(),
  center: () => sceneCenter(),
  right: () => sceneDetails(),
  bottom: () => bottomOutput("scene", ["Selection", "Console", "Validation"], sceneBottom())
};

export const indexCoreModules = [sceneCoreModule];
