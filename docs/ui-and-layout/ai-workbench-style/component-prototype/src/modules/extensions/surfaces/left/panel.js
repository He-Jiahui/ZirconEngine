import { extensionAssetsPanel } from "./assets.js";
import { extensionReferencePanel } from "./reference.js";
import { extensionToolsPanel } from "./tools.js";

export function extensionLeft(config) {
  return [
    extensionReferencePanel(config),
    extensionToolsPanel(config),
    extensionAssetsPanel(config)
  ];
}
