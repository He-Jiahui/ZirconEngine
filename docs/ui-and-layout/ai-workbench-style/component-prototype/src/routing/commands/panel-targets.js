export const panelRouteMap = new Map([
  ["diff", "module-bottom-{module}:compile-log"],
  ["compile", "module-bottom-{module}:compile-log"],
  ["validation", "module-bottom-{module}:validation"],
  ["warnings", "module-bottom-{module}:warnings"],
  ["details", "{module}-right:details"],
  ["compile-ability", "module-bottom-gameplay-ability:compile-log"],
  ["playtest", "module-bottom-gameplay-ability:timeline"],
  ["validate-tags", "module-bottom-gameplay-tags:validation-log"],
  ["validate-query", "module-bottom-ai-perception:validation"],
  ["simulate-perception", "module-bottom-ai-perception:perception-timeline"],
  ["compile-pipeline", "module-bottom-render-pipeline:compile-output"],
  ["preview-frame", "render-right:frame-stages"],
  ["build-frame", "module-bottom-render-pipeline:frame-capture-log"],
  ["preview-hud", "module-bottom-hud-editor:preview-log"],
  ["validate-ui", "module-bottom-hud-editor:validation"],
  ["build-ui", "module-bottom-hud-editor:performance"],
  ["metadata", "asset-right:metadata"],
  ["preview", "asset-right:preview"],
  ["issues", "asset-right:issues"],
  ["parameters", "material-right:parameters"],
  ["node-details", "material-right:node-details"],
  ["execution", "behavior-right:execution"],
  ["blackboard", "behavior-right:blackboard"],
  ["stages", "vfx-right:stages"],
  ["curves", "module-bottom-vfx:curves"],
  ["timeline", "module-bottom-vfx:timeline"],
  ["shader-output", "module-bottom-material:shader-output"],
  ["queue", "module-bottom-asset-browser:queue"],
  ["output", "module-bottom-asset-browser:output"]
]);

export function resolvePanelTarget(pattern, moduleId) {
  if (!pattern) return "";
  return pattern.replaceAll("{module}", moduleId);
}
