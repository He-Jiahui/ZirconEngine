import { assetStrip, graphBoard, graphLink, node, panel } from "../../../shared/module-components.js";

export function materialCenter() {
  return `<div class="zr-module-editor-grid is-material">
    ${panel("Material Graph", graphBoard("material", [
      node("Texture Sample", "Base Color", 8, 14, "blue"),
      node("Texture Sample", "Normal", 10, 52, "blue"),
      node("Multiply", "Blend", 28, 22, "green"),
      node("Lerp", "Mask", 42, 32, "green"),
      node("Roughness", "Parameter 0.65", 58, 25, "green"),
      node("M_Rock_Cliff", "Output", 82, 30, "orange")
    ], `${graphLink(21, 24, 15, 0)}${graphLink(42, 30, 26, 8)}${graphLink(62, 30, 22, 0)}${graphLink(21, 64, 42, -8)}`))}
    ${panel("Preview Variants", assetStrip(["Default", "Wet Surface", "Snowy", "Mossy", "Night"]))}
  </div>`;
}
