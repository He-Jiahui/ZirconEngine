import { searchInput } from "./atoms.js";
import { alerts } from "./collections.js";
import { icon } from "./icons.js";
import {
  actionButton,
  assetStrip,
  bottomOutput,
  compactStats,
  listRows,
  moduleTable,
  moduleTree,
  panel,
  panelGroup,
  settingsRows,
  tag
} from "./module-components.js";

const esc = (value) => String(value ?? "").replace(/[&<>"']/g, (char) => ({
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#039;"
}[char]));

export function createEditorLibraryModule(extensionModuleConfigs, coreModules, defaultModuleId) {
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
      panel("Reference Groups", referenceGroupsList(extensionModuleConfigs)),
      panel("Implementation Rule", settingsRows([
        ["Shell", tag("Shared", "cyan")],
        ["Layout", tag("Left / Main / Right / Bottom", "green")],
        ["Style", tag("Workbench Dark", "blue")],
        ["Native Sync", tag("Core 11 only", "orange")]
      ])),
      panel("Core Modules", moduleTree(coreModules.map((module, index) => [module.label, module.icon, index === 1, 0])))
    ],
    center: () => editorLibraryCenter(extensionModuleConfigs, coreModules, defaultModuleId),
    right: () => editorLibraryDetails(extensionModuleConfigs),
    bottom: () => bottomOutput("editor-library", ["Coverage", "Reference Notes", "Routing Log"], editorLibraryBottom(extensionModuleConfigs))
  };
}

function editorLibraryCenter(extensionModuleConfigs, coreModules, defaultModuleId) {
  return `<div class="zr-module-editor-grid is-library">
    ${panel("Extended Editor Modules", `<div class="zr-extension-card-grid">${extensionModuleConfigs.map(extensionModuleCard).join("")}</div>`)}
    ${panel("Reference Blueprint Drilldown", referenceBlueprintDrilldown(extensionModuleConfigs))}
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

function referenceBlueprintDrilldown(extensionModuleConfigs) {
  const featured = representativeConfigs(extensionModuleConfigs);
  return `<div class="zr-library-drilldown" data-library-drilldown="reference-blueprints">
    ${panelGroup("library-drilldown", [
      {
        label: "Blueprints",
        content: moduleTable(["Reference", "Layout", "Primary Surface"], featured.map((config, index) => ({
          cells: [config.source, tag(config.layoutKind, "blue"), config.primary?.title ?? `${config.label} Workspace`],
          selected: index === 0
        })), "1.35fr 0.75fr 1fr"),
        active: true
      },
      {
        label: "Components",
        content: `${settingsRows([
          ["Atoms", tag("buttons / fields / toggles", "green")],
          ["Collections", tag("list / tree / table", "cyan")],
          ["Surfaces", tag("drawer / window / bottom panel", "blue")],
          ["Blueprints", tag(`${extensionModuleConfigs.length} reference-specific`, "green")]
        ])}${assetStrip(["toolbar actions", "left tools", "center editor", "right details", "bottom output", "handoff gate"])}`
      },
      {
        label: "Routes",
        content: `${moduleTable(["Route", "Response", "State"], [
          { cells: ["Extension card", "Switches module hash", tag("Module", "cyan")], selected: true },
          { cells: ["Toolbar action", "Output / Validation / References / Handoff", tag("Panel", "blue")] },
          { cells: ["Panel tab", "Shareable panel route", tag("Hash", "green")] },
          { cells: ["Rows and fields", "Command response state", tag("Command", "purple")] }
        ], "1fr 1.35fr 0.8fr")}${actionButton("Validate Coverage", "check")}${actionButton("Browse References", "folder")}`
      }
    ], { className: "is-library-drilldown" })}
  </div>`;
}

function editorLibraryDetails(extensionModuleConfigs) {
  return panelGroup("library-right", [
    { label: "Catalog", active: true, content: `${searchInput("Filter modules...")}${moduleTree(extensionModuleConfigs.map((config, index) => [config.label, config.icon, index === 0, 0]))}` },
    {
      label: "Coverage",
      content: `${compactStats([["Core", "11"], ["Extended", String(extensionModuleConfigs.length)], ["AI Refs", String(extensionModuleConfigs.length)], ["Shells", "1"]])}${settingsRows([
        ["Top Tabs", tag("Core + More", "cyan")],
        ["Rail", tag("Core + More", "cyan")],
        ["Extended", tag("Library Cards", "green")],
        ["Native", tag("Core 11", "orange")]
      ])}`
    },
    {
      label: "Routing",
      content: moduleTable(["Route", "Target", "Mode"], [
        { cells: ["More Editors", "editor-library", tag("Module", "cyan")], selected: true },
        { cells: ["Extension Card", "Selected editor", tag("Module", "green")] },
        { cells: ["Extension Toolbar", "Output / Validation / References", tag("Panel", "blue")] }
      ], "1fr 1fr 86px")
    }
  ]);
}

function editorLibraryBottom(extensionModuleConfigs) {
  return `<div class="zr-module-output-grid">
    ${moduleTable(["Sample", "Prototype Module", "Coverage"], extensionCoverageRows(extensionModuleConfigs), "1.4fr 1fr 98px")}
    ${alerts([["success", `${extensionModuleConfigs.length} extended editor cards use the same response path`], ["info", "Native handoff remains scoped to the core 11 modules"], ["warning", "Extended modules are prototype-only until native ZUI surfaces are added"]])}
  </div>`;
}

function referenceGroupsList(extensionModuleConfigs) {
  const counts = new Map();
  for (const config of extensionModuleConfigs) {
    counts.set(config.category, (counts.get(config.category) ?? 0) + 1);
  }
  const groups = [...counts.entries()].sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]));
  return listRows(groups.map(([label]) => label), 0, groups.map(([, count]) => String(count)));
}

function extensionCoverageRows(extensionModuleConfigs) {
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

function representativeConfigs(extensionModuleConfigs) {
  const wanted = new Set([
    "terrain-editor",
    "shader-editor",
    "sequencer",
    "source-control",
    "weather-editor",
    "world-state"
  ]);
  return extensionModuleConfigs.filter((config) => wanted.has(config.id)).slice(0, 6);
}
