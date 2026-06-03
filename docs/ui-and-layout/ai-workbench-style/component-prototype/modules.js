import { icon } from "./icons.js";
import { cluster, grid } from "./layout.js";
import { checkbox, input, searchInput, select, slider, tabs, toggle } from "./atoms.js";
import { alerts } from "./collections.js";
import { buildExtensionModules } from "./extension-modules.js";
import {
  actionButton,
  actionIcon,
  actionStack,
  assetStrip,
  bottomOutput,
  compactStats,
  curvePanel,
  graphBoard,
  graphLink,
  listRows,
  moduleLeft,
  moduleMain,
  moduleRight,
  moduleTable,
  moduleTree,
  node,
  panel,
  panelTabs,
  panelView,
  previewTile,
  progress,
  segmentButtons,
  settingsRows,
  tag,
  timeline
} from "./module-components.js";

const esc = (value) => String(value ?? "").replace(/[&<>"']/g, (char) => ({
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#039;"
}[char]));

export const defaultModuleId = "gameplay-effect";

const coreModules = [
  {
    id: "scene",
    label: "Scene",
    icon: "cube",
    status: "Scene workbench ready",
    actions: [
      ["save", "Save"],
      ["folder", "Browse"],
      ["grid", "Snap"],
      ["play", "Preview"]
    ],
    left: () => [
      panel("Hierarchy", `${searchInput("Search scene...")}${moduleTree([
        ["Root", "cube", true, 0],
        ["Environment", "folder", false, 1],
        ["Lighting", "sun", false, 2],
        ["Level Geometry", "grid", false, 1],
        ["Props", "cube", true, 2],
        ["PlayerStart", "target", false, 1]
      ])}`),
      panel("Layers", listRows(["Gameplay", "Environment", "Lighting", "Audio", "Debug"], 0))
    ],
    center: () => sceneCenter(),
    right: () => sceneDetails(),
    bottom: () => bottomOutput("scene", ["Selection", "Console", "Validation"], sceneBottom())
  },
  {
    id: "gameplay-effect",
    label: "Gameplay Effect",
    shortLabel: "Effect",
    icon: "component",
    status: "GE_HealthRegen selected",
    actions: [
      ["save", "Save"],
      ["folder", "Browse"],
      ["check", "Compile"],
      ["history", "Diff"],
      ["play", "Simulation"]
    ],
    left: () => [
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
    ],
    center: () => gameplayCenter(),
    right: () => gameplayDetails(),
    bottom: () => bottomOutput("gameplay-effect", ["Simulation Output", "Attribute Delta", "Validation", "Compile Log"], gameplayBottom())
  },
  {
    id: "gameplay-ability",
    label: "Gameplay Ability",
    shortLabel: "Ability",
    icon: "play",
    status: "GA_DashAttack ability graph open",
    actions: [
      ["save", "Save"],
      ["check", "Compile Ability"],
      ["history", "Diff"],
      ["search", "Find"],
      ["play", "Playtest"]
    ],
    left: () => [
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
    ],
    center: () => abilityCenter(),
    right: () => abilityDetails(),
    bottom: () => bottomOutput("gameplay-ability", ["Timeline", "Compile Log", "Gameplay Event Log", "Simulation Console"], abilityBottom())
  },
  {
    id: "gameplay-tags",
    label: "Gameplay Tags",
    shortLabel: "Tags",
    icon: "target",
    status: "Character.State.Stunned selected",
    actions: [
      ["plus", "Add Tag"],
      ["file", "Rename"],
      ["move", "Move"],
      ["trash", "Delete"],
      ["check", "Validate Tags"]
    ],
    left: () => [
      panel("Tag Actions", cluster({ className: "zr-module-card-tools", wrap: true, children: [actionButton("Add", "plus"), actionButton("Rename", "file"), actionButton("Move", "move"), actionButton("Duplicate", "file")] })),
      panel("Validation Filters", settingsRows([
        ["Show Invalid", checkbox("", true)],
        ["Show Deprecated", checkbox("", true)],
        ["Show Redirects", checkbox("", true)],
        ["Show Conflicts", checkbox("", true)],
        ["Show Unused", checkbox("", false)]
      ])),
      panel("Sources", `${panelTabs(["Sources", "Plugins", "Native Sets"], 0, "tag-sources")}${moduleTree([
        ["Project", "folder", false, 0],
        ["DefaultGameplayTags.ini", "file", true, 1],
        ["Plugins", "folder", false, 0],
        ["GameplayAbilitiesTags.ini", "file", false, 1],
        ["CombatTags.ini", "file", false, 1],
        ["Native Tag Sets", "folder", false, 0],
        ["CoreGameplayTags.ini", "file", false, 1]
      ])}`)
    ],
    center: () => tagsCenter(),
    right: () => tagsDetails(),
    bottom: () => bottomOutput("gameplay-tags", ["Validation Log", "Reference Scan", "Migration Preview", "Compile Log"], tagsBottom())
  },
  {
    id: "ai-perception",
    label: "AI Perception",
    shortLabel: "Perception",
    icon: "eye",
    status: "Guard_Perception drawing sight and hearing stimuli",
    actions: [
      ["play", "Simulate Perception"],
      ["target", "Focus"],
      ["grid", "2D View"],
      ["cube", "3D View"],
      ["check", "Validate Query"]
    ],
    left: () => [
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
    ],
    center: () => perceptionCenter(),
    right: () => perceptionDetails(),
    bottom: () => bottomOutput("ai-perception", ["Perception Timeline", "Debug Log", "Query Output", "Validation", "Compile Log"], perceptionBottom())
  },
  {
    id: "material",
    label: "Material",
    icon: "material",
    status: "M_Rock_Cliff graph open",
    actions: [
      ["save", "Save"],
      ["undo", "Undo"],
      ["check", "Compile"],
      ["play", "Preview"],
      ["cube", "Build"]
    ],
    left: () => [
      panel("Node Palette", `${searchInput("Search nodes...")}${listRows(["Texture Sample", "Multiply", "Lerp", "Scalar Parameter", "Vector Parameter", "Roughness"], 0)}`),
      panel("Material Preview", previewTile("material")),
      panel("Assets", `${searchInput("Search assets...")}${moduleTree([
        ["Game/Materials", "folder", false, 0],
        ["Environment", "folder", false, 1],
        ["M_Rock_Cliff", "material", true, 2],
        ["M_Wet_Rock", "material", false, 2],
        ["Functions", "folder", false, 1]
      ])}`)
    ],
    center: () => materialCenter(),
    right: () => materialDetails(),
    bottom: () => bottomOutput("material", ["Shader Output", "Preview Variants", "Warnings"], materialBottom())
  },
  {
    id: "behavior-tree",
    label: "Behavior Tree",
    shortLabel: "Behavior",
    icon: "component",
    status: "BT_Enemy running in preview",
    actions: [
      ["save", "Save"],
      ["undo", "Undo"],
      ["play", "Play"],
      ["target", "Debug"],
      ["check", "Validate"]
    ],
    left: () => [
      panel("Node Palette", `${searchInput("Search nodes...")}${listRows(["Selector", "Sequence", "Parallel", "Blackboard", "Cooldown", "Distance", "Attack", "Wait"], 0)}`),
      panel("AI Assets", moduleTree([
        ["Blackboards", "folder", false, 0],
        ["BB_Enemy", "grid", true, 1],
        ["Behavior Trees", "folder", false, 0],
        ["BT_Enemy", "component", true, 1],
        ["EQS", "folder", false, 0],
        ["EQS_Enemy_Search", "target", false, 1]
      ]))
    ],
    center: () => behaviorCenter(),
    right: () => behaviorDetails(),
    bottom: () => bottomOutput("behavior-tree", ["AI Debug Log", "Runtime Trace", "Breakpoint Output", "Validation Issues"], behaviorBottom())
  },
  {
    id: "render-pipeline",
    label: "Render Pipeline",
    shortLabel: "Render",
    icon: "renderer",
    status: "Frame 1234 render graph captured",
    actions: [
      ["save", "Save"],
      ["undo", "Undo"],
      ["check", "Compile Pipeline"],
      ["play", "Preview Frame"],
      ["cube", "Build Frame"]
    ],
    left: () => [
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
    ],
    center: () => renderPipelineCenter(),
    right: () => renderPipelineDetails(),
    bottom: () => bottomOutput("render-pipeline", ["Frame Capture Log", "Compile Output", "Resource Transitions", "Warnings", "Errors", "Compile Log"], renderPipelineBottom())
  },
  {
    id: "asset-browser",
    label: "Asset Browser",
    shortLabel: "Assets",
    icon: "image",
    status: "SM_Tree_Oak_01 selected",
    actions: [
      ["save", "Save All"],
      ["folder", "Import"],
      ["history", "Reimport"],
      ["check", "Validate"],
      ["cube", "Build"]
    ],
    left: () => [
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
    ],
    center: () => assetCenter(),
    right: () => assetDetails(),
    bottom: () => bottomOutput("asset-browser", ["Queue", "Output", "Validation", "Cook", "Package"], assetBottom())
  },
  {
    id: "vfx",
    label: "VFX",
    icon: "sun",
    status: "P_Bolt_01 previewing at 60 fps",
    actions: [
      ["save", "Save"],
      ["save", "Save All"],
      ["undo", "Undo"],
      ["play", "Simulate"],
      ["check", "Compile"]
    ],
    left: () => [
      panel("Emitter Library", `${segmentButtons(["Emitters", "Modules", "Tools"], 0)}${searchInput("Search emitters...")}${listRows(["Point", "Box", "Sphere", "Cylinder", "Mesh", "Force", "Velocity", "Curl Noise"], 0)}`),
      panel("Content Browser", `${searchInput("Search assets...")}${moduleTree([
        ["VFX", "folder", false, 0],
        ["Systems", "folder", false, 1],
        ["P_Bolt_01", "sun", true, 2],
        ["P_RailTrail", "sun", false, 2],
        ["Textures", "folder", false, 1]
      ])}`),
      panel("Source", assetStrip(["T_Bolt_01", "M_Bolt_01", "T_Noise_01"]))
    ],
    center: () => vfxCenter(),
    right: () => vfxDetails(),
    bottom: () => bottomOutput("vfx", ["Timeline", "Curves", "Niagara Log", "Compile Output", "Event Log"], vfxBottom())
  },
  {
    id: "hud-editor",
    label: "HUD Editor",
    shortLabel: "HUD",
    icon: "image",
    status: "WeaponPanel selected in Gameplay_HUD",
    actions: [
      ["save", "Save All"],
      ["undo", "Undo"],
      ["play", "Preview HUD"],
      ["check", "Validate UI"],
      ["cube", "Build UI"]
    ],
    left: () => [
      panel("Widget Palette", `${searchInput("Search widgets...")}${listRows(["Text", "Image", "Button", "Progress Bar", "Slider", "Icon", "Container", "System"], 3)}`),
      panel("Responsive Presets", segmentButtons(["Phone", "Tablet", "Desktop", "Console"], 0)),
      panel("UI Assets", `${panelTabs(["UI Assets", "Screens"], 0, "hud-assets")}${moduleTree([
        ["HUD", "folder", false, 0],
        ["Gameplay_HUD", "image", true, 1],
        ["Vehicle_HUD", "image", false, 1],
        ["Widget Blueprints", "folder", false, 0],
        ["WBP_HealthBar", "component", false, 1],
        ["WBP_AmmoCounter", "component", false, 1],
        ["Style Resources", "folder", false, 0],
        ["Colors", "material", false, 1]
      ])}`)
    ],
    center: () => hudCenter(),
    right: () => hudDetails(),
    bottom: () => bottomOutput("hud-editor", ["Validation", "Binding Errors", "Preview Log", "Performance", "Compile Log"], hudBottom())
  }
];

