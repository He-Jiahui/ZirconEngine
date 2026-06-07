export function applyModuleCommandRoute(route, { activateModule }) {
  activateModule(route.moduleId, `Route: ${route.label}`, { panelTarget: route.panelTarget, commandId: route.command });
}
