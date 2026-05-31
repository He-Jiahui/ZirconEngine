import { inspector, popups, rail, scenePanel, showcase, statusbar, topbar, viewport, workbenchWindow } from "./surfaces.js";
import { icon } from "./icons.js";

const app = document.getElementById("app");

app.innerHTML = workbenchWindow([topbar(), rail(), scenePanel(), viewport(), inspector(), showcase(), statusbar(), popups()]);

const popup = document.getElementById("popup-layer");

document.addEventListener("click", (event) => {
  const toggle = event.target.closest("[data-toggle]");
  if (toggle) {
    if (toggle.dataset.toggle === "switch") {
      toggle.classList.toggle("is-on");
    } else {
      const checked = toggle.classList.toggle("is-checked");
      const box = toggle.querySelector(".zr-check-box");
      if (box) box.innerHTML = checked ? icon("check") : "";
    }
  }

  const radio = event.target.closest("[data-radio]");
  if (radio) {
    const group = radio.closest(".zr-check-stack") ?? radio.parentElement;
    group?.querySelectorAll("[data-radio]").forEach((item) => item.classList.remove("is-checked"));
    radio.classList.add("is-checked");
  }

  const tab = event.target.closest(".zr-tab, .zr-segment-item, .zr-panel-tab");
  if (tab) {
    [...tab.parentElement.children].forEach((item) => {
      item.classList.remove("is-active");
      item.setAttribute("aria-selected", "false");
    });
    tab.classList.add("is-active");
    tab.setAttribute("aria-selected", "true");

    const panelTarget = tab.dataset.panelTab;
    if (panelTarget) {
      const host = tab.closest("[data-panel-host]");
      host?.querySelectorAll(".zr-panel-view").forEach((view) => {
        view.classList.toggle("is-active", view.dataset.panelView === panelTarget);
      });
    }
  }

  const treeRow = event.target.closest("[data-tree-row]");
  if (treeRow) {
    document.querySelectorAll(".zr-tree-row").forEach((row) => row.classList.remove("is-selected"));
    treeRow.classList.add("is-selected");
  }

  const row = event.target.closest(".zr-list-item:not(.is-disabled), .zr-table-row:not(.zr-table-head)");
  if (row) {
    row.parentElement.querySelectorAll(".is-selected").forEach((item) => item.classList.remove("is-selected"));
    row.classList.add("is-selected");
  }

  const railButton = event.target.closest(".zr-rail .zr-icon-button");
  if (railButton) {
    document.querySelectorAll(".zr-rail .zr-icon-button").forEach((item) => item.classList.remove("is-active"));
    railButton.classList.add("is-active");
  }

  const toolButton = event.target.closest(".zr-topbar-tools .zr-icon-button");
  if (toolButton) {
    document.querySelectorAll(".zr-topbar-tools .zr-icon-button").forEach((item) => item.classList.remove("is-active"));
    toolButton.classList.add("is-active");
  }

  const dropdown = event.target.closest("[data-dropdown]");
  if (dropdown && popup) {
    const rect = dropdown.getBoundingClientRect();
    popup.style.left = `${Math.min(rect.left, window.innerWidth - 190)}px`;
    popup.style.top = `${rect.bottom + 6}px`;
    popup.classList.toggle("is-open");
    return;
  }

  if (!event.target.closest(".zr-popup-layer")) {
    popup?.classList.remove("is-open");
  }
});
