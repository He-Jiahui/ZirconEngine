import { moduleById } from "../../../modules/modules.js";

export function explicitRouteLabel(command, moduleId, panelTarget) {
  return `${command.replace(/\./g, " ")} -> ${moduleById(moduleId).label}${panelTarget ? ` / ${panelTarget.split(":").at(-1).replace(/-/g, " ")}` : ""}`;
}
