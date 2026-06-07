import { icon } from "../../../foundation/icons.js";
import { cluster } from "../../../foundation/layout.js";
import { defaultModuleId, moduleRail, moduleTabs, moduleToolbar } from "../../../modules/modules.js";
import { iconButton, select } from "../../inputs/atoms.js";

export function topbar(activeModuleId = defaultModuleId) {
  const left = [iconButton("menu", "Menu"), "divider", iconButton("file", "New"), iconButton("folder", "Open"), iconButton("save", "Save"), "divider", iconButton("undo", "Undo"), iconButton("redo", "Redo")];
  const tools = [moduleTabs(activeModuleId), moduleToolbar(activeModuleId)];
  const right = [iconButton("play", "Play", { large: true }), iconButton("chevronDown", "Play options"), "divider", iconButton("grid", "Layout"), iconButton("sun", "Lighting"), iconButton("more", "More")];
  return `<header class="zr-topbar">${toolbarGroup(left)}${toolbarGroup(tools, "zr-topbar-tools")}${toolbarGroup(right)}</header>`;
}

export function rail(activeModuleId = defaultModuleId) {
  return moduleRail(activeModuleId);
}

export function statusbar(statusText = "Ready") {
  const left = cluster({ className: "zr-status-left", gap: "lg", children: [`<span class="zr-status-item"><span class="zr-dot"></span><span class="zr-module-status-message" data-status-message>${statusText}</span></span>`, `<span class="zr-status-item">${icon("check")}No Errors</span>`, `<span class="zr-status-item">${icon("warning")}2 Warnings</span>`, `<span class="zr-status-item">${icon("info")}0 Messages</span>`] });
  const right = cluster({ className: "zr-status-right", gap: "lg", children: [select("Grid: 10 cm"), select("Snap: On"), iconButton("snap", "Snap"), iconButton("globe", "World"), iconButton("target", "Target"), select("100%")] });
  return `<footer class="zr-statusbar">${left}<span></span>${right}</footer>`;
}

function renderGroup(items) {
  return items.map((item) => item === "divider" ? '<span class="zr-divider"></span>' : item).join("");
}

function toolbarGroup(items, className = "zr-topbar-group") {
  return cluster({ className, gap: "sm", children: renderGroup(items) });
}
