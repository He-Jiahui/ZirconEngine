import { checkbox, input, searchInput, select } from "./atoms.js";
import { alerts } from "./collections.js";
import { icon } from "./icons.js";
import {
  actionButton,
  assetStrip,
  bottomOutput,
  compactStats,
  curvePanel,
  graphBoard,
  graphLink,
  listRows,
  moduleTable,
  moduleTree,
  node,
  panel,
  panelTabs,
  panelView,
  previewTile,
  progress,
  settingsRows,
  tabKey,
  tag,
  timeline
} from "./module-components.js";
import { extensionReferenceSamples } from "./reference-samples.js";

const esc = (value) => String(value ?? "").replace(/[&<>"']/g, (char) => ({
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#039;"
}[char]));

export const extensionSources = extensionReferenceSamples.map(({ source, category, glyph }) => [
  source,
  category,
  glyph
]);

const recipeByKind = {
  world: {
    detailTabs: ["Brush", "Layers", "Streaming"],
    actions: (subject, shortLabel) => [["plus", `Add ${shortLabel}`], ["grid", `Paint ${shortLabel}`], ["check", `Build ${shortLabel}`], ["play", `Preview ${shortLabel}`]],
    tools: (subject) => ["Sculpt", "Paint Layer", "Spline Tool", "Scatter Mask", `${subject} Preview`, "Streaming Cell"],
    metrics: () => [["Tiles", "64"], ["LOD", "5"], ["Layers", "7"], ["Warnings", "2", "warning"]],
    settings: (subject) => [["Brush", select(`${subject} Brush`)], ["Radius", input("", { value: "512" })], ["Strength", input("", { value: "0.38" })], ["Falloff", select("Smooth")], ["Live Preview", checkbox("", true)]],
    table: (subject) => [[`${subject}_Tile_12_08`, "Loaded", "1.2 ms"], [`${subject}_Tile_12_09`, "Loaded", "1.4 ms"], [`${subject}_Layer_Rock`, "Dirty", "Queued"], [`${subject}_Cell_A`, "Visible", "High"]]
  },
  rendering: {
    detailTabs: ["Passes", "Resources", "Issues"],
    actions: (subject, shortLabel) => [["play", `Preview ${shortLabel}`], ["check", `Compile ${shortLabel}`], ["target", `Capture ${shortLabel}`], ["save", `Save ${shortLabel}`]],
    tools: (subject) => [`${subject} Stack`, "Shader Pass", "Frame Capture", "Resource View", "Permutation Set", "Warnings"],
    metrics: () => [["GPU", "1.28 ms"], ["Passes", "9"], ["Textures", "24"], ["Warnings", "3", "warning"]],
    settings: (subject) => [["Preview", select("SM5")], ["Quality", select("High")], ["Frame", input("", { value: "1234" })], ["Capture Resources", checkbox("", true)], ["Live Compile", checkbox("", true)]],
    table: (subject) => [["GBuffer", "Ready", "0.42 ms"], ["Lighting", "Ready", "0.68 ms"], [subject, "Compiling", "0.18 ms"], ["Post Process", "Warning", "0.31 ms"]]
  },
  animation: {
    detailTabs: ["Tracks", "Curves", "Validation"],
    actions: (subject, shortLabel) => [["play", `Preview ${shortLabel}`], ["plus", `Add ${shortLabel}`], ["target", `Key ${shortLabel}`], ["check", `Validate ${shortLabel}`]],
    tools: (subject) => ["Pose Track", "Notify Track", "Blend Region", "Root Motion", `${subject} Curves`, "Sync Marker"],
    metrics: () => [["Frames", "240"], ["Tracks", "18"], ["Keys", "284"], ["Sync", "OK"]],
    settings: (subject) => [["Clip", select(`${subject}_Main`)], ["Frame Rate", select("60 fps")], ["Work Range", input("", { value: "0100-0240" })], ["Snap", checkbox("", true)], ["Auto Key", checkbox("", false)]],
    table: () => [["Base Pose", "0000-0060", "Ready"], ["Transition", "0060-0120", "Blending"], ["Notify Window", "0120-0160", "Selected"], ["Recovery", "0160-0240", "Ready"]]
  },
  ui: {
    detailTabs: ["Hierarchy", "Bindings", "Accessibility"],
    actions: (subject, shortLabel) => [["plus", `Add ${shortLabel}`], ["play", `Preview ${shortLabel}`], ["check", `Validate ${shortLabel}`], ["save", `Export ${shortLabel}`]],
    tools: (subject) => ["Widget Tree", "Responsive Rules", "Binding Graph", "Token Swatches", `${subject} Preview`, "Accessibility Audit"],
    metrics: () => [["Widgets", "42"], ["Bindings", "18"], ["Breakpoints", "4"], ["Issues", "3", "warning"]],
    settings: (subject) => [["Screen", select(`${subject} Screen`)], ["Breakpoint", select("Desktop")], ["Theme", select("Workbench Dark")], ["Show Bounds", checkbox("", true)], ["Auto Layout", checkbox("", true)]],
    table: () => [["Header", "Container", "Bound"], ["Primary Button", "Action", "Ready"], ["Status Text", "Text", "Warning"], ["Icon Grid", "List", "Ready"]]
  },
  production: {
    detailTabs: ["Queue", "Rules", "History"],
    actions: (subject, shortLabel) => [["play", `Run ${shortLabel}`], ["check", `Validate ${shortLabel}`], ["save", `Publish ${shortLabel}`], ["history", `Review ${shortLabel}`]],
    tools: (subject) => ["Queue", "Validation Gate", "Artifact Set", "Change List", `${subject} Rules`, "Report Output"],
    metrics: () => [["Jobs", "12"], ["Queued", "4"], ["Warnings", "2", "warning"], ["Ready", "8"]],
    settings: (subject) => [["Profile", select(`${subject} Default`)], ["Target", select("Windows")], ["Version", input("", { value: "2026.06" })], ["Strict Mode", checkbox("", true)], ["Archive Output", checkbox("", true)]],
    table: (subject) => [[`${subject}_Validate`, "Running", "62"], [`${subject}_Cook`, "Queued", "0"], [`${subject}_Package`, "Ready", "100"], [`${subject}_Report`, "Queued", "0"]]
  },
  diagnostics: {
    detailTabs: ["Live Log", "Counters", "Report"],
    actions: (subject, shortLabel) => [["search", `Filter ${shortLabel}`], ["trash", `Clear ${shortLabel}`], ["save", `Export ${shortLabel}`], ["check", `Open ${shortLabel}`]],
    tools: (subject) => ["Log Filter", "Counters", "Trace Events", "Warning Buckets", `${subject} Report`, "Session Diff"],
    metrics: () => [["FPS", "58"], ["Warnings", "24", "warning"], ["Errors", "1", "warning"], ["Marks", "82"]],
    settings: (subject) => [["Subsystem", select(subject)], ["Severity", select("Warnings+")], ["Regex", input("filter...")], ["Collapse Repeats", checkbox("", true)], ["Follow Tail", checkbox("", true)]],
    table: () => [["12:10:11", "Renderer", "Warning"], ["12:10:13", "Asset", "Info"], ["12:10:18", "Gameplay", "Warning"], ["12:10:21", "Runtime", "Error"]]
  },
  online: {
    detailTabs: ["Sessions", "Rules", "Telemetry"],
    actions: (subject, shortLabel) => [["play", `Simulate ${shortLabel}`], ["target", `Match ${shortLabel}`], ["check", `Validate ${shortLabel}`], ["save", `Publish ${shortLabel}`]],
    tools: (subject) => ["Queue Rules", "Party State", "Region Map", "Latency Buckets", `${subject} Preview`, "Failure Report"],
    metrics: () => [["Players", "128"], ["Queues", "6"], ["Latency", "42 ms"], ["Failures", "2", "warning"]],
    settings: (subject) => [["Region", select("Auto")], ["Rule Set", select(subject)], ["Max Wait", input("", { value: "90" })], ["Crossplay", checkbox("", true)], ["Backfill", checkbox("", true)]],
    table: () => [["NA-East", "Open", "42 ms"], ["EU-West", "Open", "58 ms"], ["Asia", "Limited", "84 ms"], ["Backfill", "Queued", "12 jobs"]]
  },
  simulation: {
    detailTabs: ["Bodies", "Materials", "Contacts"],
    actions: (subject, shortLabel) => [["plus", `Add ${shortLabel}`], ["check", `Validate ${shortLabel}`], ["grid", `Bake ${shortLabel}`], ["play", `Run ${shortLabel}`]],
    tools: (subject) => ["Body Setup", "Proxy Hull", "Material Pair", "Contact Debug", `${subject} Bake`, "Mass Preview"],
    metrics: () => [["Bodies", "12"], ["Hull Verts", "96"], ["Mass", "48 kg"], ["Errors", "1", "warning"]],
    settings: (subject) => [["Preset", select(subject)], ["Mass", input("", { value: "48.0" })], ["Friction", input("", { value: "0.62" })], ["Hit Events", checkbox("", true)], ["CCD", checkbox("", false)]],
    table: () => [["Hull_00", "Convex", "32 verts"], ["Hull_01", "Box", "8 verts"], ["Hull_02", "Convex", "56 verts"], ["Hull_03", "Invalid", "Non-manifold"]]
  },
  data: {
    detailTabs: ["Rows", "Schema", "Validation"],
    actions: (subject, shortLabel) => [["plus", `Add ${shortLabel}`], ["folder", `Import ${shortLabel}`], ["check", `Validate ${shortLabel}`], ["save", `Save ${shortLabel}`]],
    tools: (subject) => ["Schema", "CSV Import", "Diff Rows", "Validation", `${subject} References`, "Bulk Edit"],
    metrics: () => [["Rows", "128"], ["Columns", "14"], ["Invalid", "2", "warning"], ["Refs", "512"]],
    settings: (subject) => [["Row Name", input("", { value: `${subject}_Primary` })], ["Type", select("Gameplay")], ["Version", input("", { value: "12" })], ["Localized", checkbox("", true)], ["Deprecated", checkbox("", false)]],
    table: (subject) => [[`${subject}_Tier01`, "Ready", "42"], [`${subject}_Tier02`, "Selected", "68"], [`${subject}_Fallback`, "Warning", "25"], [`${subject}_Debug`, "Ready", "58"]]
  },
  gameplay: {
    detailTabs: ["Rules", "State", "Validation"],
    actions: (subject, shortLabel) => [["plus", `Add ${shortLabel}`], ["play", `Simulate ${shortLabel}`], ["target", `Inspect ${shortLabel}`], ["check", `Validate ${shortLabel}`]],
    tools: (subject) => ["Rule Stack", "State Graph", "Tag Filters", "Spawn Probe", `${subject} Preview`, "Conflict Check"],
    metrics: () => [["Rules", "18"], ["States", "12"], ["Refs", "36"], ["Conflicts", "1", "warning"]],
    settings: (subject) => [["Rule Set", select(subject)], ["Authority", select("Server")], ["Seed", input("", { value: "2026" })], ["Live Preview", checkbox("", true)], ["Strict Tags", checkbox("", true)]],
    table: (subject) => [[`${subject}_Rule_A`, "Ready", "High"], [`${subject}_Rule_B`, "Selected", "Medium"], [`${subject}_State_C`, "Queued", "Low"], [`${subject}_Conflict`, "Warning", "Tags"]]
  },
  runtime: {
    detailTabs: ["Slots", "Migration", "Validation"],
    actions: (subject, shortLabel) => [["save", `Save ${shortLabel}`], ["folder", `Load ${shortLabel}`], ["check", `Validate ${shortLabel}`], ["history", `Migrate ${shortLabel}`]],
    tools: (subject) => ["Slot Schema", "Migration Map", "Runtime Probe", "Cloud Sync", `${subject} Diff`, "Corruption Scan"],
    metrics: () => [["Slots", "6"], ["Schemas", "4"], ["Migrations", "2"], ["Warnings", "1", "warning"]],
    settings: (subject) => [["Schema", select(`${subject} v4`)], ["Slot", select("AutoSave_01")], ["Compression", select("LZ4")], ["Cloud Sync", checkbox("", true)], ["Strict Load", checkbox("", true)]],
    table: () => [["AutoSave_01", "Ready", "2.4 MB"], ["Manual_03", "Migrating", "1.8 MB"], ["Cloud_02", "Queued", "4.1 MB"], ["DebugSlot", "Warning", "Old"]]
  },
  vfx: {
    detailTabs: ["Emitters", "Curves", "Compile"],
    actions: (subject, shortLabel) => [["play", `Simulate ${shortLabel}`], ["plus", `Add ${shortLabel}`], ["check", `Compile ${shortLabel}`], ["target", `Capture ${shortLabel}`]],
    tools: (subject) => ["Emitter Stack", "Spawn Rate", "GPU Sim", "Curve Track", `${subject} Preview`, "Bounds Debug"],
    metrics: () => [["Emitters", "5"], ["Particles", "42K"], ["GPU", "0.8 ms"], ["Warnings", "2", "warning"]],
    settings: (subject) => [["Emitter", select(subject)], ["FPS", select("60 fps")], ["Duration", input("", { value: "2.0" })], ["Loop", checkbox("", true)], ["Fixed Bounds", checkbox("", false)]],
    table: () => [["Spawn", "Ready", "120/s"], ["Velocity", "Ready", "Curve"], ["Color", "Selected", "Gradient"], ["GPU Sort", "Warning", "Cost"]]
  },
  default: {
    detailTabs: ["Details", "Rules", "Validation"],
    actions: (subject, shortLabel) => [["search", `Find ${shortLabel}`], ["plus", `Add ${shortLabel}`], ["check", `Validate ${shortLabel}`], ["play", `Preview ${shortLabel}`]],
    tools: (subject) => [`${subject} Overview`, `${subject} Assets`, `${subject} Rules`, `${subject} Preview`, `${subject} Validation`, `${subject} Output`],
    metrics: () => [["Refs", "1"], ["Controls", "24"], ["Panels", "4"], ["Status", "Ready"]],
    settings: (subject) => [["Preset", select(`${subject} Default`)], ["Filter", input("Filter...")], ["Live Preview", checkbox("", true)], ["Auto Validate", checkbox("", true)], ["Density", select("Workbench Compact")]],
    table: (subject) => [[`${subject}_Primary`, "Ready", "Panel"], [`${subject}_Secondary`, "Ready", "Details"], [`${subject}_Validation`, "Queued", "Check"], [`${subject}_Output`, "Idle", "Log"]]
  }
};

