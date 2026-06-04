import { cluster } from "./layout.js";
import { checkbox, input, searchInput, select, tabs, toggle } from "./atoms.js";
import {
  actionButton,
  actionStack,
  assetStrip,
  listRows,
  moduleTree,
  panel,
  panelGroup,
  previewTile,
  segmentButtons,
  settingsRows,
  tag
} from "./module-components.js";

export function sceneLeft() {
  return [
    panel("Hierarchy", `${searchInput("Search scene...")}${moduleTree([
      ["Root", "cube", true, 0],
      ["Environment", "folder", false, 1],
      ["Lighting", "sun", false, 2],
      ["Level Geometry", "grid", false, 1],
      ["Props", "cube", true, 2],
      ["PlayerStart", "target", false, 1]
    ])}`),
    panel("Layers", listRows(["Gameplay", "Environment", "Lighting", "Audio", "Debug"], 0))
  ];
}

export function gameplayLeft() {
  return [
    panel("Effect Tools", `${tabs(["Rules", "Stacking", "Tags"], 0)}${settingsRows([
      ["Duration Policy", select("Has Duration")],
      ["Duration", `${input("", { value: "10.0" })}<small>s</small>`],
      ["Period", `${input("", { value: "1.0" })}<small>s</small>`],
      ["Execute Periodic", checkbox("", false)],
      ["Stacking Type", select("Aggregate by Source")],
      ["Stack Limit Count", input("", { value: "5" })],
      ["Deny Overflow", checkbox("", false)]
    ])}`),
    panel("Tag Requirements", listRows(["Granted Tags", "Blocked Tags", "Source Tags", "Target Tags"], 3, ["1", "0", "0", "2"])),
    panel("Effect Assets", `${searchInput("Search assets...")}${moduleTree([
      ["Gameplay Effects", "folder", false, 0],
      ["GE_HealthRegen", "gear", true, 1],
      ["GE_DamageFire", "gear", false, 1],
      ["GE_Slow", "gear", false, 1],
      ["Curve Tables", "folder", false, 0],
      ["CT_Damage", "grid", false, 1]
    ])}`)
  ];
}

export function abilityLeft() {
  return [
    panel("Ability Task Palette", `${searchInput("Search tasks...")}${listRows(["Activate Ability", "Check State", "Wait Gameplay Event", "Apply Cost", "Apply Cooldown", "Play Montage", "Apply Effect", "End Ability"], 5)}`),
    panel("Ability Assets", `${searchInput("Search assets...")}${moduleTree([
      ["Abilities", "folder", false, 0],
      ["GA_DashAttack", "play", true, 1],
      ["GA_Jump", "play", false, 1],
      ["Animation Montages", "folder", false, 0],
      ["AM_DashAttack_Montage", "history", true, 1],
      ["Effects", "folder", false, 0],
      ["GE_DashAttack_Damage", "gear", false, 1],
      ["GE_DashAttack_Cost", "gear", false, 1]
    ])}`),
    panel("Ability Debug", settingsRows([
      ["Debug Object", select("None")],
      ["Authority", tag("Server", "cyan")],
      ["Prediction", toggle("", true)]
    ]))
  ];
}

export function tagsLeft() {
  return [
    panel("Tag Actions", cluster({ className: "zr-module-card-tools", wrap: true, children: [actionButton("Add", "plus"), actionButton("Rename", "file"), actionButton("Move", "move"), actionButton("Duplicate", "file")] })),
    panel("Validation Filters", settingsRows([
      ["Show Invalid", checkbox("", true)],
      ["Show Deprecated", checkbox("", true)],
      ["Show Redirects", checkbox("", true)],
      ["Show Conflicts", checkbox("", true)],
      ["Show Unused", checkbox("", false)]
    ])),
    panel("Sources", panelGroup("tag-sources", [
      { label: "Sources", active: true, content: moduleTree([
        ["Project", "folder", false, 0],
        ["DefaultGameplayTags.ini", "file", true, 1],
        ["Plugins", "folder", false, 0],
        ["GameplayAbilitiesTags.ini", "file", false, 1],
        ["CombatTags.ini", "file", false, 1],
        ["Native Tag Sets", "folder", false, 0],
        ["CoreGameplayTags.ini", "file", false, 1]
      ]) },
      { label: "Plugins", content: moduleTree([
        ["Plugins", "folder", true, 0],
        ["GameplayAbilitiesTags.ini", "file", true, 1],
        ["CombatTags.ini", "file", false, 1]
      ]) },
      { label: "Native Sets", content: moduleTree([
        ["Native Tag Sets", "folder", true, 0],
        ["CoreGameplayTags.ini", "file", true, 1]
      ]) }
    ], { className: "is-card-panel" }))
  ];
}

export function perceptionLeft() {
  return [
    panel("Sense Tools", `${segmentButtons(["Sight", "Hearing", "Damage", "Team"], 0)}${listRows(["Guard (Balanced)", "Sniper (Long Sight)", "Scout (Wide FOV)", "Berserker (Hearing)"], 0)}`),
    panel("Debug", settingsRows([
      ["Draw Senses", toggle("", true)],
      ["Draw LoS", toggle("", true)],
      ["Draw Hearing", toggle("", true)],
      ["Show Stimuli", toggle("", true)],
      ["Color By Team", toggle("", false)]
    ])),
    panel("AI Assets", `${searchInput("Search...")}${moduleTree([
      ["Perception Configs", "folder", false, 0],
      ["Guard_Perception", "eye", true, 1],
      ["Sniper_Perception", "eye", false, 1],
      ["Stimuli Sources", "folder", false, 0],
      ["GunShot_BP", "audio", false, 1],
      ["Explosion_BP", "sun", false, 1],
      ["Pawn Classes", "folder", false, 0],
      ["AI_Guard", "component", false, 1]
    ])}`)
  ];
}

