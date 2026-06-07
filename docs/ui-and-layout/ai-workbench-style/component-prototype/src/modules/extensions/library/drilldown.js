import {
  actionButton,
  assetStrip,
  moduleTable,
  panelGroup,
  settingsRows,
  tag
} from "../../shared/module-components.js";
import { libraryRouteOptions } from "./routes.js";

export function referenceBlueprintDrilldown(extensionModuleConfigs) {
  const featured = representativeConfigs(extensionModuleConfigs);
  return `<div class="zr-library-drilldown" data-library-drilldown="reference-blueprints">
    ${panelGroup("library-drilldown", [
      {
        label: "Blueprints",
        content: moduleTable(["Reference", "Layout", "Primary Surface"], featured.map((config, index) => ({
          cells: [config.source, tag(config.layoutKind, "blue"), config.primary?.title ?? `${config.label} Workspace`],
          selected: index === 0
        })), "1.35fr 0.75fr 1fr", libraryRouteOptions("module-bottom-editor-library:reference-notes", "workbench.library.blueprint")),
        active: true
      },
      {
        label: "Components",
        content: `${settingsRows([
          ["Atoms", tag("buttons / fields / toggles", "green")],
          ["Collections", tag("list / tree / table", "cyan")],
          ["Surfaces", tag("drawer / window / bottom panel", "blue")],
          ["Blueprints", tag(`${extensionModuleConfigs.length} reference-specific`, "green")]
        ])}${assetStrip(["toolbar actions", "left tools", "center editor", "right details", "bottom output", "handoff gate"], libraryRouteOptions("module-bottom-editor-library:reference-notes", "workbench.library.component"))}`
      },
      {
        label: "Routes",
        content: `${moduleTable(["Route", "Response", "State"], [
          { cells: ["Extension card", "Switches module hash", tag("Module", "cyan")], selected: true },
          { cells: ["Toolbar action", "Output / Validation / References / Handoff", tag("Panel", "blue")] },
          { cells: ["Panel tab", "Shareable panel route", tag("Hash", "green")] },
          { cells: ["Rows and fields", "Command response state", tag("Command", "purple")] }
        ], "1fr 1.35fr 0.8fr", libraryRouteOptions("module-bottom-editor-library:routing-log", "workbench.library.route"))}${actionButton("Validate Coverage", "check", libraryRouteOptions("module-bottom-editor-library:routing-log", "workbench.library.route"))}${actionButton("Browse References", "folder", libraryRouteOptions("module-bottom-editor-library:reference-notes", "workbench.library.reference"))}`
      }
    ], { className: "is-library-drilldown" })}
  </div>`;
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
