import { alerts } from "../../../../components/data/collections.js";
import { assetStrip } from "../../../shared/module-components.js";
import { coreBottomRouteOptions } from "../routes.js";

export function materialBottom() {
  const routeOptions = coreBottomRouteOptions("material", "shader-output");
  return `<div class="zr-module-output-grid">
    <div class="zr-module-log"><p>[SM5] M_Rock_Cliff: Compiling...</p><p>[SM5] 5 instructions / 2 texture samplers</p><p class="is-success">[SM5] Compile successful</p></div>
    ${assetStrip(["Default", "Wet", "Snowy", "Mossy", "Night"], routeOptions)}
    ${alerts([["warning", "Texture sample uses default sampler"], ["warning", "Consider a packed texture"]])}
  </div>`;
}
