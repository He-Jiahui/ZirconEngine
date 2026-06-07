import { graphBoard, graphLink, node, panel } from "../../../shared/module-components.js";

export function behaviorCenter() {
  return `<div class="zr-module-editor-grid is-behavior">
    ${panel("BT_Enemy", graphBoard("behavior", [
      node("ROOT", "AI_Enemy_Controller", 46, 8, "neutral"),
      node("Selector", "Running", 46, 24, "cyan"),
      node("Sequence", "Chase", 16, 48, "purple"),
      node("Chase Target", "AI Move To", 15, 62, "blue"),
      node("Attack", "Task", 15, 80, "blue"),
      node("Find Cover", "EQS Find", 50, 72, "blue"),
      node("Patrol", "Task", 76, 60, "blue")
    ], `${graphLink(50, 20, 1, 90)}${graphLink(38, 34, 28, 168)}${graphLink(50, 34, 22, 0)}${graphLink(28, 55, 18, 90)}`))}
  </div>`;
}