export const extensionModuleConfigs = extensionSources.map(([source, category, glyph]) =>
  createReferenceExtensionConfig(source, category, glyph),
);

export function buildExtensionModules(coreModules, defaultModuleId) {
  const extensionModules = extensionModuleConfigs.map(createExtensionModule);
  return {
    editorLibraryModule: createEditorLibraryModule(coreModules, defaultModuleId),
    extensionModules
  };
}

function createReferenceExtensionConfig(source, category, glyph) {
  const id = source.replace(/^ai-|-layout\.png$/g, "");
  const label = id.split("-").map(titleWord).join(" ");
  const shortLabel = label.split(" ").slice(0, 2).join(" ");
  const subject = label.replace(/\s+(Editor|Audit|Layout|Dashboard|Manager)$/i, "");
  const recipe = recipeFor(source, category);
  return {
    id,
    label,
    shortLabel,
    icon: glyph,
    source,
    category,
    layoutKind: recipe.kind,
    status: `${label} reference panel selected`,
    actions: recipe.actions(subject, shortLabel),
    tools: recipe.tools(subject),
    assets: assetsFor(subject, category, glyph),
    metrics: recipe.metrics(subject),
    detailTabs: recipe.detailTabs,
    settings: recipe.settings(subject),
    table: recipe.table(subject)
  };
}

