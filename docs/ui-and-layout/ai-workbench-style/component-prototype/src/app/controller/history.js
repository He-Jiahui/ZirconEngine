import { routeHash } from "../route-state.js";

export function syncControllerRouteState({ moduleId, panelTarget, commandId, replace = false }) {
  const nextHash = routeHash(moduleId, panelTarget, commandId);
  if (window.location.hash === nextHash) return;
  const nextUrl = `${window.location.pathname}${window.location.search}${nextHash}`;
  window.history[replace ? "replaceState" : "pushState"]({ moduleId }, "", nextUrl);
}
