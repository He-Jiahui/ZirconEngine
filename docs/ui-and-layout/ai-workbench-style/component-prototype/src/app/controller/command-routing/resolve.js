import { explicitRouteForTarget } from "./explicit.js";
import { fallbackCommandRoute } from "./fallback.js";

export function commandRouteForTarget(target, activeModuleId) {
  return explicitRouteForTarget(target, activeModuleId)
    ?? fallbackCommandRoute(target, activeModuleId);
}
