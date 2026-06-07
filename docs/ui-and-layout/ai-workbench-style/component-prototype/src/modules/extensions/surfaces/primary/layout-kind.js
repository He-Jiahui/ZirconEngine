import { alerts } from "../../../../components/data/collections.js";
import { curvePanel, graphBoard, moduleTable, panel, previewTile, progress, timeline } from "../../../shared/module-components.js";
import { extensionRouteOptions } from "../routes.js";
import { toRows } from "../utils.js";
import { extensionLinks, graphNodes } from "./graph.js";

export function layoutKindPrimaryPanel(config) {
  switch (config.layoutKind) {
    case "animation":
      return panel(`${config.label} Timeline`, `${timeline(config.id)}${curvePanel()}${moduleTable(["Track", "Range", "State"], toRows(config.table), "1fr 0.8fr 0.8fr", extensionRouteOptions(config, "output", "workbench.extension.output"))}`);
    case "rendering":
      return panel(`${config.label} Render Stack`, `${previewTile("render")}${moduleTable(["Pass", "State", "GPU"], toRows(config.table), "1fr 0.8fr 0.8fr", extensionRouteOptions(config, "output", "workbench.extension.output"))}`);
    case "ui":
      return panel(`${config.label} Layout Map`, graphBoard("ui-extension", graphNodes(config), extensionLinks()));
    case "production":
      return panel(`${config.label} Queue`, moduleTable(["Job", "State", "Progress"], config.table.map((row, index) => ({
        cells: [row[0], row[1], progress(index === 0 ? 62 : Number(row[2]) || 0)],
        selected: index === 0
      })), "1.2fr 0.8fr 1fr", extensionRouteOptions(config, "output", "workbench.extension.output")));
    case "diagnostics":
      return panel(`${config.label} Live Feed`, `${moduleTable(["Time", "Subsystem", "Level"], toRows(config.table), "1fr 1fr 0.8fr", extensionRouteOptions(config, "output", "workbench.extension.output"))}${alerts([["warning", "Warnings are grouped by subsystem"], ["info", "Click log rows to pin a diagnostic command"]])}`);
    case "online":
    case "data":
    case "runtime":
      return panel(`${config.label} Grid`, moduleTable(["Item", "State", "Value"], toRows(config.table), "1.2fr 0.8fr 0.8fr", extensionRouteOptions(config, "output", "workbench.extension.output")));
    default:
      return panel(`${config.label} Workspace`, graphBoard(config.layoutKind, graphNodes(config), extensionLinks()));
  }
}
