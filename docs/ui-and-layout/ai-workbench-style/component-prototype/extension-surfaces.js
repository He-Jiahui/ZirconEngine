import { searchInput } from "./atoms.js";
import { alerts } from "./collections.js";
import { titleWord } from "./extension-configs.js";
import { extensionHandoffPanel } from "./extension-handoff.js";
import {
  actionButton,
  assetStrip,
  compactStats,
  curvePanel,
  graphBoard,
  graphLink,
  listRows,
  moduleTable,
  moduleTree,
  node,
  panel,
  panelGroup,
  previewTile,
  progress,
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

export function extensionLeft(config) {
  return [
    panel("Reference", settingsRows([
      ["AI Sample", tag(config.source.replace(/^ai-|-layout\.png$/g, "").replace(/-/g, " "), "cyan")],
      ["Category", tag(config.category, "blue")],
      ["Layout", tag(`${titleWord(config.layoutKind)} Workbench`, "green")],
      ["Native Contract", tag("Prototype Only", "orange")]
    ])),
    panel("Tools", `${searchInput("Search tools...")}${listRows(config.tools, 0)}`),
    panel("Assets", `${searchInput("Search assets...")}${moduleTree(config.assets)}`)
  ];
}

export function extensionCenter(config) {
  return `<div class="zr-module-editor-grid is-extension is-extension-${esc(config.layoutKind)}" data-extension-blueprint="${config.blueprint ? "reference" : "recipe"}">
    ${extensionPrimaryPanel(config)}
    ${panel("Controls & Metrics", `${previewTile(config.layoutKind)}${compactStats(config.metrics)}`)}
    ${panel("Reference Rhythm", `${assetStrip(config.tools.slice(0, 6))}${settingsRows([["Source", config.source], ["Category", config.category], ["Blueprint", tag(config.blueprint ? "Reference Specific" : "Category Recipe", config.blueprint ? "green" : "blue")], ["Response", tag("Click any control", "green")]])}`)}
  </div>`;
}

export function extensionDetails(config) {
  const [firstTab, secondTab, thirdTab] = config.detailTabs;
  return panelGroup(`${config.id}-right`, [
    { label: firstTab, active: true, content: `${settingsRows(config.settings)}${actionButton(config.actions[0][1], config.actions[0][0])}` },
    { label: secondTab, content: moduleTable(config.tableHeaders, toRows(config.table, 1), config.tableColumns) },
    { label: thirdTab, content: `${alerts([["info", `${config.label} follows the shared module panel contract`], ["success", "Visible controls route through prototype feedback"], ["warning", "Native implementation pending"]])}${actionButton("More Editors", "grid")}` }
  ], { className: "is-extension-right" });
}

export function extensionBottomOutput(config) {
  return `<section class="zr-panel zr-module-bottom" data-surface="drawer" data-module-panel="bottom" data-panel-host="module-bottom-${esc(config.id)}">
    ${panelGroup(`module-bottom-${config.id}`, [
      { label: "Output", content: extensionBottom(config), active: true },
      { label: "Validation", content: extensionValidationPanel(config) },
      { label: "References", content: extensionReferencesPanel(config) },
      { label: "Handoff", content: extensionHandoffPanel(config) }
    ], { className: "is-module-bottom" })}
  </section>`;
}

function extensionPrimaryPanel(config) {
  if (config.primary) {
    return blueprintPrimaryPanel(config);
  }
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

function extensionBottom(config) {
  return `<div class="zr-module-output-grid">
    ${moduleTable(config.tableHeaders, toRows(config.table), config.tableColumns)}
    <div class="zr-module-log"><p>${esc(config.label)}: opened from ${esc(config.source)}</p><p class="is-success">Prototype route active and response feedback enabled.</p><p class="is-warning">Native ZUI surface not generated for this extended editor.</p></div>
  </div>`;
}

function extensionValidationPanel(config) {
  return `<div class="zr-module-output-grid">
    ${moduleTable(["Check", "Scope", "State"], validationRows(config), "1.2fr 1fr 0.8fr")}
    ${settingsRows([
      ["Blueprint", tag(config.blueprint ? "Reference Specific" : "Category Recipe", config.blueprint ? "green" : "blue")],
      ["Category", tag(config.category, "cyan")],
      ["Native Surface", tag("Pending", "orange")],
      ["Route State", tag("Panel Hash Active", "green")]
    ])}
    ${alerts([["success", `${config.label} uses the shared module bottom drawer`], ["warning", "Native ZUI surface is not generated for this extension"], ["info", "Validation rows are prototype route targets"]])}
  </div>`;
}

function extensionReferencesPanel(config) {
  return `<div class="zr-module-output-grid">
    ${moduleTable(["Reference", "Kind", "Route"], referenceRows(config), "1.4fr 0.8fr 1fr")}
    ${panel("Reference assets", `${moduleTree(config.assets)}${assetStrip(config.tools.slice(0, 4))}`)}
    <div class="zr-module-log"><p>${esc(config.label)} reference source: ${esc(config.source)}</p><p class="is-success">Cards, rows, and toolbar actions keep the user inside this module route.</p><p>${esc(config.category)} panels reuse the same bottom output, validation, and reference component grammar.</p>${actionButton("More Editors", "grid")}</div>
  </div>`;
}

function validationRows(config) {
  return [
    { cells: ["Component Stack", config.layoutKind, tag("Ready", "green")], selected: true },
    { cells: ["Toolbar Routes", `${config.actions.length + 2} actions`, tag("Ready", "green")] },
    { cells: ["Reference Blueprint", config.source, tag(config.blueprint ? "Specific" : "Recipe", config.blueprint ? "cyan" : "blue")] },
    { cells: ["Native Handoff", "ZUI retained host", tag("Pending", "orange")] }
  ];
}

function referenceRows(config) {
  return [
    { cells: [config.source, "AI sample", tag("Current", "cyan")], selected: true },
    { cells: [config.category, "Category", tag(config.layoutKind, "blue")] },
    { cells: [config.tools[0] ?? config.label, "Primary tool", tag("Output", "green")] },
    { cells: [config.detailTabs[0] ?? "Details", "Right panel", tag("Details", "purple")] }
  ];
}

function blueprintPrimaryPanel(config) {
  const primary = config.primary;
  if (primary.kind === "graph") {
    return panel(primary.title, graphBoard(`${config.layoutKind}-blueprint`, primary.nodes.map(([label, type, x, y, tone]) =>
      node(label, type, x, y, tone)
    ), extensionLinks()));
  }
  if (primary.kind === "queue") {
    return panel(primary.title, moduleTable(primary.headers, primary.rows.map((row, index) => ({
      cells: [row[0], row[1], progress(progressValue(row, index))],
      selected: index === 0
    })), primary.columns));
  }
  if (primary.kind === "timeline") {
    return panel(primary.title, `${timeline(config.id)}${curvePanel()}${moduleTable(primary.headers, toRows(primary.rows), primary.columns)}`);
  }
  return panel(primary.title, moduleTable(primary.headers, toRows(primary.rows), primary.columns));
}

function toRows(table, selectedIndex = 0) {
  return table.map((row, index) => ({
    cells: row,
    selected: index === selectedIndex
  }));
}

function progressValue(row, index) {
  const raw = Number.parseFloat(row[2]);
  if (Number.isFinite(raw)) return raw;
  if (/done|passed|ready/i.test(String(row[1]))) return 100;
  if (/running|warning/i.test(String(row[1]))) return index === 0 ? 62 : 34;
  return 0;
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
