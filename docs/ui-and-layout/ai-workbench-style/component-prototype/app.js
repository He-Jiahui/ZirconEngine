import { popups, rail, statusbar, topbar, workbenchWindow } from "./surfaces.js";
import { icon } from "./icons.js";
import { defaultModuleId, moduleById, modules, moduleWorkspace } from "./modules.js";
import { applyPanelRoute, normalizeCommand, routeForCommand } from "./routes.js";

const app = document.getElementById("app");

let activeModuleId = moduleIdFromLocation();
let activePanelTarget = "";
let latestCommandId = commandIdFromLocation();
let latestStatus = moduleById(activeModuleId).status;

function renderWorkbench() {
  app.innerHTML = workbenchWindow([
    topbar(activeModuleId),
    rail(activeModuleId),
    moduleWorkspace(activeModuleId),
    statusbar(latestStatus),
    popups()
  ]);
}

renderWorkbench();
activateLocationModuleState({ silent: true });

let popup = document.getElementById("popup-layer");

function setStatus(message) {
  latestStatus = message;
  const nextResponseCount = Number.parseInt(document.documentElement.dataset.zrResponseCount ?? "0", 10) + 1;
  document.documentElement.dataset.zrResponseCount = String(nextResponseCount);
  document.documentElement.dataset.zrLastResponse = message;
  const target = document.querySelector("[data-status-message]");
  if (!target) return;
  target.textContent = message;
  target.classList.remove("zr-action-flash");
  requestAnimationFrame(() => target.classList.add("zr-action-flash"));
}

function commandLabel(target) {
  const explicit = target.dataset.action || target.dataset.module;
  if (explicit) return explicit.replace(/-/g, " ");
  return target.getAttribute("aria-label")
    || target.getAttribute("title")
    || target.textContent.trim().replace(/\s+/g, " ")
    || target.closest(".zr-module-setting")?.querySelector("span")?.textContent.trim()
    || "command";
}

function fieldLabel(target) {
  return target.getAttribute("aria-label")
    || target.getAttribute("placeholder")
    || target.value
    || target.closest(".zr-module-setting")?.querySelector("span")?.textContent.trim()
    || "field";
}

