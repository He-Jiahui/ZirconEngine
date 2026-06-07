export function createPanelReset({ state, renderWorkbench }) {
  return function resetModulePanels() {
    renderWorkbench();
    state.activePanelTarget = "";
  };
}
