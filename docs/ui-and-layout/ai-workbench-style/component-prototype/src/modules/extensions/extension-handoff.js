import { alerts } from "../../components/data/collections.js";
import { actionButton, moduleTable, settingsRows, tag } from "../shared/module-components.js";

const esc = (value) => String(value ?? "").replace(/[&<>"']/g, (char) => ({
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#039;"
}[char]));

export function extensionHandoffPanel(config) {
  const routeOptions = handoffRouteOptions(config);
  return `<div class="zr-module-output-grid" data-extension-handoff="${esc(config.id)}">
    ${moduleTable(["Gate", "Evidence", "State"], extensionHandoffRows(config), "1fr 1.35fr 0.85fr", routeOptions)}
    ${settingsRows([
      ["Web Blueprint", tag(config.blueprint ? "Reference Specific" : "Category Recipe", config.blueprint ? "green" : "blue")],
      ["Native Module", tag("Prototype Only", "orange")],
      ["ZUI Workspace", tag("pending .zui workspace", "orange")],
      ["Retained Route", tag("pending binding", "orange")],
      ["Taffy Layout", tag(`${titleWord(config.layoutKind)} grammar`, "cyan")]
    ])}
    ${alerts([
      ["info", `${esc(config.label)} is ready as a browser handoff sample.`],
      ["warning", "Promotion requires .zui workspace, retained binding, preview action, and route evidence."],
      ["success", "Low-level component families are already shared through the native component contract."]
    ])}
    <div class="zr-module-log">
      <p>${esc(config.source)} remains prototype-only until native retained/Taffy evidence exists.</p>
      <p class="is-success">Use this panel as the visible migration checklist instead of adding page-specific layout code.</p>
      ${actionButton("Open Native Gate", "check", routeOptions)}
      ${actionButton("Review Matrix", "history", routeOptions)}
      ${actionButton("More Editors", "grid")}
    </div>
  </div>`;
}

function handoffRouteOptions(config) {
  return {
    actionScope: "workbench.extension.handoff",
    routePanel: `module-bottom-${config.id}:handoff`
  };
}

function extensionHandoffRows(config) {
  return [
    { cells: ["Component grammar", "buttons, fields, tabs, rows, popup, drawer", tag("Covered", "green")], selected: true },
    { cells: ["Browser module", `${config.label} / ${config.category}`, tag("Ready", "cyan")] },
    { cells: ["Reference source", config.source, tag(config.blueprint ? "Specific" : "Recipe", config.blueprint ? "green" : "blue")] },
    { cells: ["Native workspace", "pending .zui workspace", tag("Pending", "orange")] },
    { cells: ["Retained route", "pending WorkbenchModule binding", tag("Pending", "orange")] },
    { cells: ["Preview action", "pending retained preview action", tag("Pending", "orange")] }
  ];
}

function titleWord(word) {
  return String(word)
    .split(/[-_\s]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}