function requestedModuleIdFromLocation() {
  const params = new URLSearchParams(window.location.hash.replace(/^#/, ""));
  return params.get("module");
}

function requestedPanelTargetFromLocation() {
  const params = new URLSearchParams(window.location.hash.replace(/^#/, ""));
  return params.get("panel");
}

function commandIdFromLocation() {
  const params = new URLSearchParams(window.location.hash.replace(/^#/, ""));
  return params.get("command") || "";
}

function moduleIdFromLocation() {
  const requestedModuleId = requestedModuleIdFromLocation();
  return modules.some((module) => module.id === requestedModuleId) ? requestedModuleId : defaultModuleId;
}

function routeHash(moduleId, panelTarget = "", commandId = "") {
  const params = new URLSearchParams();
  params.set("module", moduleId);
  if (panelTarget) {
    params.set("panel", panelTarget);
  }
  if (commandId) {
    params.set("command", commandId);
  }
  return `#${params.toString()}`;
}

function syncRouteState({ moduleId = activeModuleId, panelTarget = activePanelTarget, commandId = latestCommandId, replace = false } = {}) {
  const nextHash = routeHash(moduleId, panelTarget, commandId);
  if (window.location.hash === nextHash) return;
  const nextUrl = `${window.location.pathname}${window.location.search}${nextHash}`;
  window.history[replace ? "replaceState" : "pushState"]({ moduleId }, "", nextUrl);
}

function syncModuleHash(moduleId, replace = false, panelTarget = "", commandId = latestCommandId) {
  syncRouteState({ moduleId, panelTarget, commandId, replace });
}

function recordCommand(commandId, options = {}) {
  latestCommandId = normalizeCommand(commandId);
  if (!latestCommandId) return;
  syncRouteState({ commandId: latestCommandId, replace: options.replaceHistory });
}

function activateModule(moduleId, statusMessage = `Opened ${moduleById(moduleId).label}`, options = {}) {
  const resolvedModule = moduleById(moduleId);
  activeModuleId = resolvedModule.id;
  activePanelTarget = "";
  latestCommandId = options.commandId ? normalizeCommand(options.commandId) : "";
  latestStatus = moduleById(activeModuleId).status;
  renderWorkbench();
  popup = document.getElementById("popup-layer");
  if (options.panelTarget) {
    activatePanelTarget(options.panelTarget, { fromHistory: true });
  }
  if (!options.fromHistory) {
    syncModuleHash(activeModuleId, options.replaceHistory, activePanelTarget, latestCommandId);
  }
  setStatus(statusMessage);
}

function activatePanelTarget(panelTarget, options = {}) {
  if (!panelTarget) {
    activePanelTarget = "";
    if (options.clearCommand) {
      latestCommandId = "";
    }
    if (!options.fromHistory) {
      syncModuleHash(activeModuleId, options.replaceHistory, "", latestCommandId);
    }
    return false;
  }
  if (!applyPanelRoute(panelTarget)) {
    activePanelTarget = "";
    if (options.clearCommand) {
      latestCommandId = "";
    }
    if (!options.fromHistory) {
      syncModuleHash(activeModuleId, true, "", latestCommandId);
    }
    return false;
  }
  activePanelTarget = panelTarget;
  if (options.commandId) {
    latestCommandId = normalizeCommand(options.commandId);
  }
  if (!options.fromHistory) {
    syncModuleHash(activeModuleId, options.replaceHistory, activePanelTarget, latestCommandId);
  }
  if (options.statusMessage) {
    setStatus(options.statusMessage);
  }
  return true;
}

function resetModulePanels() {
  renderWorkbench();
  popup = document.getElementById("popup-layer");
  activePanelTarget = "";
}

function activateLocationModuleState(options = {}) {
  const requestedModuleId = requestedModuleIdFromLocation();
  const rawRequestedPanelTarget = requestedPanelTargetFromLocation();
  const requestedCommandId = commandIdFromLocation();
  const nextModuleId = moduleIdFromLocation();
  const requestedPanelTarget = requestedModuleId === nextModuleId ? rawRequestedPanelTarget : "";
  const moduleChanged = nextModuleId !== activeModuleId;
  if (requestedModuleId !== nextModuleId) {
    syncModuleHash(nextModuleId, true, "", "");
  }
  if (moduleChanged) {
    activeModuleId = nextModuleId;
    latestStatus = moduleById(activeModuleId).status;
    resetModulePanels();
  } else if (!requestedPanelTarget && activePanelTarget) {
    resetModulePanels();
  }
  const panelApplied = requestedPanelTarget
    ? activatePanelTarget(requestedPanelTarget, { fromHistory: true })
    : false;
  if (rawRequestedPanelTarget && (!requestedPanelTarget || !panelApplied)) {
    syncModuleHash(nextModuleId, true, "", requestedCommandId);
  }
  latestCommandId = normalizeCommand(requestedCommandId);
  if (!options.silent && (moduleChanged || requestedPanelTarget || activePanelTarget || latestCommandId)) {
    const panelLabel = activePanelTarget ? ` / ${activePanelTarget.split(":").at(-1).replace(/-/g, " ")}` : "";
    const commandLabel = latestCommandId ? ` / ${latestCommandId.replace(/-/g, " ")}` : "";
    setStatus(`History: ${moduleById(nextModuleId).label}${panelLabel}${commandLabel}`);
  }
}

function applyCommandRoute(target) {
  const route = routeForCommand(target.dataset.action || commandLabel(target), activeModuleId);
  if (!route) return false;
  if (route.moduleId !== activeModuleId) {
    activateModule(route.moduleId, `Route: ${route.label}`, { panelTarget: route.panelTarget, commandId: route.command });
  } else if (route.panelTarget) {
    activatePanelTarget(route.panelTarget, { commandId: route.command });
  } else {
    activatePanelTarget("", { clearCommand: true });
    recordCommand(route.command);
  }
  setStatus(`Route: ${route.label}`);
  return true;
}

document.addEventListener("click", (event) => {
  let handled = false;
  const moduleButton = event.target.closest("[data-module]");
  if (moduleButton) {
    activateModule(moduleButton.dataset.module);
    return;
  }

  const action = event.target.closest("[data-action]");
  if (action) {
    action.classList.remove("zr-action-flash");
    requestAnimationFrame(() => action.classList.add("zr-action-flash"));
    const group = action.closest("[data-action-group]");
    if (group) {
      group.querySelectorAll(".is-active").forEach((item) => item.classList.remove("is-active"));
      action.classList.add("is-active");
    }
    if (!applyCommandRoute(action)) {
      recordCommand(action.dataset.action || commandLabel(action));
      setStatus(`Action: ${commandLabel(action)}`);
    }
    if (action.closest(".zr-popup-layer")) {
      popup?.classList.remove("is-open");
    }
    handled = true;
  }

  const toggle = event.target.closest("[data-toggle]");
  if (toggle) {
    if (toggle.dataset.toggle === "switch") {
      toggle.classList.toggle("is-on");
    } else {
      const checked = toggle.classList.toggle("is-checked");
      const box = toggle.querySelector(".zr-check-box");
      if (box) box.innerHTML = checked ? icon("check") : "";
    }
    recordCommand(`toggle-${commandLabel(toggle)}`);
    setStatus(`Toggled ${commandLabel(toggle)}`);
    handled = true;
  }

  const radio = event.target.closest("[data-radio]");
  if (radio) {
    const group = radio.closest(".zr-check-stack") ?? radio.parentElement;
    group?.querySelectorAll("[data-radio]").forEach((item) => item.classList.remove("is-checked"));
    radio.classList.add("is-checked");
    recordCommand(`radio-${commandLabel(radio)}`);
    setStatus(`Selected ${commandLabel(radio)}`);
    handled = true;
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
      activatePanelTarget(panelTarget, { commandId: `panel-${panelTarget}` });
    } else {
      recordCommand(`tab-${commandLabel(tab)}`);
    }
    setStatus(`Tab: ${commandLabel(tab)}`);
    handled = true;
  }

  const treeRow = event.target.closest("[data-tree-row]");
  if (treeRow) {
    document.querySelectorAll(".zr-tree-row, .zr-module-tree-row").forEach((row) => row.classList.remove("is-selected"));
    treeRow.classList.add("is-selected");
    const command = normalizeCommand(commandLabel(treeRow));
    if (!applyCommandRoute(treeRow)) {
      recordCommand(`tree-${command}`);
      setStatus(`Selected ${command.replace(/-/g, " ")}`);
    }
    handled = true;
  }

  const row = event.target.closest(".zr-list-item:not(.is-disabled), .zr-table-row:not(.zr-table-head), .zr-module-list-row, .zr-module-table-row:not(.is-head)");
  if (row) {
    row.parentElement.querySelectorAll(".is-selected").forEach((item) => item.classList.remove("is-selected"));
    row.classList.add("is-selected");
    if (!applyCommandRoute(row)) {
      recordCommand(`row-${commandLabel(row)}`);
      setStatus(`Selected ${commandLabel(row)}`);
    }
    handled = true;
  }

  const railButton = event.target.closest(".zr-rail .zr-icon-button");
  if (railButton) {
    document.querySelectorAll(".zr-rail .zr-icon-button").forEach((item) => item.classList.remove("is-active"));
    railButton.classList.add("is-active");
    recordCommand(`rail-${commandLabel(railButton)}`);
    handled = true;
  }

  const toolButton = event.target.closest(".zr-topbar-tools .zr-icon-button");
  if (toolButton) {
    document.querySelectorAll(".zr-topbar-tools .zr-icon-button").forEach((item) => item.classList.remove("is-active"));
    toolButton.classList.add("is-active");
    recordCommand(`tool-${commandLabel(toolButton)}`);
    handled = true;
  }

  const dropdown = event.target.closest("[data-dropdown]");
  if (dropdown && popup) {
    const rect = dropdown.getBoundingClientRect();
    popup.style.left = `${Math.min(rect.left, window.innerWidth - 190)}px`;
    popup.style.top = `${rect.bottom + 6}px`;
    popup.classList.toggle("is-open");
    recordCommand(`dropdown-${commandLabel(dropdown)}`);
    setStatus(`Dropdown: ${commandLabel(dropdown)}`);
    return;
  }

  if (!event.target.closest(".zr-popup-layer")) {
    popup?.classList.remove("is-open");
  }

  const genericButton = event.target.closest("button, .zr-menu-row");
  if (genericButton && !handled) {
    genericButton.classList.remove("zr-action-flash");
    requestAnimationFrame(() => genericButton.classList.add("zr-action-flash"));
    recordCommand(commandLabel(genericButton));
    setStatus(`Command: ${commandLabel(genericButton)}`);
  }
});

document.addEventListener("focusin", (event) => {
  const field = event.target.closest("input:not([disabled]), textarea:not([disabled])");
  if (!field) return;
  field.classList.add("is-focused");
  recordCommand(`focus-${fieldLabel(field)}`, { replaceHistory: true });
  setStatus(`Focused: ${fieldLabel(field)}`);
});

document.addEventListener("keydown", (event) => {
  if (event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey) return;
  if (!["Enter", " ", "Spacebar"].includes(event.key)) return;

  const editable = event.target.closest("input, textarea, select, [contenteditable='true']");
  if (editable) return;

  const target = event.target.closest('button[data-action], [role="button"]:not(button), [tabindex="0"][data-action]:not(button)');
  if (!target) return;

  event.preventDefault();
  target.click();
});

document.addEventListener("input", (event) => {
  const field = event.target.closest("input:not([disabled]), textarea:not([disabled])");
  if (!field) return;
  recordCommand(`edit-${fieldLabel(field)}`, { replaceHistory: true });
  setStatus(`Edited: ${fieldLabel(field)}`);
});

window.addEventListener("popstate", activateLocationModuleState);
window.addEventListener("hashchange", activateLocationModuleState);
