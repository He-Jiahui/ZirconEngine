import { graphLink, node } from "../../../shared/module-components.js";
import { extensionRouteOptions } from "../routes.js";

export function graphNodes(config) {
  const positions = [[8, 18], [32, 14], [56, 22], [24, 56], [52, 58], [76, 38]];
  return config.tools.slice(0, 6).map((tool, index) => {
    const [x, y] = positions[index];
    return node(tool, index === 0 ? "Selected" : config.category, x, y, ["cyan", "blue", "green", "purple", "orange", "neutral"][index % 6], extensionRouteOptions(config, "output", "workbench.extension.graph"));
  });
}

export function extensionLinks() {
  return `${graphLink(20, 28, 16)}${graphLink(44, 26, 16, 12)}${graphLink(36, 58, 13, -20)}${graphLink(64, 62, 10, -18)}`;
}
