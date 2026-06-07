import { curvePanel, graphBoard, moduleTable, node, panel, progress, timeline } from "../../../shared/module-components.js";
import { extensionRouteOptions } from "../routes.js";
import { progressValue, toRows } from "../utils.js";
import { extensionLinks } from "./graph.js";

export function blueprintPrimaryPanel(config) {
  const primary = config.primary;
  if (primary.kind === "graph") {
    return panel(primary.title, graphBoard(`${config.layoutKind}-blueprint`, primary.nodes.map(([label, type, x, y, tone]) =>
      node(label, type, x, y, tone, extensionRouteOptions(config, "output", "workbench.extension.graph"))
    ), extensionLinks()));
  }
  if (primary.kind === "queue") {
    return panel(primary.title, moduleTable(primary.headers, primary.rows.map((row, index) => ({
      cells: [row[0], row[1], progress(progressValue(row, index))],
      selected: index === 0
    })), primary.columns, extensionRouteOptions(config, "output", "workbench.extension.output")));
  }
  if (primary.kind === "timeline") {
    return panel(primary.title, `${timeline(config.id)}${curvePanel()}${moduleTable(primary.headers, toRows(primary.rows), primary.columns, extensionRouteOptions(config, "output", "workbench.extension.output"))}`);
  }
  return panel(primary.title, moduleTable(primary.headers, toRows(primary.rows), primary.columns, extensionRouteOptions(config, "output", "workbench.extension.output")));
}
