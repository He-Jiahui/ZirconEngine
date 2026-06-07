import { moduleById } from "../../modules/modules.js";
import { commandIdFromLocation, moduleIdFromLocation } from "../route-state.js";

export function createControllerState() {
  const initialModuleId = moduleIdFromLocation();
  return {
    activeModuleId: initialModuleId,
    activePanelTarget: "",
    latestCommandId: commandIdFromLocation(),
    latestStatus: moduleById(initialModuleId).status,
    popup: null
  };
}