const { editorLibraryModule, extensionModules } = buildExtensionModules(coreModules, defaultModuleId);

export const nativeModules = coreModules;
export const webModuleTabs = [...coreModules, editorLibraryModule];
export const modules = [...coreModules, editorLibraryModule, ...extensionModules];
export { extensionModules };

export function moduleById(id) {
  return modules.find((module) => module.id === id) ?? modules.find((module) => module.id === defaultModuleId);
}

export function moduleTabs(activeId = defaultModuleId) {
  return `<nav class="zr-module-tabs" aria-label="Editor modules">${webModuleTabs.map((module) => {
    const active = module.id === activeId || (module.id === "editor-library" && moduleById(activeId).extension);
    return `<button class="zr-module-tab ${active ? "is-active" : ""}" type="button" data-module="${esc(module.id)}" aria-selected="${active ? "true" : "false"}">${icon(module.icon)}<span>${esc(module.shortLabel ?? module.label)}</span></button>`;
  }).join("")}</nav>`;
}

export function moduleToolbar(activeId = defaultModuleId) {
  const module = moduleById(activeId);
  return `<div class="zr-module-toolbar" data-action-group="module-toolbar">${module.actions.map(([glyph, label], index) => (
    actionButton(label, glyph, { active: index === 2 && label === "Compile" })
  )).join("")}</div>`;
}

