import { actionPath } from "../../../../foundation/action-paths.js";

export function applyPanelTabRoute(tab, controller) {
  const panelTarget = tab.dataset.panelTab;
  if (!panelTarget) return false;

  const host = tab.closest("[data-panel-host]");
  host?.querySelectorAll(".zr-panel-view").forEach((view) => {
    view.classList.toggle("is-active", view.dataset.panelView === panelTarget);
  });
  controller.activatePanelTarget(panelTarget, { commandId: actionPath("workbench.panel.select", panelTarget) });
  return true;
}
