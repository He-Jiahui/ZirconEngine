import { checkbox, input, searchInput, select, slider, toggle } from "./atoms.js";
import { alerts } from "./collections.js";
import {
  actionButton,
  compactStats,
  listRows,
  moduleTable,
  moduleTree,
  panelGroup,
  previewTile,
  settingsRows,
  tag
} from "./module-components.js";

export function sceneDetails() {
  return panelGroup("scene-right", [
    { label: "Inspector", active: true, content: `${settingsRows([
      ["Object", select("Props")],
      ["Tag", select("Untagged")],
      ["Position", input("", { value: "128.4, 64.2, -32.7" })],
      ["Rotation", input("", { value: "0, 90, 0" })],
      ["Scale", input("", { value: "1, 1, 1" })],
      ["Static", checkbox("", false)]
    ])}${actionButton("Add Component", "plus")}` },
    { label: "History", content: listRows(["Selected Props", "Moved Box_01", "Updated Material", "Saved Scene"], 0) }
  ]);
}

export function gameplayDetails() {
  return panelGroup("gameplay-right", [
    { label: "Effect Hierarchy", active: true, content: `${searchInput("Search hierarchy...")}${moduleTree([
      ["GE_HealthRegen", "gear", true, 0],
      ["Modifiers (4)", "folder", false, 1],
      ["HealthRegen (Additive)", "component", true, 2],
      ["IncomingHealing", "component", false, 2],
      ["Executions (1)", "folder", false, 1],
      ["Clamp Health", "play", false, 2],
      ["Granted Tags", "folder", false, 1],
      ["Regen.Health", "target", false, 2]
    ])}` },
    { label: "Details", content: `${settingsRows([
      ["Attribute", select("Health")],
      ["Modifier Op", select("Additive")],
      ["Magnitude Type", select("Scalable Float")],
      ["Magnitude", input("", { value: "10.0" })],
      ["Snapshot", checkbox("", false)],
      ["Replicate", toggle("", true)]
    ])}${slider("Magnitude Curve", 72, "10.0")}` }
  ]);
}

export function abilityDetails() {
  return panelGroup("ability-right", [
    { label: "Graph Outline", active: true, content: `${searchInput("Search outline...")}${moduleTree([
      ["GA_DashAttack", "play", true, 0],
      ["Phases", "folder", false, 1],
      ["Activation", "target", false, 2],
      ["Tasks", "list", false, 2],
      ["Play Montage", "history", true, 3],
      ["Wait Gameplay Event", "component", false, 3],
      ["Dependencies", "folder", false, 1],
      ["GE_DashAttack_Damage", "gear", false, 2]
    ])}` },
    { label: "Task Properties", content: `${settingsRows([
      ["Task", tag("Play Montage", "cyan")],
      ["Montage", select("AM_DashAttack_Montage")],
      ["Play Rate", input("", { value: "1.0" })],
      ["Start Section", select("Default")],
      ["Targeting Mode", select("Self")],
      ["Prediction Key", select("Use Ability Key")],
      ["Replication", select("Server Initiated")]
    ])}${slider("Blend Weight", 80, "1.0")}` },
    { label: "Validation", content: `${alerts([["success", "Compile succeeded"], ["warning", "Montage has no default slot"], ["info", "Prediction path is simulated"]])}${actionButton("Fix Montage Slot", "check")}` }
  ]);
}

export function tagsDetails() {
  return panelGroup("tags-right", [
    { label: "Hierarchy", active: true, content: `${searchInput("Search hierarchy...")}${moduleTree([
      ["Ability", "folder", false, 0],
      ["Character", "folder", false, 0],
      ["Character.State", "folder", false, 1],
      ["Character.State.Alive", "target", false, 2],
      ["Character.State.Dead", "target", false, 2],
      ["Character.State.Stunned", "target", true, 2],
      ["Combat", "folder", false, 0],
      ["UI", "folder", false, 0]
    ])}${settingsRows([
      ["Tag", input("", { value: "Character.State.Stunned" })],
      ["Namespace", select("Game")],
      ["Source", "DefaultGameplayTags.ini"],
      ["Status", tag("Valid", "green")],
      ["Deprecated", checkbox("", false)]
    ])}` },
    { label: "References", content: moduleTable(["Owner", "Type", "Path"], [
      { cells: ["GA_DashAttack", "Ability", "/Game/Abilities"] },
      { cells: ["BT_Enemy", "AI", "/Game/AI"] },
      { cells: ["WBP_Status", "UI", "/Game/UI"] }
    ], "1fr 0.8fr 1.3fr") },
    { label: "Owners", content: listRows(["DefaultGameplayTags.ini", "CombatTags.ini", "Native Sets"], 0, ["36", "4", "2"]) },
    { label: "Redirects", content: `${alerts([["error", "Redirect conflict from Character.State.Stun"]])}${actionButton("Resolve Redirect", "check")}` }
  ]);
}

