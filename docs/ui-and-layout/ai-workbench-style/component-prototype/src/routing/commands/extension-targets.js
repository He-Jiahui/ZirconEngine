import { moduleById } from "../../modules/modules.js";

export function extensionRouteForCommand(command, activeModuleId) {
  const module = moduleById(activeModuleId);
  if (!module.extension) return null;
  if (command === "more-editors") {
    return { moduleId: "editor-library", panelTarget: "module-bottom-editor-library:routing-log" };
  }
  return {
    moduleId: activeModuleId,
    panelTarget: `module-bottom-${activeModuleId}:${extensionPanelKeyForCommand(command)}`
  };
}

export function extensionPanelKeyForCommand(command) {
  const tokens = command.split("-").filter(Boolean);
  const verb = tokens[0] ?? "";
  if (["native", "handoff", "promote", "promotion", "matrix", "gate", "zui", "retained"].includes(verb)
    || tokens.some((token) => ["native", "handoff", "promotion", "matrix", "gate", "zui", "retained"].includes(token))) {
    return "handoff";
  }
  if (["validate", "compile", "build", "check", "audit", "open"].includes(verb)
    || tokens.some((token) => ["issue", "issues", "warning", "warnings", "error", "errors"].includes(token))) {
    return "validation";
  }
  if (["reference", "references", "browse", "history", "review", "save", "export", "publish", "migrate", "load"].includes(verb)) {
    return "references";
  }
  return "output";
}
