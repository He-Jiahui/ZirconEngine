export function applyLocationPanelTarget({ activatePanelTarget }, { requestedPanelTarget }) {
  return requestedPanelTarget
    ? activatePanelTarget(requestedPanelTarget, { fromHistory: true })
    : false;
}
