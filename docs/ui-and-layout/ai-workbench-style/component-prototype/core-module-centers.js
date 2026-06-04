import { cluster, grid } from "./layout.js";
import { checkbox, input, searchInput, select, slider, toggle } from "./atoms.js";
import {
  actionButton,
  actionIcon,
  assetStrip,
  compactStats,
  curvePanel,
  graphBoard,
  graphLink,
  listRows,
  moduleTable,
  node,
  panel,
  previewTile,
  segmentButtons,
  settingsRows,
  tag,
  timeline
} from "./module-components.js";

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

export function gameplayCenter() {
  return grid({ className: "zr-module-editor-grid is-gameplay", children: [
    panel("Modifiers", `${cluster({ className: "zr-module-card-tools", justify: "end", children: [actionButton("Add Modifier", "plus"), actionButton("Duplicate", "file"), actionButton("Delete", "trash", { kind: "danger" })] })}${moduleTable(["#", "Name", "Attribute", "Modifier Op", "Magnitude", "Source", "Tags"], [
      { cells: ["1", "HealthRegen", "Health", "Additive", "Scalable Float", "Source", tag("Regen.Health", "cyan")], selected: true },
      { cells: ["2", "IncomingHealing", "Healing Received", "Multiplicative", "Scalable Float", "Target", tag("Regen.Bonus", "green")] },
      { cells: ["3", "MaxHealthCap", "Max Health", "Additive", "Captured Attribute", "Source", tag("Regen.Cap", "blue")] },
      { cells: ["4", "RegenPerStack", "Health", "Additive", "Scalable Float", "Source", tag("Regen.Stack", "orange")] }
    ], "38px 1.2fr 1.1fr 1.1fr 1fr 0.8fr 1fr")}`),
    panel("Dependency Graph", graphBoard("dependency", [
      node("HealthRegen", "Modifier", 44, 12, "cyan"),
      node("RegenPerStack", "Modifier", 70, 20, "cyan"),
      node("Health", "Attribute", 46, 42, "blue"),
      node("Max Health", "Attribute", 72, 42, "blue"),
      node("Clamp Health", "Execution", 47, 72, "orange"),
      node("Target Tags", "Require", 18, 72, "green"),
      node("Blocked Tags", "Block", 74, 72, "green")
    ], `${graphLink(52, 22, 22, 8)}${graphLink(50, 38, 16, -18)}${graphLink(52, 58, 18, 90)}${graphLink(32, 80, 18, 0)}${graphLink(60, 80, 18, 0)}`)),
    panel("Attribute Preview", `${settingsRows([["Preview at Level", select("1")]])}${curvePanel()}`)
  ] });
}

export function abilityCenter() {
  return `<div class="zr-module-editor-grid is-ability">
    ${panel("Ability Graph", graphBoard("ability", [
      node("Activate Ability", "Authority", 8, 34, "green"),
      node("Apply Cost", "Cost Effect", 25, 24, "orange"),
      node("Apply Cooldown", "Cooldown Effect", 25, 56, "orange"),
      node("Play Montage", "AM_DashAttack", 44, 35, "cyan"),
      node("Wait Gameplay Event", "Event.Data.Hit", 64, 24, "purple"),
      node("Apply Effect", "GE_DashAttack_Damage", 64, 58, "purple"),
      node("End Ability", "Success", 84, 35, "neutral")
    ], `${graphLink(19, 42, 10)}${graphLink(36, 32, 9)}${graphLink(36, 64, 11, -12)}${graphLink(56, 42, 10)}${graphLink(70, 42, 10, 90)}${graphLink(76, 66, 8, -28)}`))}
    ${panel("Ability Phase Matrix", moduleTable(["Phase", "Task", "Asset", "Net Role", "Status"], [
      { cells: ["Activation", "Activate Ability", "Authority Gate", "Server", tag("Ready", "green")] },
      { cells: ["Tasks", "Play Montage", "AM_DashAttack_Montage", "Predicted", tag("Selected", "cyan")], selected: true },
      { cells: ["Event Response", "Apply Effect", "GE_DashAttack_Damage", "Server", tag("Valid", "green")] },
      { cells: ["End", "End Ability", "Success", "Server", tag("Ready", "green")] }
    ], "0.9fr 1.2fr 1.4fr 0.9fr 86px"))}
    ${panel("Ability Timing", timeline("ability"))}
  </div>`;
}

