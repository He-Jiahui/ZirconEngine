import { cluster } from "../../../foundation/layout.js";
import { iconButton, select } from "../../inputs/atoms.js";

export function viewport() {
  const gridLines = [
    ...[0, 1, 2, 3, 4, 5].map((line) => `<span class="zr-viewport-grid-line is-horizontal ${line === 2 || line === 4 ? "is-major" : ""}" style="--line:${line}"></span>`),
    ...[0, 1, 2, 3, 4, 5, 6].map((line) => `<span class="zr-viewport-grid-line is-vertical ${line === 2 || line === 5 ? "is-major" : ""}" style="--line:${line}"></span>`),
  ].join("");
  return `<section class="zr-viewport">
    <div class="zr-scene-shell">
      <div class="zr-scene-ceiling"><span class="zr-scene-light l1 is-soft"></span><span class="zr-scene-light l2"></span><span class="zr-scene-light l3"></span><span class="zr-scene-light l4"></span></div>
      <div class="zr-scene-wall"><span class="zr-scene-wall-detail center-lines"></span><div class="zr-scene-door"><span></span></div><span class="zr-scene-wall-panel p1"></span><span class="zr-scene-wall-panel p2"></span><span class="zr-scene-wall-panel p3"></span><span class="zr-scene-column c-left"></span><span class="zr-scene-column c-right"></span><span class="zr-scene-beacon b1"></span><span class="zr-scene-beacon b2"></span></div>
      <span class="zr-scene-lightwash left"></span>
      <span class="zr-scene-lightwash center"></span>
      <span class="zr-scene-shadow top-bay"></span>
      <span class="zr-scene-shadow ceiling-left"></span>
      <span class="zr-scene-shadow ceiling-mid"></span>
      <span class="zr-scene-lightwash wall-right"></span>
      <span class="zr-scene-lightwash rear-walkway"></span>
      <div class="zr-scene-side left"></div>
      <div class="zr-scene-side right"></div>
      <span class="zr-scene-rack left"></span>
      <div class="zr-scene-floor"><span class="zr-floor-reflection"></span><span class="zr-floor-grate right"></span><span class="zr-floor-panel fp1"></span><span class="zr-floor-panel fp2"></span><span class="zr-floor-panel fp3"></span><span class="zr-floor-seam seam-right"></span>${gridLines}</div>
      <span class="zr-scene-lightwash lower"></span>
      <span class="zr-scene-lightwash floor"></span>
      <span class="zr-scene-lightwash floor-cool"></span>
      <span class="zr-scene-lightwash floor-right"></span>
      <span class="zr-scene-shadow left-floor"></span>
      <span class="zr-scene-shadow right-floor"></span>
      <div class="zr-scene-handrail left"></div>
      <div class="zr-scene-handrail right"></div>
      <div class="zr-scene-cargo c1"><span class="zr-cargo-inner"></span></div>
      <div class="zr-scene-cargo c2"><span class="zr-cargo-inner"></span></div>
      <div class="zr-scene-cargo c3"><span class="zr-cargo-inner"></span></div>
      <div class="zr-scene-cargo c4"><span class="zr-cargo-inner"></span></div>
      <div class="zr-crate"><span class="zr-crate-top"></span><span class="zr-selection-edge top"></span><span class="zr-selection-edge right"></span><span class="zr-selection-edge bottom"></span><span class="zr-selection-edge left"></span><span class="zr-transform-origin"></span><span class="zr-transform-axis axis-x"></span><span class="zr-transform-axis axis-y"></span><span class="zr-transform-axis axis-z"></span><span class="zr-transform-label label-x">X</span><span class="zr-transform-label label-y">Y</span></div>
      <span class="zr-axis-mini left"></span>
      <div class="zr-orientation-gizmo"><span class="axis y">Y</span><span class="axis z">Z</span><span class="axis x">X</span><span class="center"></span></div>
      <span class="zr-scene-vignette"></span>
    </div>
    <div class="zr-viewport-tools">${cluster({ as: "span", className: "zr-viewport-cluster", children: [select("Perspective"), select("Lit", { icon: "sun" })] })}${cluster({ as: "span", className: "zr-viewport-cluster", children: [iconButton("target", "Target"), iconButton("grid", "Snap", { active: true }), iconButton("snap", "Snap"), iconButton("snap", "Magnet"), iconButton("folder", "Local"), select("10°"), select("0.25"), iconButton("scale", "Fullscreen")] })}</div>
  </section>`;
}
