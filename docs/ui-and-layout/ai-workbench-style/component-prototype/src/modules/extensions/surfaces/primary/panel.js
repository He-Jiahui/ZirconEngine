import { blueprintPrimaryPanel } from "./blueprint.js";
import { layoutKindPrimaryPanel } from "./layout-kind.js";

export function extensionPrimaryPanel(config) {
  if (config.primary) {
    return blueprintPrimaryPanel(config);
  }
  return layoutKindPrimaryPanel(config);
}
