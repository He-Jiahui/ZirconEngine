import { checkbox, input, select } from "./atoms.js";
import { extensionBlueprints } from "./extension-blueprints.js";
import { extensionReferenceSamples } from "./reference-samples.js";

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

function createReferenceExtensionConfig(source, category, glyph) {
  const id = source.replace(/^ai-|-layout\.png$/g, "");
  const label = id.split("-").map(titleWord).join(" ");
  const shortLabel = label.split(" ").slice(0, 2).join(" ");
  const subject = label.replace(/\s+(Editor|Audit|Layout|Dashboard|Manager)$/i, "");
  const recipe = recipeFor(source, category);
  const blueprint = extensionBlueprints[id] ?? {};
  const primary = blueprint.primary ?? null;
  const table = blueprint.table ?? primary?.rows ?? recipe.table(subject);
  return {
    id,
    label,
    shortLabel,
    icon: glyph,
    source,
    category,
    layoutKind: recipe.kind,
    blueprint: Boolean(primary),
    primary,
    status: blueprint.status ?? `${label} reference panel selected`,
    actions: blueprint.actions ?? recipe.actions(subject, shortLabel),
    tools: blueprint.tools ?? recipe.tools(subject),
    assets: blueprint.assets ?? assetsFor(subject, category, glyph),
    metrics: blueprint.metrics ?? recipe.metrics(subject),
    detailTabs: blueprint.detailTabs ?? recipe.detailTabs,
    settings: hydrateSettings(blueprint.settings ?? recipe.settings(subject)),
    table,
    tableHeaders: primary?.headers ?? ["Item", "State", "Value"],
    tableColumns: primary?.columns ?? "1.1fr 0.9fr 0.9fr"
  };
}

function hydrateSettings(rows) {
  return rows.map(([label, control]) => [label, controlMarkup(control)]);
}

function controlMarkup(control) {
  if (!control || typeof control !== "object") {
    return control;
  }
  if (control.kind === "select") {
    return select(control.value);
  }
  if (control.kind === "input") {
    return input("", { value: control.value });
  }
  if (control.kind === "checkbox") {
    return checkbox("", control.value);
  }
  return String(control.value ?? "");
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

export function titleWord(word) {
  const upper = word.toUpperCase();
  if (["AI", "UI", "UX", "VFX", "HUD", "DCC"].includes(upper)) {
    return upper;
  }
  return word.charAt(0).toUpperCase() + word.slice(1);
}
