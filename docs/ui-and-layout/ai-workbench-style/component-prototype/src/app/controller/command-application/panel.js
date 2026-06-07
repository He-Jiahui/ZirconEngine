export function applyPanelCommandRoute(route, { activatePanelTarget }) {
  activatePanelTarget(route.panelTarget, { commandId: route.command });
}
