import { applyCommandRouteForTarget } from "../command-application.js";

export function createWorkbenchCommandHandler({
  state,
  activateModule,
  activatePanelTarget,
  recordCommand,
  setStatus
}) {
  return function handleWorkbenchCommandRoute(target) {
    return applyCommandRouteForTarget(target, {
      state,
      activateModule,
      activatePanelTarget,
      recordCommand,
      setStatus
    });
  };
}