export function tagsCenter() {
  return `<div class="zr-module-editor-grid is-tags">
    ${panel("Gameplay Tag Registry", `${cluster({ className: "zr-module-filterbar", children: [searchInput("Search tags..."), checkbox("Show Inherited", true), select("View Options")] })}${moduleTable(["Tag", "Namespace", "References", "Status", "Source"], [
      { cells: ["Ability.Activate", "Game", "128", tag("Valid", "green"), "DefaultGameplayTags.ini"] },
      { cells: ["Ability.Cancel", "Game", "32", tag("Valid", "green"), "DefaultGameplayTags.ini"] },
      { cells: ["Character.State.Alive", "Game", "68", tag("Valid", "green"), "DefaultGameplayTags.ini"] },
      { cells: ["Character.State.Stunned", "Game", "36", tag("Valid", "green"), "DefaultGameplayTags.ini"], selected: true },
      { cells: ["Character.Type.Player", "Game", "24", tag("Deprecated", "orange"), "DefaultGameplayTags.ini"] },
      { cells: ["Combat.Damage.Physical", "Game", "36", tag("Valid", "green"), "CombatTags.ini"] }
    ], "1.5fr 0.75fr 0.65fr 0.75fr 1.4fr")}`)}
    ${panel("Reference Summary", compactStats([["Direct", "6"], ["Indirect", "30"], ["Owners", "12"], ["Conflicts", "1", "warning"]]))}
  </div>`;
}

export function perceptionCenter() {
  return `<div class="zr-module-editor-grid is-perception">
    ${panel("World Perception Map", perceptionMap())}
    ${panel("Stimuli Channels", compactStats([["Agents", "3"], ["Actors", "4"], ["Events", "6"], ["Lost Sight", "2", "warning"]]))}
    ${panel("Sense Profile", `${settingsRows([
      ["Preset", select("Guard Balanced")],
      ["Sight Radius", input("", { value: "2000.0" })],
      ["Lose Radius", input("", { value: "2500.0" })],
      ["Peripheral Angle", input("", { value: "120deg" })]
    ])}${slider("Max Age", 52, "5.0s")}`)}
  </div>`;
}

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

export function assetCenter() {
  return `<div class="zr-module-editor-grid is-assets">
    ${panel("Content / Environments / Forest", `${cluster({ className: "zr-module-filterbar", children: [select("Type: All"), select("Status: All"), select("Tags: All"), actionButton("Add Filter", "plus"), searchInput("Search Assets")] })}${moduleTable(["", "Name", "Type", "Tags", "Size", "Status", "Modified"], [
      { cells: [checkbox("", false), "Foliage", "Folder", "-", "-", "-", "2026-05-19"] },
      { cells: [checkbox("", true), "SM_Tree_Oak_01", "Static Mesh", `${tag("Nature", "green")} ${tag("Tree", "green")}`, "1.24 MB", tag("Valid", "green"), "2026-05-18 14:32"], selected: true },
      { cells: [checkbox("", false), "SM_Rock_Cliff_01", "Static Mesh", `${tag("Rock", "purple")} ${tag("Cliff", "purple")}`, "2.15 MB", tag("Valid", "green"), "2026-05-18 14:34"] },
      { cells: [checkbox("", false), "T_Forest_Ground_01", "Texture 2D", tag("Ground", "orange"), "4.10 MB", tag("Valid", "green"), "2026-05-18 14:20"] }
    ], "36px 1.4fr 1fr 1.2fr 90px 90px 150px")}`)}
  </div>`;
}