export function perceptionDetails() {
  return panelGroup("perception-right", [
    { label: "World Overview", active: true, content: `${listRows(["AI_Guard_01", "AI_Guard_02", "AI_Guard_03"], 0, ["Sight", "Sight", "Hearing"])}${moduleTable(["Time", "Actor", "Sense"], [
      { cells: ["00:12.345", "Noise_Maker_BP", tag("Hearing", "purple")] },
      { cells: ["00:13.104", "Enemy_01", tag("Sight", "cyan")], selected: true },
      { cells: ["00:13.590", "Explosion_BP", tag("Hearing", "purple")] },
      { cells: ["00:14.512", "Enemy_01", tag("Sight", "cyan")] }
    ], "82px 1fr 88px")}` },
    { label: "Sight Details", content: `${settingsRows([
      ["Enabled", toggle("", true)],
      ["Radius", input("", { value: "2000.0" })],
      ["Lose Sight Radius", input("", { value: "2500.0" })],
      ["Age Max", input("", { value: "5.0s" })],
      ["Tick Interval", input("", { value: "0.2s" })],
      ["Detect Enemies", checkbox("", true)],
      ["Detect Neutrals", checkbox("", true)],
      ["Detect Friendlies", checkbox("", false)]
    ])}${slider("Peripheral Angle", 72, "120deg")}` },
    { label: "Filters", content: `${moduleTree([
      ["Target Tags", "folder", false, 0],
      ["Faction.Enemy", "target", true, 1],
      ["Faction.Neutral", "target", false, 1],
      ["Class Filter", "folder", false, 0],
      ["AI_Guard", "component", true, 1]
    ])}${actionButton("Add Filter", "plus")}` }
  ]);
}

export function materialDetails() {
  return panelGroup("material-right", [
    { label: "Graph Outline", active: true, content: `${searchInput("Search...")}${moduleTree([
      ["M_Rock_Cliff", "material", false, 0],
      ["Texture Sample", "image", true, 1],
      ["Moss Mask", "image", false, 1],
      ["Multiply", "component", false, 1],
      ["Lerp", "component", false, 1],
      ["Roughness", "component", false, 1]
    ])}` },
    { label: "Parameters", content: settingsRows([
      ["Tiling", input("", { value: "4.0" })],
      ["Use Moss", checkbox("", true)],
      ["Tint", select("Olive")],
      ["Moss Color", select("Green")],
      ["Roughness", input("", { value: "0.65" })]
    ]) },
    { label: "Node Details", content: settingsRows([
      ["Node Name", input("", { value: "TextureSample_0" })],
      ["Texture", select("T_Rock_Cliff_Albedo")],
      ["Sampler Source", select("From Texture Asset")],
      ["Mip Value Mode", select("None")]
    ]) }
  ]);
}

export function behaviorDetails() {
  return panelGroup("behavior-right", [
    { label: "BT Outline", content: `${searchInput("Search...")}${moduleTree([
      ["ROOT", "target", false, 0],
      ["Selector", "component", true, 1],
      ["Sequence", "list", false, 2],
      ["Chase Target", "target", false, 3],
      ["Attack", "play", true, 3]
    ])}` },
    { label: "Execution", active: true, content: `${listRows(["Selector - Running", "Sequence - Running", "Chase Target - Success", "Attack - Running", "Patrol - Inactive"], 3)}${settingsRows([["Status", tag("Running", "cyan")], ["Elapsed", "1.45s"], ["Last Result", "In Progress"]])}` },
    { label: "Blackboard", content: settingsRows([["TargetActor", "Player_01"], ["LastKnownLocation", "128, 64, -12"], ["CanAttack", checkbox("", true)]]) }
  ]);
}

