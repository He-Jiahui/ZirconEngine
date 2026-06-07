import { defaultModuleId, modules } from "../modules/modules.js";

export function requestedModuleIdFromLocation() {
  const params = new URLSearchParams(window.location.hash.replace(/^#/, ""));
  return params.get("module");
}

export function requestedPanelTargetFromLocation() {
  const params = new URLSearchParams(window.location.hash.replace(/^#/, ""));
  return params.get("panel");
}

export function commandIdFromLocation() {
  const params = new URLSearchParams(window.location.hash.replace(/^#/, ""));
  return params.get("action") || params.get("command") || "";
}

export function moduleIdFromLocation() {
  const requestedModuleId = requestedModuleIdFromLocation();
  return modules.some((module) => module.id === requestedModuleId) ? requestedModuleId : defaultModuleId;
}

export function routeHash(moduleId, panelTarget = "", commandId = "") {
  const params = new URLSearchParams();
  params.set("module", moduleId);
  if (panelTarget) {
    params.set("panel", panelTarget);
  }
  if (commandId) {
    params.set("action", commandId);
  }
  return `#${params.toString()}`;
}
