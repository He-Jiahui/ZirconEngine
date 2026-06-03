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
  return `<section class="zr-panel zr-module-bottom" data-surface="drawer" data-module-panel="bottom" data-panel-host="module-bottom-${esc(id)}">
    ${panelTabs(tabLabels, 0, `module-bottom-${id}`)}
    <div class="zr-module-bottom-body">${panelView(`module-bottom-${id}`, tabKey(tabLabels[0]), true, body)}${tabLabels.slice(1).map((label) => panelView(`module-bottom-${id}`, tabKey(label), false, placeholderOutput(label))).join("")}</div>
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

function placeholderOutput(label) {
  return `<div class="zr-module-placeholder"><strong>${esc(label)}</strong><span>No live data is connected in this HTML/CSS prototype.</span>${actionButton("Acknowledge", "check")}</div>`;
}
