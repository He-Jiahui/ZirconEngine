import { moduleById } from "./modules.js";

const moduleRouteMap = new Map([
  ["browse", "asset-browser"],
  ["import", "asset-browser"],
  ["import-assets", "asset-browser"],
  ["import-from-path", "asset-browser"],
  ["reimport", "asset-browser"],
  ["reimport-assets", "asset-browser"],
  ["build", "asset-browser"],
  ["cook", "asset-browser"],
  ["package", "asset-browser"],
  ["simulation", "gameplay-effect"],
  ["preview", "scene"],
  ["preview-level", "scene"],
  ["play", "scene"],
  ["debug", "behavior-tree"],
  ["validate", "asset-browser"],
  ["add-tag", "gameplay-tags"],
  ["rename", "gameplay-tags"],
  ["validate-tags", "gameplay-tags"],
  ["compile", "gameplay-effect"],
  ["compile-ability", "gameplay-ability"],
  ["playtest", "gameplay-ability"],
  ["add-modifier", "gameplay-effect"],
  ["activate-ability", "gameplay-ability"],
  ["play-montage", "gameplay-ability"],
  ["texture-sample", "material"],
  ["multiply", "material"],
  ["lerp", "material"],
  ["roughness", "material"],
  ["selector", "behavior-tree"],
  ["sequence", "behavior-tree"],
  ["attack", "behavior-tree"],
  ["simulate-perception", "ai-perception"],
  ["validate-query", "ai-perception"],
  ["sight", "ai-perception"],
  ["hearing", "ai-perception"],
  ["compile-pipeline", "render-pipeline"],
  ["preview-frame", "render-pipeline"],
  ["build-frame", "render-pipeline"],
  ["post-process-pass", "render-pipeline"],
  ["preview-hud", "hud-editor"],
  ["validate-ui", "hud-editor"],
  ["build-ui", "hud-editor"],
  ["weapon-panel", "hud-editor"],
  ["p-bolt-01", "vfx"],
  ["simulate", "vfx"],
  ["more-editors", "editor-library"],
  ["find-editor", "editor-library"],
  ["browse-references", "editor-library"],
  ["validate-coverage", "editor-library"],
  ["core-modules", "gameplay-effect"]
]);

const moduleScopedRouteMap = new Map([
  ["scene:preview", { moduleId: "scene", panelTarget: "module-bottom-scene:selection" }],
  ["gameplay-effect:compile", { moduleId: "gameplay-effect", panelTarget: "module-bottom-gameplay-effect:compile-log" }],
  ["gameplay-effect:diff", { moduleId: "gameplay-effect", panelTarget: "module-bottom-gameplay-effect:compile-log" }],
  ["gameplay-effect:simulation", { moduleId: "gameplay-effect", panelTarget: "module-bottom-gameplay-effect:simulation-output" }],
  ["material:compile", { moduleId: "material", panelTarget: "module-bottom-material:shader-output" }],
  ["material:preview", { moduleId: "material", panelTarget: "module-bottom-material:preview-variants" }],
  ["material:build", { moduleId: "material", panelTarget: "module-bottom-material:warnings" }],
  ["behavior-tree:play", { moduleId: "behavior-tree", panelTarget: "behavior-right:execution" }],
  ["behavior-tree:debug", { moduleId: "behavior-tree", panelTarget: "module-bottom-behavior-tree:runtime-trace" }],
  ["behavior-tree:validate", { moduleId: "behavior-tree", panelTarget: "module-bottom-behavior-tree:validation-issues" }],
  ["asset-browser:validate", { moduleId: "asset-browser", panelTarget: "module-bottom-asset-browser:validation" }],
  ["asset-browser:build", { moduleId: "asset-browser", panelTarget: "module-bottom-asset-browser:cook" }],
  ["vfx:simulate", { moduleId: "vfx", panelTarget: "module-bottom-vfx:timeline" }],
  ["vfx:compile", { moduleId: "vfx", panelTarget: "module-bottom-vfx:compile-output" }]
]);

const panelRouteMap = new Map([
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

export function routeForCommand(command, activeModuleId) {
  const normalized = normalizeCommand(command);
  const scopedRoute = moduleScopedRouteMap.get(`${activeModuleId}:${normalized}`)
    ?? extensionRouteForCommand(normalized, activeModuleId);
  const nextModuleId = scopedRoute?.moduleId ?? moduleRouteMap.get(normalized);
  const panelTarget = resolvePanelTarget(
    scopedRoute?.panelTarget ?? panelRouteMap.get(normalized),
    nextModuleId ?? activeModuleId
  );
  if (!nextModuleId && !panelTarget) return null;
  return {
    command: normalized,
    moduleId: nextModuleId ?? activeModuleId,
    panelTarget,
    label: routeLabel(normalized, nextModuleId ?? activeModuleId, panelTarget)
  };
}

function extensionRouteForCommand(command, activeModuleId) {
  const module = moduleById(activeModuleId);
  if (!module.extension) return null;
  if (command === "more-editors") {
    return { moduleId: "editor-library", panelTarget: "module-bottom-editor-library:routing-log" };
  }
  return {
    moduleId: activeModuleId,
    panelTarget: `module-bottom-${activeModuleId}:${extensionPanelKeyForCommand(command)}`
  };
}

function extensionPanelKeyForCommand(command) {
  const tokens = command.split("-").filter(Boolean);
  const verb = tokens[0] ?? "";
  if (["validate", "compile", "build", "check", "audit", "open"].includes(verb)
    || tokens.some((token) => ["issue", "issues", "warning", "warnings", "error", "errors"].includes(token))) {
    return "validation";
  }
  if (["reference", "references", "browse", "history", "review", "save", "export", "publish", "migrate", "load"].includes(verb)) {
    return "references";
  }
  return "output";
}

export function applyPanelRoute(panelTarget, root = document) {
  if (!panelTarget) return false;
  const tab = root.querySelector(`[data-panel-tab="${cssEscape(panelTarget)}"]`);
  if (!tab) return false;
  activateTab(tab);
  return true;
}

export function normalizeCommand(value) {
  return String(value ?? "")
    .trim()
    .toLowerCase()
    .replace(/[_\s/]+/g, "-")
    .replace(/[^a-z0-9-]+/g, "")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");
}

function resolvePanelTarget(pattern, moduleId) {
  if (!pattern) return "";
  return pattern.replaceAll("{module}", moduleId);
}

function routeLabel(command, moduleId, panelTarget) {
  const module = moduleById(moduleId);
  const panel = panelTarget ? ` / ${panelTarget.split(":").at(-1).replace(/-/g, " ")}` : "";
  return `${command.replace(/-/g, " ")} -> ${module.label}${panel}`;
}

function activateTab(tab) {
  [...tab.parentElement.children].forEach((item) => {
    item.classList.remove("is-active");
    item.setAttribute("aria-selected", "false");
  });
  tab.classList.add("is-active");
  tab.setAttribute("aria-selected", "true");

  const panelTarget = tab.dataset.panelTab;
  const host = tab.closest("[data-panel-host]");
  host?.querySelectorAll(".zr-panel-view").forEach((view) => {
    view.classList.toggle("is-active", view.dataset.panelView === panelTarget);
  });
}

function cssEscape(value) {
  if (globalThis.CSS?.escape) return globalThis.CSS.escape(value);
  return String(value).replace(/["\\]/g, "\\$&");
}