export function renderPipelineDetails() {
  return panelGroup("render-right", [
    { label: "Passes", active: true, content: `${searchInput("Search...")}${moduleTree([
      ["Frame 1234", "renderer", true, 0],
      ["Setup", "folder", false, 1],
      ["GBuffer", "folder", false, 1],
      ["1 GBuffer Pass", "renderer", false, 2],
      ["Lighting", "folder", false, 1],
      ["2 Lighting Pass", "sun", false, 2],
      ["3 SSR Pass", "renderer", false, 2],
      ["5 Post Process Pass", "renderer", true, 2],
      ["Output", "folder", false, 1],
      ["7 UI Composite Pass", "image", false, 2]
    ])}${settingsRows([
      ["Pass", tag("Post Process Pass (#5)", "purple")],
      ["Pass Type", select("Render Pass")],
      ["Enabled", checkbox("", true)],
      ["SceneColor", select("R11G11B10_FLOAT")],
      ["AO", select("R8_UNORM")],
      ["PostColor", select("R11G11B10_FLOAT")]
    ])}` },
    { label: "Resources", content: moduleTable(["Resource", "Format", "State"], [
      { cells: ["SceneColor", "R11G11B10_FLOAT", tag("Read", "cyan")] },
      { cells: ["PostColor", "R11G11B10_FLOAT", tag("Write", "orange")], selected: true },
      { cells: ["Depth", "D32_FLOAT", tag("Read", "cyan")] }
    ], "1fr 1.2fr 0.8fr") },
    { label: "Frame Stages", content: `${compactStats([["GPU", "0.45 ms"], ["CPU", "0.08 ms"], ["Draws", "42"], ["Bandwidth", "1.28 GB"]])}${actionButton("View in Profiler", "target")}` }
  ]);
}

export function assetDetails() {
  return panelGroup("asset-right", [
    { label: "References", active: true, content: `${moduleTree([
      ["SM_Tree_Oak_01", "cube", true, 0],
      ["Referenced By (5)", "folder", false, 1],
      ["BP_Tree_Oak", "component", false, 2],
      ["Foliage_Oak_Set", "grid", false, 2],
      ["Level_Forest", "globe", false, 2],
      ["Depends On (12)", "folder", false, 1]
    ])}` },
    { label: "Metadata", content: `${settingsRows([
      ["Name", "SM_Tree_Oak_01"],
      ["Type", "Static Mesh"],
      ["Path", "/Game/Environments/Forest"],
      ["Size", "1.24 MB"],
      ["Status", tag("Valid", "green")],
      ["Nanite", tag("Enabled", "green")]
    ])}${previewTile("asset")}` },
    { label: "Preview", content: previewTile("asset") },
    { label: "Issues", content: alerts([["warning", "1 warning"], ["error", "1 invalid collision"]]) }
  ]);
}

export function vfxDetails() {
  return panelGroup("vfx-right", [
    { label: "System Overview", active: true, content: `${moduleTree([
      ["P_Bolt_01", "sun", true, 0],
      ["E_Bolt", "component", true, 1],
      ["E_Bolt_Light", "sun", false, 1],
      ["E_Bolt_Sparks", "sun", false, 1]
    ])}${listRows(["Spawn", "Update", "Post Update", "Render"], 1, ["10", "22", "6", "5"])}` },
    { label: "Stages", content: listRows(["Stage 0 Spawn", "Stage 1 Update", "Stage 2 Post Update", "Stage 3 Render"], 1) },
    { label: "Details", content: `${settingsRows([
      ["Curl Noise", checkbox("", true)],
      ["Noise Strength", input("", { value: "75.0" })],
      ["Frequency", input("", { value: "2.5" })],
      ["Octaves", select("3")],
      ["Noise Type", select("Curl")],
      ["Space", select("World")]
    ])}${slider("Mask", 68, "None")}` },
    { label: "Compile", content: alerts([["success", "E_Bolt compile success"], ["warning", "Warnings (2)"], ["info", "Infos (3)"]]) }
  ]);
}

export function hudDetails() {
  return panelGroup("hud-right", [
    { label: "Widget Hierarchy", active: true, content: `${searchInput("Search hierarchy...")}${moduleTree([
      ["Gameplay_HUD (Screen)", "image", true, 0],
      ["Canvas Panel", "columns", false, 1],
      ["TopBar", "folder", false, 1],
      ["TeamScore_Left", "component", false, 2],
      ["MatchTimer", "component", false, 2],
      ["Minimap", "image", false, 1],
      ["WeaponPanel", "component", true, 1],
      ["Weapon_Icon", "image", false, 2],
      ["Ammo_Clip", "component", false, 2],
      ["AbilityBar", "folder", false, 1]
    ])}` },
    { label: "Inspector", content: `${settingsRows([
      ["Widget", tag("WeaponPanel", "cyan")],
      ["Is Variable", checkbox("", true)],
      ["Visible", checkbox("", true)],
      ["Opacity", select("100%")],
      ["Render Layer", input("", { value: "0" })],
      ["Tooltip", input("Enter text...")]
    ])}${slider("Scale", 62, "1.00")}` },
    { label: "Bindings", content: moduleTable(["Property", "Binding", "Status"], [
      { cells: ["Ammo_Clip", "GetCurrentAmmo", tag("OK", "green")] },
      { cells: ["Ammo_Reserve", "GetReserveAmmo", tag("Missing", "orange")], selected: true },
      { cells: ["HealthBar", "GetHealthRatio", tag("OK", "green")] }
    ], "1fr 1.3fr 88px") }
  ]);
}