function createEditorLibraryModule(coreModules, defaultModuleId) {
  return {
    id: "editor-library",
    label: "More Editors",
    shortLabel: "More",
    icon: "grid",
    status: "Extended editor module library ready",
    actions: [
      ["search", "Find Editor"],
      ["folder", "Browse References"],
      ["grid", "Core Modules"],
      ["check", "Validate Coverage"]
    ],
    left: () => [
      panel("Reference Groups", referenceGroupsList()),
      panel("Implementation Rule", settingsRows([
        ["Shell", tag("Shared", "cyan")],
        ["Layout", tag("Left / Main / Right / Bottom", "green")],
        ["Style", tag("Workbench Dark", "blue")],
        ["Native Sync", tag("Core 11 only", "orange")]
      ])),
      panel("Core Modules", moduleTree(coreModules.map((module, index) => [module.label, module.icon, index === 1, 0])))
    ],
    center: () => editorLibraryCenter(coreModules, defaultModuleId),
    right: () => editorLibraryDetails(),
    bottom: () => bottomOutput("editor-library", ["Coverage", "Reference Notes", "Routing Log"], editorLibraryBottom())
  };
}

function createExtensionModule(config) {
  return {
    id: config.id,
    label: config.label,
    shortLabel: config.shortLabel,
    icon: config.icon,
    extension: true,
    source: config.source,
    category: config.category,
    layoutKind: config.layoutKind,
    status: config.status,
    actions: [["grid", "More Editors"], ...config.actions],
    left: () => [
      panel("Reference", settingsRows([
        ["AI Sample", tag(config.source.replace(/^ai-|-layout\.png$/g, "").replace(/-/g, " "), "cyan")],
        ["Category", tag(config.category, "blue")],
        ["Layout", tag(`${titleWord(config.layoutKind)} Workbench`, "green")],
        ["Native Contract", tag("Prototype Only", "orange")]
      ])),
      panel("Tools", `${searchInput("Search tools...")}${listRows(config.tools, 0)}`),
      panel("Assets", `${searchInput("Search assets...")}${moduleTree(config.assets)}`)
    ],
    center: () => extensionCenter(config),
    right: () => extensionDetails(config),
    bottom: () => bottomOutput(config.id, ["Output", "Validation", "References"], extensionBottom(config))
  };
}

