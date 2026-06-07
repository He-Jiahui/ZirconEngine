import { icon } from "../../foundation/icons.js";
import { cluster } from "../../foundation/layout.js";
import { select } from "../../components/inputs/atoms.js";
import { actionIcon } from "./actions.js";
import { esc } from "./utils.js";

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
