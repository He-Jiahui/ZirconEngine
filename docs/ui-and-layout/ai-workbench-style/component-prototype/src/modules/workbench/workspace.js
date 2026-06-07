import {
  moduleLeft,
  moduleMain,
  moduleRight
} from "../shared/module-components.js";
import { defaultModuleId, moduleById } from "./registry.js";

export function moduleWorkspace(activeId = defaultModuleId) {
  const module = moduleById(activeId);
  return `${moduleLeft(module)}${moduleMain(module)}${moduleRight(module)}${module.bottom()}`;
}