export function materialLeft() {
  return [
    panel("Node Palette", `${searchInput("Search nodes...")}${listRows(["Texture Sample", "Multiply", "Lerp", "Scalar Parameter", "Vector Parameter", "Roughness"], 0)}`),
    panel("Material Preview", previewTile("material")),
    panel("Assets", `${searchInput("Search assets...")}${moduleTree([
      ["Game/Materials", "folder", false, 0],
      ["Environment", "folder", false, 1],
      ["M_Rock_Cliff", "material", true, 2],
      ["M_Wet_Rock", "material", false, 2],
      ["Functions", "folder", false, 1]
    ])}`)
  ];
}

export function behaviorLeft() {
  return [
    panel("Node Palette", `${searchInput("Search nodes...")}${listRows(["Selector", "Sequence", "Parallel", "Blackboard", "Cooldown", "Distance", "Attack", "Wait"], 0)}`),
    panel("AI Assets", moduleTree([
      ["Blackboards", "folder", false, 0],
      ["BB_Enemy", "grid", true, 1],
      ["Behavior Trees", "folder", false, 0],
      ["BT_Enemy", "component", true, 1],
      ["EQS", "folder", false, 0],
      ["EQS_Enemy_Search", "target", false, 1]
    ]))
  ];
}

export function renderPipelineLeft() {
  return [
    panel("Pass Palette", `${searchInput("Search passes...")}${listRows(["Render Pass", "Compute Pass", "Copy Pass", "Clear Pass", "Shadow Pass", "Lighting Pass", "Reflection Pass", "Bloom Pass", "Tone Map Pass", "Debug Pass"], 5)}`),
    panel("Pipeline Assets", `${searchInput("Search assets...")}${moduleTree([
      ["Pipelines", "folder", false, 0],
      ["MainPipeline.rp", "renderer", true, 1],
      ["MobilePipeline.rp", "renderer", false, 1],
      ["Passes", "folder", false, 0],
      ["Lighting", "folder", false, 1],
      ["PostProcess", "folder", false, 1],
      ["Shaders", "folder", false, 0],
      ["Textures", "folder", false, 0]
    ])}`)
  ];
}

export function assetLeft() {
  return [
    panel("Filters", `${listRows(["All Assets", "Recently Modified", "Checked Out", "Missing References", "Validation Issues"], 0, ["12,347", "142", "8", "23", "19"])}${actionStack(["Import Assets", "Reimport Assets", "Import From Path"])}`),
    panel("Folder Tree", moduleTree([
      ["Nightingale", "folder", false, 0],
      ["Content", "folder", false, 1],
      ["Characters", "folder", false, 2],
      ["Environments", "folder", false, 2],
      ["Forest", "folder", true, 3],
      ["Materials", "folder", false, 2],
      ["VFX", "folder", false, 2]
    ]))
  ];
}

export function vfxLeft() {
  return [
    panel("Emitter Library", `${segmentButtons(["Emitters", "Modules", "Tools"], 0)}${searchInput("Search emitters...")}${listRows(["Point", "Box", "Sphere", "Cylinder", "Mesh", "Force", "Velocity", "Curl Noise"], 0)}`),
    panel("Content Browser", `${searchInput("Search assets...")}${moduleTree([
      ["VFX", "folder", false, 0],
      ["Systems", "folder", false, 1],
      ["P_Bolt_01", "sun", true, 2],
      ["P_RailTrail", "sun", false, 2],
      ["Textures", "folder", false, 1]
    ])}`),
    panel("Source", assetStrip(["T_Bolt_01", "M_Bolt_01", "T_Noise_01"]))
  ];
}

export function hudLeft() {
  return [
    panel("Widget Palette", `${searchInput("Search widgets...")}${listRows(["Text", "Image", "Button", "Progress Bar", "Slider", "Icon", "Container", "System"], 3)}`),
    panel("Responsive Presets", segmentButtons(["Phone", "Tablet", "Desktop", "Console"], 0)),
    panel("UI Assets", panelGroup("hud-assets", [
      { label: "UI Assets", active: true, content: moduleTree([
        ["HUD", "folder", false, 0],
        ["Gameplay_HUD", "image", true, 1],
        ["Vehicle_HUD", "image", false, 1],
        ["Widget Blueprints", "folder", false, 0],
        ["WBP_HealthBar", "component", false, 1],
        ["WBP_AmmoCounter", "component", false, 1],
        ["Style Resources", "folder", false, 0],
        ["Colors", "material", false, 1]
      ]) },
      { label: "Screens", content: moduleTree([
        ["HUD", "folder", true, 0],
        ["Gameplay_HUD", "image", true, 1],
        ["Vehicle_HUD", "image", false, 1]
      ]) }
    ], { className: "is-card-panel" }))
  ];
}
