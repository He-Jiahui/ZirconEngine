import { escapeHtml } from "./view-utils.js";

export function renderStageBody(type) {
  if (type === "scene") return renderSceneStage();
  if (type === "material") return renderMaterialStage();
  if (type === "ui") return renderUiStage();
  if (type === "montage") return renderMontageStage();
  if (type === "assets") return renderAssetStage();
  if (type === "diagnostics") return renderDiagnosticsStage();
  return renderProjectStage();
}

function renderSceneStage() {
  return `
    <div class="viewport scene-viewport">
      <div class="viewport-depth"></div>
      <div class="floor-grid"></div>
      <div class="scene-crate crate-a"></div>
      <div class="scene-crate crate-b"></div>
      <div class="selected-cube"><span></span></div>
      <div class="gizmo x-axis"></div>
      <div class="gizmo y-axis"></div>
      <div class="gizmo z-axis"></div>
      <div class="axis-widget">X&nbsp;&nbsp;Y&nbsp;&nbsp;Z</div>
      <div class="viewport-badge">Grid: 100 / Units: cm</div>
    </div>
  `;
}

function renderMaterialStage() {
  return `
    <div class="graph-stage">
      <div class="preview-sphere"></div>
      ${graphNode("Texture Sample", "Base Color", 14, 12, "wide")}
      ${graphNode("Normal Map", "Texture", 14, 42, "wide")}
      ${graphNode("Multiply", "Blend", 38, 24)}
      ${graphNode("Moss Mask", "Parameter", 47, 56)}
      ${graphNode("Roughness", "0.65", 58, 33)}
      ${graphNode("M_Rock_Cliff", "Output", 78, 25, "output")}
      <span class="wire w1"></span>
      <span class="wire w2"></span>
      <span class="wire w3"></span>
      <span class="mini-map"></span>
    </div>
  `;
}

function graphNode(title, subtitle, left, top, extra = "") {
  return `
    <article class="graph-node ${extra}" style="left:${left}%;top:${top}%">
      <strong>${escapeHtml(title)}</strong>
      <span>${escapeHtml(subtitle)}</span>
      <i></i><i></i><i></i>
    </article>
  `;
}

function renderUiStage() {
  return `
    <div class="ui-designer">
      <div class="device-frame">
        <header></header>
        <section class="hud-card large"></section>
        <section class="hud-card small a"></section>
        <section class="hud-card small b"></section>
        <footer></footer>
      </div>
      <div class="responsive-rulers">
        <span>Desktop 16:9</span>
        <span>Tablet 4:3</span>
        <span>Compact</span>
      </div>
    </div>
  `;
}

function renderMontageStage() {
  return `
    <div class="montage-stage">
      <div class="character-preview">
        <span class="rig-head"></span>
        <span class="rig-body"></span>
        <span class="rig-arm left"></span>
        <span class="rig-arm right"></span>
      </div>
      <div class="timeline">
        ${["Start", "AttackA", "HitWindow", "AttackB", "Recover"].map((label, index) => `<span style="--i:${index}">${label}</span>`).join("")}
      </div>
    </div>
  `;
}

function renderAssetStage() {
  const rows = ["SM_Tree_Oak_01", "SM_Tree_Pine_01", "SM_Rock_Cliff_01", "T_Forest_Ground", "M_Forest_Bark", "BP_Forest_DayNight"];
  return `
    <div class="asset-stage">
      <div class="asset-toolbar">
        <button class="active">Type: All</button>
        <button>Status: All</button>
        <button>Tags: All</button>
        <label><span>Search</span><input placeholder="Search assets" /></label>
      </div>
      <div class="asset-table">
        ${rows
          .map(
            (name, index) => `
              <div class="asset-row ${index === 0 ? "selected" : ""}">
                <span class="asset-thumb"></span>
                <strong>${name}</strong>
                <span>${index % 2 ? "Material" : "Static Mesh"}</span>
                <em>${index === 0 ? "Valid" : "Ready"}</em>
              </div>
            `
          )
          .join("")}
      </div>
    </div>
  `;
}

function renderDiagnosticsStage() {
  return `
    <div class="diagnostic-stage">
      ${["Frame", "GPU", "Draw Calls", "Assets"].map((label, index) => renderMetric(label, index)).join("")}
      <div class="trace-chart">
        ${Array.from({ length: 28 }, (_, index) => `<span style="--h:${30 + ((index * 17) % 54)}%"></span>`).join("")}
      </div>
    </div>
  `;
}

function renderMetric(label, index) {
  const values = ["14.8 ms", "9.6 ms", "1,284", "4,120"];
  return `
    <article class="metric-card">
      <span>${escapeHtml(label)}</span>
      <strong>${values[index]}</strong>
      <em>${index === 1 ? "High" : "Nominal"}</em>
    </article>
  `;
}

function renderProjectStage() {
  return `
    <div class="project-stage">
      ${["Source Control", "Build Farm", "Asset Health", "Runtime Tests", "Team Activity", "Release Gate"].map(
        (title, index) => `
          <article class="project-tile ${index === 2 ? "warning" : ""}">
            <span>${escapeHtml(title)}</span>
            <strong>${index === 2 ? "19 warnings" : index === 1 ? "2 queued" : "Ready"}</strong>
            <em>${index % 2 ? "Updated 10 min ago" : "Live"}</em>
          </article>
        `
      ).join("")}
    </div>
  `;
}
