import { searchInput } from "../../../../components/inputs/atoms.js";
import { moduleTree, panel } from "../../../shared/module-components.js";
import { extensionRouteOptions } from "../routes.js";

export function extensionAssetsPanel(config) {
  return panel("Assets", `${searchInput("Search assets...")}${moduleTree(config.assets, extensionRouteOptions(config, "references", "workbench.extension.asset"))}`);
}
