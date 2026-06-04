import { icon } from "./icons.js";
import { cluster } from "./layout.js";
import { select } from "./atoms.js";

const esc = (value) => String(value ?? "").replace(/[&<>"']/g, (char) => ({
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#039;"
}[char]));

export function moduleLeft(module) {
  return `<aside class="zr-panel zr-module-left" data-surface="drawer" data-module-panel="left" data-panel-host="${esc(module.id)}">${module.left().join("")}</aside>`;
}

export function moduleMain(module) {
  return `<section class="zr-viewport zr-module-main is-${esc(module.id)}" data-surface="module-main" data-module-panel="main" data-module-active="${esc(module.id)}">
    <div class="zr-module-mainbar">
      <div class="zr-module-title">${icon(module.icon)}<strong>${esc(module.label)}</strong><span>${esc(module.status)}</span></div>
      ${cluster({ className: "zr-module-main-actions", gap: "sm", children: [actionIcon("Select", "cursor", true), actionIcon("Move", "move"), actionIcon("Frame", "target"), select("100%")] })}
    </div>
    ${module.center()}
  </section>`;
}

export function moduleRight(module) {
  return `<aside class="zr-panel zr-module-right" data-surface="window" data-module-panel="right" data-panel-host="${esc(module.id)}">${module.right()}</aside>`;
}

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

export function panel(title, body, actions = "") {
  return `<section class="zr-module-card" data-module-card>
    <header class="zr-module-card-head"><span>${esc(title)}</span>${actions || actionIcon(`More ${title}`, "more")}</header>
    <div class="zr-module-card-body">${body}</div>
  </section>`;
}

export function panelTabs(items, active, panel) {
  return `<div class="zr-panel-tabs zr-module-panel-tabs" role="tablist">${items.map((item, index) => {
    const key = tabKey(item);
    return `<button class="zr-panel-tab ${index === active ? "is-active" : ""}" type="button" role="tab" aria-selected="${index === active ? "true" : "false"}" data-panel-tab="${esc(panel)}:${key}">${esc(item)}</button>`;
  }).join("")}</div>`;
}

export function panelView(panel, key, active, content) {
  return `<div class="zr-panel-view ${active ? "is-active" : ""}" data-surface="panel-view" data-panel-view="${esc(panel)}:${esc(key)}">${content}</div>`;
}

export function panelGroup(panel, items, options = {}) {
  const active = Math.max(0, items.findIndex((item) => item.active));
  const activeIndex = active >= 0 ? active : Number(options.active ?? 0);
  const host = options.host ?? panel;
  const classes = ["zr-panel-group"];
  if (options.className) classes.push(options.className);
  return `<div class="${classes.join(" ")}" data-panel-group="${esc(panel)}" data-panel-host="${esc(host)}">
    ${panelTabs(items.map((item) => item.label), activeIndex, panel)}
    <div class="zr-panel-group-body">${items.map((item, index) => panelView(panel, tabKey(item.key ?? item.label), index === activeIndex, item.content)).join("")}</div>
  </div>`;
}

export function tabKey(value) {
  return String(value).toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
}

function tableRowActionId(row, index) {
  const readableCell = row.cells.find((cell) => {
    const text = String(cell ?? "").trim();
    return text && !/^\d+(?:\.\d+)?$/.test(text) && !/[<>]/.test(text);
  });
  return tabKey(readableCell ?? `row-${index + 1}`);
}

function tableRowLabel(row, index) {
  const readableCell = row.cells.find((cell) => {
    const text = String(cell ?? "").trim();
    return text && !/^\d+(?:\.\d+)?$/.test(text) && !/[<>]/.test(text);
  });
  return String(readableCell ?? `Row ${index + 1}`);
}

export function actionIcon(label, glyph, active = false) {
  return `<button class="zr-icon-button ${active ? "is-active" : ""}" type="button" title="${esc(label)}" aria-label="${esc(label)}" data-action="${tabKey(label)}">${icon(glyph)}</button>`;
}

export function actionButton(label, glyph, options = {}) {
  const classes = ["zr-button", "zr-module-action"];
  if (options.active) classes.push("is-active");
  if (options.kind) classes.push(`is-${options.kind}`);
  return `<button class="${classes.join(" ")}" type="button" data-action="${tabKey(label)}">${glyph ? icon(glyph) : ""}<span>${esc(label)}</span></button>`;
}

export function settingsRows(rows) {
  return `<div class="zr-module-settings">${rows.map(([label, control]) => `<div class="zr-module-setting"><span>${esc(label)}</span><span>${control}</span></div>`).join("")}</div>`;
}

export function listRows(items, selected = 0, values = []) {
  return `<div class="zr-list zr-module-list">${items.map((item, index) => `<button class="zr-list-item zr-module-list-row ${index === selected ? "is-selected" : ""}" type="button" data-action="${tabKey(item)}" aria-label="${esc(item)}"><span>${esc(item)}</span><small>${esc(values[index] ?? "")}</small></button>`).join("")}</div>`;
}

export function actionStack(labels) {
  return `<div class="zr-module-action-stack">${labels.map((label) => actionButton(label, "", { kind: "secondary" })).join("")}</div>`;
}

