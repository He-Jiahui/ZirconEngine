import { icon } from "../../foundation/icons.js";
import { actionPath } from "../../foundation/action-paths.js";
import { esc, routeAttrs } from "./utils.js";

export function actionIcon(label, glyph, active = false) {
  return `<button class="zr-icon-button ${active ? "is-active" : ""}" type="button" title="${esc(label)}" aria-label="${esc(label)}" data-action="${actionPath("workbench.module.icon", label)}">${icon(glyph)}</button>`;
}

export function actionButton(label, glyph, options = {}) {
  const classes = ["zr-button", "zr-module-action"];
  if (options.active) classes.push("is-active");
  if (options.kind) classes.push(`is-${options.kind}`);
  return `<button class="${classes.join(" ")}" type="button" data-action="${actionPath(options.actionScope ?? "workbench.module.action", label)}"${routeAttrs(options)}>${glyph ? icon(glyph) : ""}<span>${esc(label)}</span></button>`;
}

export function actionStack(labels) {
  return `<div class="zr-module-action-stack">${labels.map((label) => actionButton(label, "", { kind: "secondary" })).join("")}</div>`;
}