export function moduleRail(activeId = defaultModuleId) {
  return `<nav class="zr-rail">${webModuleTabs.map((module) => {
    const active = module.id === activeId || (module.id === "editor-library" && moduleById(activeId).extension);
    return `<button class="zr-icon-button zr-rail-module ${active ? "is-active" : ""}" type="button" title="${esc(module.label)}" aria-label="${esc(module.label)}" data-module="${esc(module.id)}">${icon(module.icon)}</button>`;
  }).join("")}<span class="zr-rail-spacer"></span>${actionIcon("Settings", "gear")}${actionIcon("Help", "help")}</nav>`;
}

export function moduleWorkspace(activeId = defaultModuleId) {
  const module = moduleById(activeId);
  return `${moduleLeft(module)}${moduleMain(module)}${moduleRight(module)}${module.bottom()}`;
}

function sceneCenter() {
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

function sceneDetails() {
  return `${panelTabs(["Inspector", "History"], 0, "scene-right")}
    ${panelView("scene-right", "inspector", true, `${settingsRows([
      ["Object", select("Props")],
      ["Tag", select("Untagged")],
      ["Position", input("", { value: "128.4, 64.2, -32.7" })],
      ["Rotation", input("", { value: "0, 90, 0" })],
      ["Scale", input("", { value: "1, 1, 1" })],
      ["Static", checkbox("", false)]
    ])}${actionButton("Add Component", "plus")}`)}
    ${panelView("scene-right", "history", false, listRows(["Selected Props", "Moved Box_01", "Updated Material", "Saved Scene"], 0))}`;
}

function sceneBottom() {
  return `${moduleTable(["Name", "Type", "Size", "Modified"], [
    { cells: ["Item_01", "Mesh", "2.4 MB", "2m ago"] },
    { cells: ["Item_02", "Material", "512 KB", "10m ago"], selected: true },
    { cells: ["Item_03", "Texture", "1.20 MB", "1m ago"] }
  ], "minmax(120px,1.2fr) 110px 90px 120px")}${alerts([["info", "Scene selection ready"], ["success", "No runtime errors"], ["warning", "2 layout warnings"]])}`;
}

function gameplayCenter() {
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

function gameplayDetails() {
  return `${panelTabs(["Effect Hierarchy", "Details"], 0, "gameplay-right")}
    ${panelView("gameplay-right", "effect-hierarchy", true, `${searchInput("Search hierarchy...")}${moduleTree([
      ["GE_HealthRegen", "gear", true, 0],
      ["Modifiers (4)", "folder", false, 1],
      ["HealthRegen (Additive)", "component", true, 2],
      ["IncomingHealing", "component", false, 2],
      ["Executions (1)", "folder", false, 1],
      ["Clamp Health", "play", false, 2],
      ["Granted Tags", "folder", false, 1],
      ["Regen.Health", "target", false, 2]
    ])}`)}
    ${panelView("gameplay-right", "details", false, `${settingsRows([
      ["Attribute", select("Health")],
      ["Modifier Op", select("Additive")],
      ["Magnitude Type", select("Scalable Float")],
      ["Magnitude", input("", { value: "10.0" })],
      ["Snapshot", checkbox("", false)],
      ["Replicate", toggle("", true)]
    ])}${slider("Magnitude Curve", 72, "10.0")}`)}`;
}

function gameplayBottom() {
  return `<div class="zr-module-output-grid">
    ${settingsRows([["Instigator", select("Player_01")], ["Target", select("Player_01")], ["Level", input("", { value: "1" })], ["Duration", input("", { value: "10.0" })]])}
    ${moduleTable(["Time", "Event", "Attribute", "Base", "Delta", "Final", "Source"], [
      { cells: ["0.00", "Apply GE", "Health", "100.00", "+0.00", "100.00", "Player_01"] },
      { cells: ["1.00", "Periodic Exec", "Health", "100.00", "+10.00", "110.00", "Player_01"], selected: true },
      { cells: ["2.00", "Periodic Exec", "Health", "110.00", "+10.00", "120.00", "Player_01"] }
    ], "70px 1.1fr 90px 70px 70px 70px 1fr")}
    ${alerts([["success", "All Good"], ["warning", "0 Warnings"], ["info", "Compile successful"]])}
  </div>`;
}

function abilityCenter() {
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

function abilityDetails() {
  return `${panelTabs(["Graph Outline", "Task Properties", "Validation"], 0, "ability-right")}
    ${panelView("ability-right", "graph-outline", true, `${searchInput("Search outline...")}${moduleTree([
      ["GA_DashAttack", "play", true, 0],
      ["Phases", "folder", false, 1],
      ["Activation", "target", false, 2],
      ["Tasks", "list", false, 2],
      ["Play Montage", "history", true, 3],
      ["Wait Gameplay Event", "component", false, 3],
      ["Dependencies", "folder", false, 1],
      ["GE_DashAttack_Damage", "gear", false, 2]
    ])}`)}
    ${panelView("ability-right", "task-properties", false, `${settingsRows([
      ["Task", tag("Play Montage", "cyan")],
      ["Montage", select("AM_DashAttack_Montage")],
      ["Play Rate", input("", { value: "1.0" })],
      ["Start Section", select("Default")],
      ["Targeting Mode", select("Self")],
      ["Prediction Key", select("Use Ability Key")],
      ["Replication", select("Server Initiated")]
    ])}${slider("Blend Weight", 80, "1.0")}`)}
    ${panelView("ability-right", "validation", false, `${alerts([["success", "Compile succeeded"], ["warning", "Montage has no default slot"], ["info", "Prediction path is simulated"]])}${actionButton("Fix Montage Slot", "check")}`)}`;
}

function abilityBottom() {
  return `<div class="zr-module-output-grid">
    ${settingsRows([["Speed", select("1.0x")], ["Duration", input("", { value: "4.00s" })], ["Playhead", input("", { value: "1.25s" })]])}
    ${timeline("ability")}
    ${moduleTable(["Time", "Event", "Result", "Asset"], [
      { cells: ["1.22s", "Ability Activated", tag("OK", "green"), "GA_DashAttack"], selected: true },
      { cells: ["1.30s", "Cost Applied", tag("OK", "green"), "GE_DashAttack_Cost"] },
      { cells: ["2.45s", "Gameplay Event", tag("Received", "cyan"), "Event.Data.Hit"] },
      { cells: ["2.48s", "Effect Applied", tag("OK", "green"), "GE_DashAttack_Damage"] }
    ], "70px 1.2fr 92px 1.4fr")}
  </div>`;
}

function tagsCenter() {
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

function tagsDetails() {
  return `${panelTabs(["Hierarchy", "References", "Owners", "Redirects"], 0, "tags-right")}
    ${panelView("tags-right", "hierarchy", true, `${searchInput("Search hierarchy...")}${moduleTree([
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
    ])}`)}
    ${panelView("tags-right", "references", false, moduleTable(["Owner", "Type", "Path"], [
      { cells: ["GA_DashAttack", "Ability", "/Game/Abilities"] },
      { cells: ["BT_Enemy", "AI", "/Game/AI"] },
      { cells: ["WBP_Status", "UI", "/Game/UI"] }
    ], "1fr 0.8fr 1.3fr"))}
    ${panelView("tags-right", "owners", false, listRows(["DefaultGameplayTags.ini", "CombatTags.ini", "Native Sets"], 0, ["36", "4", "2"]))}
    ${panelView("tags-right", "redirects", false, `${alerts([["error", "Redirect conflict from Character.State.Stun"]])}${actionButton("Resolve Redirect", "check")}`)}`;
}

function tagsBottom() {
  return `<div class="zr-module-output-grid">
    ${alerts([["error", "2 errors"], ["warning", "8 warnings"], ["info", "6 infos"]])}
    ${moduleTable(["Severity", "Tag", "Message", "Source"], [
      { cells: [tag("Error", "orange"), "Character.State.Stunned", "Redirect conflict: also redirected from Character.State.Stun", "DefaultGameplayTags.ini:42"], selected: true },
      { cells: [tag("Error", "orange"), "Ability.Unknown", "Invalid tag name", "DefaultGameplayTags.ini:113"] },
      { cells: [tag("Warning", "orange"), "Character.Type", "Deprecated tag used 62 times", "DefaultGameplayTags.ini:78"] },
      { cells: [tag("Info", "blue"), "Combat.Heal", "Tag is valid", "CombatTags.ini:55"] }
    ], "98px 1fr 2fr 1.2fr")}
    ${settingsRows([["Export", select("CSV")], ["Filter", select("All")], ["Auto Fix", toggle("", false)]])}
  </div>`;
}

function perceptionCenter() {
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

function perceptionDetails() {
  return `${panelTabs(["World Overview", "Sight Details", "Filters"], 0, "perception-right")}
    ${panelView("perception-right", "world-overview", true, `${listRows(["AI_Guard_01", "AI_Guard_02", "AI_Guard_03"], 0, ["Sight", "Sight", "Hearing"])}${moduleTable(["Time", "Actor", "Sense"], [
      { cells: ["00:12.345", "Noise_Maker_BP", tag("Hearing", "purple")] },
      { cells: ["00:13.104", "Enemy_01", tag("Sight", "cyan")], selected: true },
      { cells: ["00:13.590", "Explosion_BP", tag("Hearing", "purple")] },
      { cells: ["00:14.512", "Enemy_01", tag("Sight", "cyan")] }
    ], "82px 1fr 88px")}`)}
    ${panelView("perception-right", "sight-details", false, `${settingsRows([
      ["Enabled", toggle("", true)],
      ["Radius", input("", { value: "2000.0" })],
      ["Lose Sight Radius", input("", { value: "2500.0" })],
      ["Age Max", input("", { value: "5.0s" })],
      ["Tick Interval", input("", { value: "0.2s" })],
      ["Detect Enemies", checkbox("", true)],
      ["Detect Neutrals", checkbox("", true)],
      ["Detect Friendlies", checkbox("", false)]
    ])}${slider("Peripheral Angle", 72, "120deg")}`)}
    ${panelView("perception-right", "filters", false, `${moduleTree([
      ["Target Tags", "folder", false, 0],
      ["Faction.Enemy", "target", true, 1],
      ["Faction.Neutral", "target", false, 1],
      ["Class Filter", "folder", false, 0],
      ["AI_Guard", "component", true, 1]
    ])}${actionButton("Add Filter", "plus")}`)}`;
}

function perceptionBottom() {
  return `<div class="zr-module-output-grid">
    ${settingsRows([["Agents", select("All Agents")], ["Show Lost", checkbox("", true)], ["Speed", select("1.0x")]])}
    ${timeline("perception")}
    ${moduleTable(["Time", "Agent", "Event", "Sense"], [
      { cells: ["00:11.8", "AI_Guard_01", "Hearing stimulus", tag("Hearing", "purple")] },
      { cells: ["00:13.1", "AI_Guard_02", "Enemy_01 seen", tag("Sight", "cyan")], selected: true },
      { cells: ["00:14.0", "AI_Guard_02", "Lost sight", tag("Warning", "orange")] }
    ], "80px 1fr 1.4fr 92px")}
  </div>`;
}

function materialCenter() {
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

function materialDetails() {
  return `${panelTabs(["Graph Outline", "Parameters", "Node Details"], 0, "material-right")}
    ${panelView("material-right", "graph-outline", true, `${searchInput("Search...")}${moduleTree([
      ["M_Rock_Cliff", "material", false, 0],
      ["Texture Sample", "image", true, 1],
      ["Moss Mask", "image", false, 1],
      ["Multiply", "component", false, 1],
      ["Lerp", "component", false, 1],
      ["Roughness", "component", false, 1]
    ])}`)}
    ${panelView("material-right", "parameters", false, settingsRows([
      ["Tiling", input("", { value: "4.0" })],
      ["Use Moss", checkbox("", true)],
      ["Tint", select("Olive")],
      ["Moss Color", select("Green")],
      ["Roughness", input("", { value: "0.65" })]
    ]))}
    ${panelView("material-right", "node-details", false, settingsRows([
      ["Node Name", input("", { value: "TextureSample_0" })],
      ["Texture", select("T_Rock_Cliff_Albedo")],
      ["Sampler Source", select("From Texture Asset")],
      ["Mip Value Mode", select("None")]
    ]))}`;
}

function materialBottom() {
  return `<div class="zr-module-output-grid">
    <div class="zr-module-log"><p>[SM5] M_Rock_Cliff: Compiling...</p><p>[SM5] 5 instructions / 2 texture samplers</p><p class="is-success">[SM5] Compile successful</p></div>
    ${assetStrip(["Default", "Wet", "Snowy", "Mossy", "Night"])}
    ${alerts([["warning", "Texture sample uses default sampler"], ["warning", "Consider a packed texture"]])}
  </div>`;
}

function behaviorCenter() {
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

function behaviorDetails() {
  return `${panelTabs(["BT Outline", "Execution", "Blackboard"], 1, "behavior-right")}
    ${panelView("behavior-right", "bt-outline", false, `${searchInput("Search...")}${moduleTree([
      ["ROOT", "target", false, 0],
      ["Selector", "component", true, 1],
      ["Sequence", "list", false, 2],
      ["Chase Target", "target", false, 3],
      ["Attack", "play", true, 3]
    ])}`)}
    ${panelView("behavior-right", "execution", true, `${listRows(["Selector - Running", "Sequence - Running", "Chase Target - Success", "Attack - Running", "Patrol - Inactive"], 3)}${settingsRows([["Status", tag("Running", "cyan")], ["Elapsed", "1.45s"], ["Last Result", "In Progress"]])}`)}
    ${panelView("behavior-right", "blackboard", false, settingsRows([["TargetActor", "Player_01"], ["LastKnownLocation", "128, 64, -12"], ["CanAttack", checkbox("", true)]]))}`;
}

function behaviorBottom() {
  return `<div class="zr-module-log is-debug"><p><span></span>[12:10.123] [BT_Enemy] Selector (1) - Running</p><p><span></span>[12:10.124] [BT_Enemy] Sequence (2) - Running</p><p class="is-success"><span></span>[12:50.125] Chase Target (3) - Success</p><p class="is-warning"><span></span>[12:45.230] Attack (4) - Running</p></div>`;
}

function renderPipelineCenter() {
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

function renderPipelineDetails() {
  return `${panelTabs(["Passes", "Resources", "Frame Stages"], 0, "render-right")}
    ${panelView("render-right", "passes", true, `${searchInput("Search...")}${moduleTree([
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
    ])}`)}
    ${panelView("render-right", "resources", false, moduleTable(["Resource", "Format", "State"], [
      { cells: ["SceneColor", "R11G11B10_FLOAT", tag("Read", "cyan")] },
      { cells: ["PostColor", "R11G11B10_FLOAT", tag("Write", "orange")], selected: true },
      { cells: ["Depth", "D32_FLOAT", tag("Read", "cyan")] }
    ], "1fr 1.2fr 0.8fr"))}
    ${panelView("render-right", "frame-stages", false, `${compactStats([["GPU", "0.45 ms"], ["CPU", "0.08 ms"], ["Draws", "42"], ["Bandwidth", "1.28 GB"]])}${actionButton("View in Profiler", "target")}`)}`;
}

function renderPipelineBottom() {
  return `<div class="zr-module-output-grid">
    ${settingsRows([["Frame", input("", { value: "1234" })], ["Platform", select("Windows DX12")], ["FPS", select("30 fps")]])}
    ${moduleTable(["Event", "Pass", "Description", "GPU ms"], [
      { cells: [tag("Info", "blue"), "Frame Start", "Frame 1234 captured", "0.000"] },
      { cells: [tag("OK", "green"), "Lighting Pass", "2 Lighting Pass", "1.872"] },
      { cells: [tag("OK", "green"), "Post Process Pass", "5 Post Process Pass", "0.450"], selected: true },
      { cells: [tag("OK", "green"), "UI Composite Pass", "7 UI Composite Pass", "0.184"] }
    ], "82px 1fr 1.8fr 90px")}
    ${alerts([["success", "Pipeline compile succeeded"], ["warning", "3 resource transition warnings"], ["info", "0 errors"]])}
  </div>`;
}

function assetCenter() {
  return `<div class="zr-module-editor-grid is-assets">
    ${panel("Content / Environments / Forest", `${cluster({ className: "zr-module-filterbar", children: [select("Type: All"), select("Status: All"), select("Tags: All"), actionButton("Add Filter", "plus"), searchInput("Search Assets")] })}${moduleTable(["", "Name", "Type", "Tags", "Size", "Status", "Modified"], [
      { cells: [checkbox("", false), "Foliage", "Folder", "-", "-", "-", "2026-05-19"] },
      { cells: [checkbox("", true), "SM_Tree_Oak_01", "Static Mesh", `${tag("Nature", "green")} ${tag("Tree", "green")}`, "1.24 MB", tag("Valid", "green"), "2026-05-18 14:32"], selected: true },
      { cells: [checkbox("", false), "SM_Rock_Cliff_01", "Static Mesh", `${tag("Rock", "purple")} ${tag("Cliff", "purple")}`, "2.15 MB", tag("Valid", "green"), "2026-05-18 14:34"] },
      { cells: [checkbox("", false), "T_Forest_Ground_01", "Texture 2D", tag("Ground", "orange"), "4.10 MB", tag("Valid", "green"), "2026-05-18 14:20"] }
    ], "36px 1.4fr 1fr 1.2fr 90px 90px 150px")}`)}
  </div>`;
}

function assetDetails() {
  return `${panelTabs(["References", "Metadata", "Preview", "Issues"], 0, "asset-right")}
    ${panelView("asset-right", "references", true, `${moduleTree([
      ["SM_Tree_Oak_01", "cube", true, 0],
      ["Referenced By (5)", "folder", false, 1],
      ["BP_Tree_Oak", "component", false, 2],
      ["Foliage_Oak_Set", "grid", false, 2],
      ["Level_Forest", "globe", false, 2],
      ["Depends On (12)", "folder", false, 1]
    ])}`)}
    ${panelView("asset-right", "metadata", false, `${settingsRows([
      ["Name", "SM_Tree_Oak_01"],
      ["Type", "Static Mesh"],
      ["Path", "/Game/Environments/Forest"],
      ["Size", "1.24 MB"],
      ["Status", tag("Valid", "green")],
      ["Nanite", tag("Enabled", "green")]
    ])}${previewTile("asset")}`)}
    ${panelView("asset-right", "preview", false, previewTile("asset"))}
    ${panelView("asset-right", "issues", false, alerts([["warning", "1 warning"], ["error", "1 invalid collision"]]))}`;
}

function assetBottom() {
  return `<div class="zr-module-output-grid">
    ${moduleTable(["ID", "Task", "Path", "Status", "Progress"], [
      { cells: ["IMP-1021", "Import FBX", "/Game/Forest/SM_Cliff_Rock_02.fbx", "Importing", progress(62)] },
      { cells: ["IMP-1022", "Import Textures", "/Game/Textures/T_Forest_Rock_01.*", "Queued", progress(0)] },
      { cells: ["VAL-2041", "Validate Assets", "/Game/Environments/Forest/*", "Queued", progress(0)] }
    ], "76px 140px 1.6fr 100px 130px")}
    <div class="zr-module-log"><p>10:20:11 Import started: SM_Cliff_Rock_02.fbx</p><p class="is-warning">10:20:12 2 warnings</p><p class="is-error">10:20:15 Error: invalid collision</p></div>
  </div>`;
}

function vfxCenter() {
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

function vfxDetails() {
  return `${panelTabs(["System Overview", "Stages", "Details", "Compile"], 0, "vfx-right")}
    ${panelView("vfx-right", "system-overview", true, `${moduleTree([
      ["P_Bolt_01", "sun", true, 0],
      ["E_Bolt", "component", true, 1],
      ["E_Bolt_Light", "sun", false, 1],
      ["E_Bolt_Sparks", "sun", false, 1]
    ])}${listRows(["Spawn", "Update", "Post Update", "Render"], 1, ["10", "22", "6", "5"])}`)}
    ${panelView("vfx-right", "stages", false, listRows(["Stage 0 Spawn", "Stage 1 Update", "Stage 2 Post Update", "Stage 3 Render"], 1))}
    ${panelView("vfx-right", "details", false, `${settingsRows([
      ["Curl Noise", checkbox("", true)],
      ["Noise Strength", input("", { value: "75.0" })],
      ["Frequency", input("", { value: "2.5" })],
      ["Octaves", select("3")],
      ["Noise Type", select("Curl")],
      ["Space", select("World")]
    ])}${slider("Mask", 68, "None")}`)}
    ${panelView("vfx-right", "compile", false, alerts([["success", "E_Bolt compile success"], ["warning", "Warnings (2)"], ["info", "Infos (3)"]]))}`;
}

function vfxBottom() {
  return `<div class="zr-module-output-grid is-vfx-bottom">${timeline("vfx")}${moduleTable(["Time", "System", "Emitter", "Event"], [
    { cells: ["00:00.00", "P_Bolt_01", "E_Bolt", "Activated"] },
    { cells: ["00:00.01", "P_Bolt_01", "E_Bolt", "Spawn Burst 20"] },
    { cells: ["00:00.45", "P_Bolt_01", "E_Bolt", "Collision 15"], selected: true }
  ], "90px 1fr 1fr 1.4fr")}</div>`;
}

function hudCenter() {
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

function hudDetails() {
  return `${panelTabs(["Widget Hierarchy", "Inspector", "Bindings"], 0, "hud-right")}
    ${panelView("hud-right", "widget-hierarchy", true, `${searchInput("Search hierarchy...")}${moduleTree([
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
    ])}`)}
    ${panelView("hud-right", "inspector", false, `${settingsRows([
      ["Widget", tag("WeaponPanel", "cyan")],
      ["Is Variable", checkbox("", true)],
      ["Visible", checkbox("", true)],
      ["Opacity", select("100%")],
      ["Render Layer", input("", { value: "0" })],
      ["Tooltip", input("Enter text...")]
    ])}${slider("Scale", 62, "1.00")}`)}
    ${panelView("hud-right", "bindings", false, moduleTable(["Property", "Binding", "Status"], [
      { cells: ["Ammo_Clip", "GetCurrentAmmo", tag("OK", "green")] },
      { cells: ["Ammo_Reserve", "GetReserveAmmo", tag("Missing", "orange")], selected: true },
      { cells: ["HealthBar", "GetHealthRatio", tag("OK", "green")] }
    ], "1fr 1.3fr 88px"))}`;
}

function hudBottom() {
  return `<div class="zr-module-output-grid">
    ${alerts([["warning", "Text MatchTimer is not localized"], ["error", "Binding AmmoCount could not be resolved"], ["info", "DPI scale set to 1.00"]])}
    ${moduleTable(["Type", "Severity", "Message", "Widget", "Line"], [
      { cells: [icon("warning"), "Warning", "Text 'MatchTimer' is not localized", "MatchTimer", "--"] },
      { cells: [icon("x"), "Error", "Binding 'AmmoCount' could not be resolved", "Ammo_Reserve", "57"], selected: true },
      { cells: [icon("info"), "Info", "Image 'Minimap' has no alt text", "Minimap", "--"] }
    ], "46px 82px 2fr 1fr 64px")}
    ${settingsRows([["Filter", select("All")], ["Clear", actionButton("Clear All", "trash")], ["Auto Preview", toggle("", true)]])}
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