function editorLibraryCenter(coreModules, defaultModuleId) {
  return `<div class="zr-module-editor-grid is-library">
    ${panel("Extended Editor Modules", `<div class="zr-extension-card-grid">${extensionModuleConfigs.map(extensionModuleCard).join("")}</div>`)}
    ${panel("Core Native-Synced Modules", moduleTable(["Module", "Reference", "Native"], coreModules.map((module) => ({
      cells: [module.label, module.shortLabel ?? module.label, tag("Synced", "green")],
      selected: module.id === defaultModuleId
    })), "1.2fr 1fr 82px"))}
  </div>`;
}

function extensionModuleCard(config) {
  return `<button class="zr-extension-card is-${esc(config.layoutKind)}" type="button" data-module="${esc(config.id)}" data-module-source="extension-library" aria-label="${esc(config.label)}">
    <span class="zr-extension-card-icon">${icon(config.icon)}</span>
    <span class="zr-extension-card-copy">
      <strong>${esc(config.label)}</strong>
      <small>${esc(config.category)} / ${esc(config.source)}</small>
    </span>
    <span class="zr-extension-card-status">${config.metrics.map(([label, value]) => `${esc(label)} ${esc(value)}`).slice(0, 2).join(" | ")}</span>
  </button>`;
}

function editorLibraryDetails() {
  return `${panelTabs(["Catalog", "Coverage", "Routing"], 0, "library-right")}
    ${panelView("library-right", "catalog", true, `${searchInput("Filter modules...")}${moduleTree(extensionModuleConfigs.map((config, index) => [config.label, config.icon, index === 0, 0]))}`)}
    ${panelView("library-right", "coverage", false, `${compactStats([["Core", "11"], ["Extended", String(extensionModuleConfigs.length)], ["AI Refs", String(extensionModuleConfigs.length)], ["Shells", "1"]])}${settingsRows([
      ["Top Tabs", tag("Core + More", "cyan")],
      ["Rail", tag("Core + More", "cyan")],
      ["Extended", tag("Library Cards", "green")],
      ["Native", tag("Core 11", "orange")]
    ])}`)}
    ${panelView("library-right", "routing", false, moduleTable(["Route", "Target", "Mode"], [
      { cells: ["More Editors", "editor-library", tag("Module", "cyan")], selected: true },
      { cells: ["Extension Card", "Selected editor", tag("Module", "green")] },
      { cells: ["Extension Toolbar", "Output / Validation / References", tag("Panel", "blue")] }
    ], "1fr 1fr 86px"))}`;
}

