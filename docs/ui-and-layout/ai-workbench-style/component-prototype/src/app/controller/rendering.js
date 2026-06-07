import { popups, rail, statusbar, topbar, workbenchWindow } from "../../components/surfaces/surfaces.js";
import { moduleWorkspace } from "../../modules/modules.js";

export function renderWorkbenchShell(app, moduleId, status) {
  app.innerHTML = workbenchWindow([
    topbar(moduleId),
    rail(moduleId),
    moduleWorkspace(moduleId),
    statusbar(status),
    popups()
  ]);
}

export function findPopupLayer() {
  return document.getElementById("popup-layer");
}
