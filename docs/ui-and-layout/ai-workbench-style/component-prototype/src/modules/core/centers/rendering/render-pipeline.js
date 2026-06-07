import { assetStrip, graphBoard, graphLink, node, panel, previewTile, timeline } from "../../../shared/module-components.js";

export function renderPipelineCenter() {
  return `<div class="zr-module-editor-grid is-render">
    ${panel("Render Graph", graphBoard("render", [
      node("GBuffer Pass", "#1", 6, 32, "neutral"),
      node("Lighting Pass", "#2", 24, 28, "green"),
      node("SSR Pass", "#3", 42, 16, "blue"),
      node("SSAO Pass", "#4", 42, 54, "blue"),
      node("Post Process Pass", "#5", 60, 34, "purple"),
      node("Tone Map Pass", "#6", 76, 42, "orange"),
      node("UI Composite Pass", "#7", 88, 46, "neutral")
    ], `${graphLink(18, 42, 9)}${graphLink(36, 32, 8, -18)}${graphLink(36, 58, 8, 18)}${graphLink(54, 28, 8, 24)}${graphLink(54, 62, 9, -18)}${graphLink(70, 46, 7)}${graphLink(84, 50, 5)}`))}
    ${panel("Frame Preview", `${previewTile("render")}${assetStrip(["Albedo", "Normal", "Depth", "Lighting", "PostColor", "BackBuffer"])}`)}
    ${panel("Frame Timeline", timeline("render"))}
  </div>`;
}
