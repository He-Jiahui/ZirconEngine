import { tabKey } from "../../shared/module-components.js";

export function extensionRouteOptions(config, panel, actionScope) {
  const routePanel = String(panel).startsWith("right:")
    ? `${config.id}-right:${tabKey(String(panel).slice("right:".length))}`
    : `module-bottom-${config.id}:${panel}`;
  return {
    actionScope,
    routePanel
  };
}
