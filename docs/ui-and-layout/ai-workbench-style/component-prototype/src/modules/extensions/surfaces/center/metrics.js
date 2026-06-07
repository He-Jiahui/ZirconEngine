import { compactStats, panel, previewTile } from "../../../shared/module-components.js";

export function extensionMetricsPanel(config) {
  return panel("Controls & Metrics", `${previewTile(config.layoutKind)}${compactStats(config.metrics)}`);
}
