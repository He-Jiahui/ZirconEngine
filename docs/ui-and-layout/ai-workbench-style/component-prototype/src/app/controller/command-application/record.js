export function recordPlainCommandRoute(route, { activatePanelTarget, recordCommand }) {
  activatePanelTarget("", { clearCommand: true });
  recordCommand(route.command);
}
