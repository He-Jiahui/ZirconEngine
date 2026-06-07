import { select } from "../../components/inputs/atoms.js";
import { actionSegment } from "../../foundation/action-paths.js";
import { actionButton } from "./actions.js";
import { panelGroup } from "./panels.js";
import { moduleTable, settingsRows } from "./rows.js";
import { esc, tabKey, titleCase } from "./utils.js";
import { progress, tag } from "./visuals.js";

export function bottomOutput(id, tabLabels, body) {
  const panelId = `module-bottom-${id}`;
  return `<section class="zr-panel zr-module-bottom" data-surface="drawer" data-module-panel="bottom" data-panel-host="module-bottom-${esc(id)}">
    ${panelGroup(panelId, tabLabels.map((label, index) => ({
      label,
      active: index === 0,
      content: index === 0 ? body : generatedBottomPanel(id, label, index)
    })), { className: "is-module-bottom" })}
  </section>`;
}

export function generatedBottomPanel(id, label, index) {
  const key = tabKey(label);
  const route = `module-bottom-${id}:${key}`;
  const actionScope = `workbench.generated_bottom.${actionSegment(id)}.${actionSegment(key)}`;
  const routeOptions = { actionScope, routePanel: route };
  return `<div class="zr-module-output-grid is-generated-bottom" data-generated-bottom-panel="${esc(route)}">
    ${moduleTable(["Channel", "Scope", "State"], generatedBottomRows(id, label, key, index), "1.2fr 1fr 0.95fr", routeOptions)}
    ${settingsRows([
      ["Panel", tag(label, "cyan")],
      ["Module", tag(titleCase(id), id === "editor-library" ? "orange" : "green")],
      ["Route", tag(route, "blue")],
      ["Mode", select(bottomPanelMode(label))],
      ["Live Filter", select("Active selection")]
    ])}
    <div class="zr-module-log"><p>${esc(titleCase(id))} / ${esc(label)} is composed from the shared bottom drawer panel generator.</p><p class="is-success">Panel tabs, rows, fields, and buttons all route through the same prototype response path.</p><p class="is-warning">${esc(bottomPanelWarning(id, label))}</p>${actionButton(`Open ${label}`, "target", routeOptions)}${actionButton(`Pin ${label}`, "check", routeOptions)}</div>
  </div>`;
}

function generatedBottomRows(id, label, key, index) {
  const title = titleCase(id);
  return [
    { cells: [`${label} Feed`, title, tag("Routed", "green")], selected: true },
    { cells: ["Selected Context", `${title} shared drawer`, tag(bottomPanelMode(label), "cyan")] },
    { cells: ["Progress", key, progress(Math.min(100, 42 + index * 12))] },
    { cells: ["Native Target", id === "editor-library" ? "Prototype library route" : "Retained/Taffy bottom panel", tag(id === "editor-library" ? "Web" : "Core", id === "editor-library" ? "orange" : "green")] }
  ];
}

function bottomPanelMode(label) {
  const normalized = tabKey(label);
  if (/compile|shader|cook|package|build|error|warning/.test(normalized)) return "Build";
  if (/validation|issue|binding|migration/.test(normalized)) return "Validation";
  if (/timeline|trace|event|simulation|perception|debug|console/.test(normalized)) return "Runtime";
  if (/preview|variant|reference|resource|selection|queue/.test(normalized)) return "Review";
  return "Output";
}

function bottomPanelWarning(id, label) {
  if (id === "editor-library") {
    return `${label} stays in the browser prototype and links back to the extension catalog.`;
  }
  return `${label} is a route-responsive web panel ready for native retained-host promotion.`;
}
