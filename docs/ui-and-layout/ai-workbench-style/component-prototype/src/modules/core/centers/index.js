import { compactStats, graphBoard, graphLink, node, panel } from "../../shared/module-components.js";

export function sceneCenter() {
  return `<div class="zr-module-editor-grid is-scene">
    ${panel("Viewport Composition", graphBoard("scene", [
      node("Camera", "View", 18, 22, "blue"),
      node("Directional Light", "Lighting", 58, 16, "green"),
      node("PlayerStart", "Spawn", 38, 55, "cyan"),
      node("Props", "Selected", 60, 42, "orange")
    ], `${graphLink(25, 30, 28, 10)}${graphLink(45, 46, 22, -14)}`))}
    ${panel("Scene Metrics", compactStats([["Draw Calls", "184"], ["Lights", "12"], ["Selected", "Props"], ["Warnings", "2", "warning"]]))}
  </div>`;
}