function editorLibraryBottom() {
  return `<div class="zr-module-output-grid">
    ${moduleTable(["Sample", "Prototype Module", "Coverage"], extensionCoverageRows(), "1.4fr 1fr 98px")}
    ${alerts([["success", `${extensionModuleConfigs.length} extended editor cards use the same response path`], ["info", "Native handoff remains scoped to the core 11 modules"], ["warning", "Extended modules are prototype-only until native ZUI surfaces are added"]])}
  </div>`;
}

function extensionCenter(config) {
  return `<div class="zr-module-editor-grid is-extension is-extension-${esc(config.layoutKind)}">
    ${extensionPrimaryPanel(config)}
    ${panel("Controls & Metrics", `${previewTile(config.layoutKind)}${compactStats(config.metrics)}`)}
    ${panel("Reference Rhythm", `${assetStrip(config.tools.slice(0, 6))}${settingsRows([["Source", config.source], ["Category", config.category], ["Response", tag("Click any control", "green")]])}`)}
  </div>`;
}

function extensionPrimaryPanel(config) {
  switch (config.layoutKind) {
    case "animation":
      return panel(`${config.label} Timeline`, `${timeline(config.id)}${curvePanel()}${moduleTable(["Track", "Range", "State"], toRows(config.table), "1fr 0.8fr 0.8fr")}`);
    case "rendering":
      return panel(`${config.label} Render Stack`, `${previewTile("render")}${moduleTable(["Pass", "State", "GPU"], toRows(config.table), "1fr 0.8fr 0.8fr")}`);
    case "ui":
      return panel(`${config.label} Layout Map`, graphBoard("ui-extension", graphNodes(config), extensionLinks()));
    case "production":
      return panel(`${config.label} Queue`, moduleTable(["Job", "State", "Progress"], config.table.map((row, index) => ({
        cells: [row[0], row[1], progress(index === 0 ? 62 : Number(row[2]) || 0)],
        selected: index === 0
      })), "1.2fr 0.8fr 1fr"));
    case "diagnostics":
      return panel(`${config.label} Live Feed`, `${moduleTable(["Time", "Subsystem", "Level"], toRows(config.table), "1fr 1fr 0.8fr")}${alerts([["warning", "Warnings are grouped by subsystem"], ["info", "Click log rows to pin a diagnostic command"]])}`);
    case "online":
    case "data":
    case "runtime":
      return panel(`${config.label} Grid`, moduleTable(["Item", "State", "Value"], toRows(config.table), "1.2fr 0.8fr 0.8fr"));
    default:
      return panel(`${config.label} Workspace`, graphBoard(config.layoutKind, graphNodes(config), extensionLinks()));
  }
}