export function vfxCenter() {
  return `<div class="zr-module-editor-grid is-vfx">
    ${panel("Preview", `${previewTile("vfx")}${cluster({ className: "zr-module-playbar", children: [actionIcon("Play", "play"), actionIcon("Pause", "more"), actionIcon("Record", "target"), select("60 fps"), input("", { value: "00:01.23" })] })}`)}
    ${panel("Emitter Stack", graphBoard("vfx", [
      node("Spawn", "Rate / Burst", 16, 34, "green"),
      node("Update", "Force / Curl Noise", 44, 28, "blue"),
      node("Output", "Sprite Renderer", 72, 36, "cyan")
    ], `${graphLink(27, 40, 20, 0)}${graphLink(54, 40, 16, 0)}`))}
    ${panel("Timeline", timeline("vfx"))}
  </div>`;
}

export function hudCenter() {
  return `<div class="zr-module-editor-grid is-hud">
    ${panel("Gameplay_HUD Canvas", hudCanvas())}
    ${panel("Responsive Layout", `${settingsRows([
      ["Device", select("iPhone 15 Pro")],
      ["Aspect", select("19.5:9")],
      ["DPI Scale", input("", { value: "1.00" })],
      ["Safe Zone", checkbox("", true)]
    ])}${compactStats([["Widgets", "18"], ["Bindings", "12"], ["Warnings", "3", "warning"], ["Errors", "1", "warning"]])}`)}
  </div>`;
}

function perceptionMap() {
  return `<div class="zr-module-map is-perception-map">
    <span class="zr-map-wall is-1"></span><span class="zr-map-wall is-2"></span><span class="zr-map-wall is-3"></span><span class="zr-map-wall is-4"></span>
    <button class="zr-map-point is-agent is-1" type="button" data-action="ai-guard-01"><span>AI_Guard_01</span></button>
    <button class="zr-map-point is-agent is-2" type="button" data-action="ai-guard-02"><span>AI_Guard_02</span></button>
    <button class="zr-map-point is-agent is-3" type="button" data-action="ai-guard-03"><span>AI_Guard_03</span></button>
    <button class="zr-map-point is-hostile" type="button" data-action="enemy-01"><span>Enemy_01</span></button>
    <button class="zr-map-point is-sound" type="button" data-action="noise-maker-bp"><span>Noise_Maker_BP</span></button>
    <span class="zr-map-cone is-cyan is-1"></span><span class="zr-map-cone is-cyan is-2"></span><span class="zr-map-cone is-purple"></span>
    <span class="zr-map-path is-1"></span><span class="zr-map-path is-2"></span><span class="zr-map-path is-3"></span>
  </div>`;
}

function hudCanvas() {
  return `<div class="zr-module-hud-canvas">
    <button class="zr-hud-widget is-minimap" type="button" data-action="minimap">Minimap</button>
    <button class="zr-hud-widget is-score-left" type="button" data-action="team-score-left">12</button>
    <button class="zr-hud-widget is-score-right" type="button" data-action="team-score-right">08</button>
    <button class="zr-hud-widget is-weapon is-selected" type="button" data-action="weapon-panel"><strong>30</strong><small>/120</small></button>
    <button class="zr-hud-widget is-status" type="button" data-action="player-status"><strong>Ranger_7</strong><span></span></button>
    <button class="zr-hud-widget is-ability a1" type="button" data-action="ability-slot-1">Q</button>
    <button class="zr-hud-widget is-ability a2" type="button" data-action="ability-slot-2">E</button>
    <button class="zr-hud-widget is-ability a3" type="button" data-action="ability-slot-3">R</button>
    <button class="zr-hud-widget is-action b1" type="button" data-action="sprint">Run</button>
    <button class="zr-hud-widget is-action b2" type="button" data-action="crosshair">Aim</button>
    <span class="zr-hud-crosshair"></span>
  </div>`;
}
