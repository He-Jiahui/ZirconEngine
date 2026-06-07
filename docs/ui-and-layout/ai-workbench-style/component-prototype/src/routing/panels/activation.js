export function applyPanelRoute(panelTarget, root = document) {
  if (!panelTarget) return false;
  const tab = root.querySelector(`[data-panel-tab="${cssEscape(panelTarget)}"]`);
  if (!tab) return false;
  activateTab(tab);
  return true;
}

function activateTab(tab) {
  [...tab.parentElement.children].forEach((item) => {
    item.classList.remove("is-active");
    item.setAttribute("aria-selected", "false");
  });
  tab.classList.add("is-active");
  tab.setAttribute("aria-selected", "true");

  const panelTarget = tab.dataset.panelTab;
  const host = tab.closest("[data-panel-host]");
  host?.querySelectorAll(".zr-panel-view").forEach((view) => {
    view.classList.toggle("is-active", view.dataset.panelView === panelTarget);
  });
}

function cssEscape(value) {
  if (globalThis.CSS?.escape) return globalThis.CSS.escape(value);
  return String(value).replace(/["\\]/g, "\\$&");
}