function extensionDetails(config) {
  const [firstTab, secondTab, thirdTab] = config.detailTabs;
  return `${panelTabs(config.detailTabs, 0, `${config.id}-right`)}
    ${panelView(`${config.id}-right`, tabKey(firstTab), true, `${settingsRows(config.settings)}${actionButton(config.actions[0][1], config.actions[0][0])}`)}
    ${panelView(`${config.id}-right`, tabKey(secondTab), false, moduleTable(["Item", "State", "Value"], toRows(config.table, 1), "1.1fr 0.9fr 0.9fr"))}
    ${panelView(`${config.id}-right`, tabKey(thirdTab), false, `${alerts([["info", `${config.label} follows the shared module panel contract`], ["success", "Visible controls route through prototype feedback"], ["warning", "Native implementation pending"]])}${actionButton("More Editors", "grid")}`)}`;
}

function extensionBottom(config) {
  return `<div class="zr-module-output-grid">
    ${moduleTable(["Item", "State", "Value"], toRows(config.table), "1.1fr 0.9fr 0.9fr")}
    <div class="zr-module-log"><p>${esc(config.label)}: opened from ${esc(config.source)}</p><p class="is-success">Prototype route active and response feedback enabled.</p><p class="is-warning">Native ZUI surface not generated for this extended editor.</p></div>
  </div>`;
}

