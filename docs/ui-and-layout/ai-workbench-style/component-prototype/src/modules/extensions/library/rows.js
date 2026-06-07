import { listRows, tag } from "../../shared/module-components.js";
import { libraryRouteOptions } from "./routes.js";

export function referenceGroupsList(extensionModuleConfigs) {
  const counts = new Map();
  for (const config of extensionModuleConfigs) {
    counts.set(config.category, (counts.get(config.category) ?? 0) + 1);
  }
  const groups = [...counts.entries()].sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]));
  return listRows(groups.map(([label]) => label), 0, groups.map(([, count]) => String(count)), libraryRouteOptions("module-bottom-editor-library:coverage", "workbench.library.group"));
}

export function extensionCoverageRows(extensionModuleConfigs) {
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