export function moduleTree(rows) {
  return `<div class="zr-tree zr-module-tree">${rows.map(([label, glyph, selected, depth]) => `<button class="zr-tree-row zr-module-tree-row ${selected ? "is-selected" : ""}" type="button" data-tree-row="${tabKey(label)}" data-depth="${depth}"><span>${depth > 0 ? icon("chevronRight") : icon("chevronDown")}</span>${icon(glyph)}<span>${esc(label)}</span><small>${selected ? icon("check") : ""}</small></button>`).join("")}</div>`;
}

export function segmentButtons(items, active = 0) {
  return `<div class="zr-segment zr-module-segment" role="tablist">${items.map((item, index) => `<button class="zr-segment-item ${index === active ? "is-active" : ""}" type="button" role="tab" aria-selected="${index === active ? "true" : "false"}">${esc(item)}</button>`).join("")}</div>`;
}

export function moduleTable(headers, rows, columns) {
  return `<div class="zr-table zr-module-table" style="--module-table-cols:${esc(columns)}">
    <div class="zr-module-table-row is-head">${headers.map((header) => `<span>${esc(header)}</span>`).join("")}</div>
    ${rows.map((row, index) => `<div class="zr-module-table-row ${row.selected ? "is-selected" : ""}" role="button" tabindex="0" aria-label="${esc(tableRowLabel(row, index))}" data-action="${tableRowActionId(row, index)}">${row.cells.map((cell) => `<span>${cell}</span>`).join("")}</div>`).join("")}
  </div>`;
}

export function tag(label, tone = "neutral") {
  return `<span class="zr-module-tag is-${esc(tone)}">${esc(label)}</span>`;
}

export function previewTile(kind) {
  return `<div class="zr-module-preview is-${esc(kind)}"><span class="zr-preview-orb"></span><span class="zr-preview-grid"></span></div>`;
}

export function assetStrip(items) {
  return `<div class="zr-module-asset-strip">${items.map((item, index) => `<button class="zr-module-asset" type="button" data-action="${tabKey(item)}"><span class="is-${index + 1}"></span><strong>${esc(item)}</strong><small>${index === 1 ? "Material" : "Texture 2D"}</small></button>`).join("")}</div>`;
}

export function node(label, type, x, y, tone = "blue") {
  return `<button class="zr-module-node is-${esc(tone)}" type="button" data-action="${tabKey(label)}" style="--x:${x}%;--y:${y}%"><strong>${esc(label)}</strong><small>${esc(type)}</small></button>`;
}

export function graphBoard(kind, nodes, links = "") {
  return `<div class="zr-module-graph is-${esc(kind)}">${links}${nodes.join("")}<span class="zr-module-minimap"></span></div>`;
}

export function graphLink(x, y, w, rotate = 0, tone = "soft") {
  return `<span class="zr-graph-link is-${tone}" style="--x:${x}%;--y:${y}%;--w:${w}%;--r:${rotate}deg"></span>`;
}

export function compactStats(items) {
  return `<div class="zr-module-stat-grid">${items.map(([label, value, tone]) => `<span class="zr-module-stat ${tone ? `is-${tone}` : ""}"><small>${esc(label)}</small><strong>${esc(value)}</strong></span>`).join("")}</div>`;
}

export function curvePanel() {
  return `<div class="zr-module-curve">
    <svg viewBox="0 0 260 140" aria-hidden="true">
      <path d="M18 120 H244 M18 84 H244 M18 40 H244 M18 12 V130 H246" />
      <path class="is-base" d="M18 112 C60 70 95 56 132 50 S202 41 244 34" />
      <path class="is-alt" d="M18 106 C50 82 76 70 116 66 S202 64 244 58" />
      <path class="is-cap" d="M18 44 H244" />
    </svg>
  </div>`;
}

export function timeline(kind = "default") {
  return `<div class="zr-module-timeline is-${esc(kind)}">
    <div class="zr-timeline-ruler">${["0.00", "0.50", "1.00", "2.00", "180.00", "240.00", "50.00"].map((tick) => `<span>${tick}</span>`).join("")}</div>
    <div class="zr-timeline-track is-green"></div>
    <div class="zr-timeline-track is-blue"></div>
    <div class="zr-timeline-track is-orange"></div>
    <span class="zr-timeline-playhead"></span>
  </div>`;
}

export function progress(value) {
  return `<span class="zr-module-progress" style="--progress:${Number(value) || 0}%"><span></span><small>${esc(value)}%</small></span>`;
}

function generatedBottomPanel(id, label, index) {
  const key = tabKey(label);
  const route = `module-bottom-${id}:${key}`;
  return `<div class="zr-module-output-grid is-generated-bottom" data-generated-bottom-panel="${esc(route)}">
    ${moduleTable(["Channel", "Scope", "State"], generatedBottomRows(id, label, key, index), "1.2fr 1fr 0.95fr")}
    ${settingsRows([
      ["Panel", tag(label, "cyan")],
      ["Module", tag(titleCase(id), id === "editor-library" ? "orange" : "green")],
      ["Route", tag(route, "blue")],
      ["Mode", select(bottomPanelMode(label))],
      ["Live Filter", select("Active selection")]
    ])}
    <div class="zr-module-log"><p>${esc(titleCase(id))} / ${esc(label)} is composed from the shared bottom drawer panel generator.</p><p class="is-success">Panel tabs, rows, fields, and buttons all route through the same prototype response path.</p><p class="is-warning">${esc(bottomPanelWarning(id, label))}</p>${actionButton(`Open ${label}`, "target")}${actionButton(`Pin ${label}`, "check")}</div>
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

function titleCase(value) {
  return String(value)
    .split("-")
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}
