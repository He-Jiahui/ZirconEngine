import { moduleById } from "../../modules/modules.js";

export function routeLabel(command, moduleId, panelTarget) {
  const module = moduleById(moduleId);
  const panel = panelTarget ? ` / ${panelTarget.split(":").at(-1).replace(/-/g, " ")}` : "";
  return `${command.replace(/-/g, " ")} -> ${module.label}${panel}`;
}
