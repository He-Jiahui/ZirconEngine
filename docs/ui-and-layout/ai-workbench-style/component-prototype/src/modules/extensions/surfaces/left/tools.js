import { searchInput } from "../../../../components/inputs/atoms.js";
import { listRows, panel } from "../../../shared/module-components.js";
import { extensionRouteOptions } from "../routes.js";

export function extensionToolsPanel(config) {
  return panel("Tools", `${searchInput("Search tools...")}${listRows(config.tools, 0, [], extensionRouteOptions(config, "output", "workbench.extension.tool"))}`);
}
