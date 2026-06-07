import { actionPath } from "../../foundation/action-paths.js";
import { esc, routeAttrs } from "./utils.js";

export function tag(label, tone = "neutral") {
  return `<span class="zr-module-tag is-${esc(tone)}">${esc(label)}</span>`;
}

export function previewTile(kind) {
  return `<div class="zr-module-preview is-${esc(kind)}"><span class="zr-preview-orb"></span><span class="zr-preview-grid"></span></div>`;
}

export function assetStrip(items, options = {}) {
  return `<div class="zr-module-asset-strip">${items.map((item, index) => `<button class="zr-module-asset" type="button" data-action="${actionPath(options.actionScope ?? "workbench.module.asset", item)}"${routeAttrs(options)}><span class="is-${index + 1}"></span><strong>${esc(item)}</strong><small>${index === 1 ? "Material" : "Texture 2D"}</small></button>`).join("")}</div>`;
}

export function node(label, type, x, y, tone = "blue", options = {}) {
  return `<button class="zr-module-node is-${esc(tone)}" type="button" data-action="${actionPath(options.actionScope ?? "workbench.module.graph", label)}"${routeAttrs(options)} style="--x:${x}%;--y:${y}%"><strong>${esc(label)}</strong><small>${esc(type)}</small></button>`;
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
