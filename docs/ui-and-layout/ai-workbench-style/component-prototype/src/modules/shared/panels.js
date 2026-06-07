import { actionIcon } from "./actions.js";
import { esc, tabKey } from "./utils.js";

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