function referenceGroupsList() {
  const counts = new Map();
  for (const config of extensionModuleConfigs) {
    counts.set(config.category, (counts.get(config.category) ?? 0) + 1);
  }
  const groups = [...counts.entries()].sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]));
  return listRows(groups.map(([label]) => label), 0, groups.map(([, count]) => String(count)));
}

function extensionCoverageRows() {
  const visibleRows = extensionModuleConfigs.slice(0, 10).map((config, index) => ({
    cells: [config.source, config.label, tag("Panel Ready", "green")],
    selected: index === 0
  }));
  const remaining = extensionModuleConfigs.length - visibleRows.length;
  if (remaining > 0) {
    visibleRows.push({
      cells: [`+${remaining} more references`, "Open from cards above", tag("Ready", "cyan")]
    });
  }
  return visibleRows;
}

function toRows(table, selectedIndex = 0) {
  return table.map((row, index) => ({
    cells: row,
    selected: index === selectedIndex
  }));
}

function graphNodes(config) {
  const positions = [[8, 18], [32, 14], [56, 22], [24, 56], [52, 58], [76, 38]];
  return config.tools.slice(0, 6).map((tool, index) => {
    const [x, y] = positions[index];
    return node(tool, index === 0 ? "Selected" : config.category, x, y, ["cyan", "blue", "green", "purple", "orange", "neutral"][index % 6]);
  });
}

function extensionLinks() {
  return `${graphLink(20, 28, 16)}${graphLink(44, 26, 16, 12)}${graphLink(36, 58, 13, -20)}${graphLink(64, 62, 10, -18)}`;
}

function assetsFor(subject, category, glyph) {
  return [
    [category, "folder", false, 0],
    [`${subject} Root`, glyph, true, 1],
    [`${subject} Preset A`, "file", false, 2],
    [`${subject} Preset B`, "file", false, 2],
    ["Shared References", "folder", false, 1],
    ["Workbench Style", "material", false, 2]
  ];
}

function recipeFor(source, category) {
  const key = source.toLowerCase();
  const kind = layoutKindFor(key, category);
  return { kind, ...recipeByKind[kind] };
}

function layoutKindFor(key, category) {
  if (/particle|vfx/.test(key) || category === "VFX") return "vfx";
  if (/world-state|spawn-rules|gameplay/.test(key) || category === "Gameplay") return "gameplay";
  if (/save-data/.test(key) || category === "Runtime") return "runtime";
  if (/data-table/.test(key) || category === "Data") return "data";
  if (/console|diagnostics|performance|telemetry/.test(key) || category === "Diagnostics") return "diagnostics";
  if (/lobby|matchmaking/.test(key) || category === "Online") return "online";
  if (/physics|collision/.test(key) || category === "Simulation") return "simulation";
  if (/navmesh|perception/.test(key) || category === "AI") return "gameplay";
  if (/lighting|post-process|shader|render/.test(key) || category === "Rendering") return "rendering";
  if (/sequencer|montage|animation|blend|motion|pose|retarget|control-rig/.test(key) || category === "Animation" || category === "Cinematic") return "animation";
  if (/ui-|accessibility|font|icon|menu-flow/.test(key) || category === "UI/UX") return "ui";
  if (/automation|build-export|plugin|project-overview|source-control/.test(key) || category === "Production") return "production";
  if (/terrain|foliage|scatter|weather|level|volume|prefab/.test(key) || category === "World Building") return "world";
  return "default";
}

function titleWord(word) {
  const upper = word.toUpperCase();
  if (["AI", "UI", "UX", "VFX", "HUD", "DCC"].includes(upper)) {
    return upper;
  }
  return word.charAt(0).toUpperCase() + word.slice(1);
}
