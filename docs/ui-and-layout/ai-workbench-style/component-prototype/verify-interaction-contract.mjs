import { readFileSync, readdirSync } from "node:fs";
import { popups, rail, statusbar, topbar, workbenchWindow } from "./surfaces.js";
import { defaultModuleId, extensionModules, modules, nativeModules, webModuleTabs, moduleWorkspace } from "./modules.js";
import {
  allReferenceSampleSources,
  coreReferenceSamples,
  extensionReferenceSamples,
  supplementalReferenceSamples
} from "./reference-samples.js";

const appHtml = workbenchWindow([topbar(defaultModuleId), rail(defaultModuleId), moduleWorkspace(defaultModuleId), statusbar("Ready"), popups()]);
const libraryHtml = moduleWorkspace("editor-library");
const extensionHtml = moduleWorkspace(extensionModules[0]?.id);
const appSource = readFileSync(new URL("./app.js", import.meta.url), "utf8");
const indexSource = readFileSync(new URL("./index.html", import.meta.url), "utf8");
const layoutSource = readFileSync(new URL("./layout.js", import.meta.url), "utf8");
const layoutCssSource = readFileSync(new URL("./layout.css", import.meta.url), "utf8");
const modulesSource = readFileSync(new URL("./modules.js", import.meta.url), "utf8");
const extensionModulesSource = readFileSync(new URL("./extension-modules.js", import.meta.url), "utf8");
const moduleComponentsSource = readFileSync(new URL("./module-components.js", import.meta.url), "utf8");
const collectionsSource = readFileSync(new URL("./collections.js", import.meta.url), "utf8");
const modulesCssSource = readFileSync(new URL("./modules.css", import.meta.url), "utf8");
const routesSource = readFileSync(new URL("./routes.js", import.meta.url), "utf8");
const responsiveSource = readFileSync(new URL("./validate-responsive.mjs", import.meta.url), "utf8");
const controlRouteSource = readFileSync(new URL("./validate-control-routes.mjs", import.meta.url), "utf8");
const referencePngSources = readdirSync(new URL("../", import.meta.url))
  .filter((name) => name.toLowerCase().endsWith(".png"))
  .sort();
const manifestSources = allReferenceSampleSources();
const manifestSourceSet = new Set(manifestSources);
const referencePngSourceSet = new Set(referencePngSources);
const duplicateManifestSources = manifestSources.filter((source, index) => manifestSources.indexOf(source) !== index);
const missingManifestSources = referencePngSources.filter((source) => !manifestSourceSet.has(source));
const staleManifestSources = manifestSources.filter((source) => !referencePngSourceSet.has(source));
const expectedExtensionCount = extensionReferenceSamples.length;
const representativeExtensionIds = [
  "terrain-editor",
  "lighting-bake",
  "sequencer",
  "montage-editor",
  "physics-collision",
  "navmesh-ai",
  "data-table",
  "console-diagnostics",
  "shader-editor",
  "source-control",
  "ui-binding",
  "weather-editor",
  "world-state"
];

const checks = [
  ["tokens layer loaded first", indexSource.indexOf("tokens.css") < indexSource.indexOf("layout.css")],
  ["layout layer before atoms", indexSource.indexOf("layout.css") < indexSource.indexOf("atoms.css")],
  ["atoms layer before collections", indexSource.indexOf("atoms.css") < indexSource.indexOf("collections.css")],
  ["collections layer before surfaces", indexSource.indexOf("collections.css") < indexSource.indexOf("surfaces.css")],
  ["surfaces layer before modules", indexSource.indexOf("surfaces.css") < indexSource.indexOf("modules.css")],
  ["modules layer before workbench", indexSource.indexOf("modules.css") < indexSource.indexOf("workbench.css")],
  ["layout stack factory", layoutSource.includes("export function stack") && layoutCssSource.includes(".zr-layout")],
  ["layout cluster factory", layoutSource.includes("export function cluster") && layoutCssSource.includes('[data-zr-align="center"]')],
  ["layout grid factory", layoutSource.includes("export function grid") && layoutCssSource.includes(".zr-grid")],
  ["reference sample manifest covers png directory", referencePngSources.length >= 50 && missingManifestSources.length === 0 && staleManifestSources.length === 0 && duplicateManifestSources.length === 0],
  ["approved shell sample mapped to gameplay effect", coreReferenceSamples.some((sample) => sample.source === "ai-gameplay-effect-layout.png" && sample.moduleId === defaultModuleId && sample.role === "approved-shell-core")],
  ["core reference samples mapped to native modules", coreReferenceSamples.length === nativeModules.length && coreReferenceSamples.every((sample) => nativeModules.some((module) => module.id === sample.moduleId))],
  ["extension reference samples mapped to extension modules", extensionReferenceSamples.length === extensionModules.length && extensionReferenceSamples.every((sample) => extensionModules.some((module) => module.source === sample.source))],
  ["supplemental reference samples classified", supplementalReferenceSamples.length === 4 && supplementalReferenceSamples.every((sample) => modules.some((module) => module.id === sample.moduleId) && ["variant", "shell-style"].includes(sample.role))],
  ["native module registry has eleven editor modules", nativeModules.length === 11 && ["scene", "gameplay-effect", "gameplay-ability", "gameplay-tags", "ai-perception", "material", "behavior-tree", "render-pipeline", "asset-browser", "vfx", "hud-editor"].every((id) => nativeModules.some((module) => module.id === id))],
  ["extension module registry", extensionModules.length === expectedExtensionCount && modules.length === nativeModules.length + extensionModules.length + 1 && webModuleTabs.length === nativeModules.length + 1 && representativeExtensionIds.every((id) => extensionModules.some((module) => module.id === id))],
  ["extension module generator boundary", modulesSource.includes("./extension-modules.js") && modulesSource.includes("buildExtensionModules(coreModules, defaultModuleId)") && extensionModulesSource.includes("export function buildExtensionModules") && extensionModulesSource.includes("./reference-samples.js")],
  ["extension module category recipes", extensionModulesSource.includes("recipeByKind") && ["world", "rendering", "animation", "ui", "production", "diagnostics", "online", "simulation", "data", "gameplay", "runtime", "vfx"].every((kind) => extensionModulesSource.includes(`${kind}:`))],
  ["extension module varied layouts", extensionHtml.includes("is-extension is-extension-world") && moduleWorkspace("shader-editor").includes("is-extension-rendering") && moduleWorkspace("sequencer").includes("is-extension-animation") && moduleWorkspace("source-control").includes("is-extension-production") && moduleWorkspace("weather-editor").includes("is-extension-world")],
  ["module shared component layer", modulesSource.includes("./module-components.js") && moduleComponentsSource.includes("export function moduleMain") && moduleComponentsSource.includes("export function moduleTable")],
  ["semantic module table action ids", moduleComponentsSource.includes("function tableRowActionId") && moduleComponentsSource.includes("function tableRowLabel") && moduleComponentsSource.includes("readableCell") && moduleComponentsSource.includes("aria-label") && moduleComponentsSource.includes("!/^\\d+(?:\\.\\d+)?$/")],
  ["module workspace uses layout primitives", appHtml.includes("zr-module-editor-grid") && appHtml.includes("zr-layout") && appHtml.includes("zr-grid")],
  ["toolbar uses cluster primitive", appHtml.includes("zr-topbar-tools") && appHtml.includes("zr-cluster")],
  ["module tabs in topbar", appHtml.includes("zr-module-tabs") && (appHtml.match(/class="zr-module-tab(?:\s|")/g) ?? []).length === webModuleTabs.length],
  ["extension module library cards", libraryHtml.includes("zr-extension-card-grid") && (libraryHtml.match(/data-module-source="extension-library"/g) ?? []).length === extensionModules.length],
  ["extension module surfaces", extensionHtml.includes("zr-module-editor-grid is-extension") && extensionHtml.includes("Prototype Only") && extensionHtml.includes("More Editors")],
  ["module toolbar actions", appHtml.includes("zr-module-toolbar") && (appHtml.match(/data-action=/g) ?? []).length >= 12],
  ["module left drawer", appHtml.includes("zr-module-left") && appHtml.includes('data-module-panel="left"')],
  ["module main surface", appHtml.includes("zr-module-main") && appHtml.includes('data-module-panel="main"')],
  ["module right window", appHtml.includes("zr-module-right") && appHtml.includes('data-module-panel="right"')],
  ["module bottom drawer", appHtml.includes("zr-module-bottom") && appHtml.includes('data-module-panel="bottom"')],
  ["module graph surface", appHtml.includes("zr-module-graph") && appHtml.includes("zr-module-node")],
  ["module table/list/tree surfaces", appHtml.includes("zr-module-table") && appHtml.includes("zr-module-list") && appHtml.includes("zr-module-tree")],
  ["module list action row labels", moduleComponentsSource.includes("export function listRows") && moduleComponentsSource.includes('aria-label="${esc(item)}"') && responsiveSource.includes("row-target-tags")],
  ["module status message", appHtml.includes("data-status-message") && appHtml.includes("zr-module-status-message")],
  ["button atom", appHtml.includes("zr-button")],
  ["input atom", appHtml.includes("zr-input")],
  ["checkbox atom", appHtml.includes("zr-checkbox")],
  ["toggle atom", appHtml.includes("zr-switch")],
  ["icon button atom", appHtml.includes("zr-icon-button")],
  ["tabs atom", appHtml.includes("zr-tabs")],
  ["dropdown atom", appHtml.includes("zr-select") && appHtml.includes("data-dropdown")],
  ["list collection", appHtml.includes("zr-list")],
  ["list collection action rows", collectionsSource.includes('class="zr-list-item') && collectionsSource.includes('type="button" data-action="${actionKey(item.label)}"') && collectionsSource.includes('aria-label="${esc(item.label)}"') && responsiveSource.includes("assertCollectionRowButtons")],
  ["tree view collection", appHtml.includes("zr-module-tree")],
  ["tree collection action rows", collectionsSource.includes('<button class="zr-tree-row') && collectionsSource.includes('type="button" data-action="${actionKey(node.label)}"') && collectionsSource.includes('aria-label="${esc(node.label)}"') && responsiveSource.includes("assertCollectionTreeRows")],
  ["table view collection", appHtml.includes("zr-module-table")],
  ["table collection action rows", collectionsSource.includes('class="zr-table-row') && collectionsSource.includes('type="button" data-action="${actionKey(row[0])}"') && collectionsSource.includes('aria-label="${esc(row[0])}"') && responsiveSource.includes("row-healthregen")],
  ["popup collection", appHtml.includes("zr-popup-layer")],
  ["module panel tab target", appHtml.includes("data-panel-tab") && appHtml.includes("module-bottom-gameplay-effect")],
  ["window surface", appHtml.includes('data-surface="window"')],
  ["drawer surfaces", (appHtml.match(/data-surface="drawer"/g) ?? []).length >= 2],
  ["module window surface", appHtml.includes('class="zr-panel zr-module-right" data-surface="window"')],
  ["panel view surfaces", (appHtml.match(/data-surface="panel-view"/g) ?? []).length >= 4],
  ["tab aria selected", appHtml.includes('role="tab" aria-selected="true"')],
  ["dropdown popup layer", appHtml.includes('id="popup-layer"')],
  ["menu rows are action buttons", appHtml.includes('class="zr-menu-row') && appHtml.includes('data-menu-item="new"') && appHtml.includes('data-action="menu-new"') && appSource.includes('action.closest(".zr-popup-layer")')],
  ["module navigation handler", appSource.includes("[data-module]") && appSource.includes("moduleWorkspace(activeModuleId)")],
  ["module deep link handler", appSource.includes("moduleIdFromLocation") && appSource.includes("syncModuleHash") && appSource.includes('addEventListener("popstate"') && responsiveSource.includes("assertDeepLink")],
  ["panel deep link handler", appSource.includes("requestedPanelTargetFromLocation") && appSource.includes("activatePanelTarget") && appSource.includes("routeHash") && responsiveSource.includes("assertPanelDeepLinks") && responsiveSource.includes("assertCommandPanelHistory")],
  ["command hash state handler", appSource.includes("commandIdFromLocation") && appSource.includes("recordCommand") && appSource.includes("latestCommandId") && responsiveSource.includes("assertCommandState") && responsiveSource.includes("hashCommand")],
  ["action feedback handler", appSource.includes("[data-action]") && appSource.includes("setStatus") && appSource.includes("zr-action-flash")],
  ["setting-row command labels", appSource.includes('target.closest(".zr-module-setting")?.querySelector("span")?.textContent.trim()')],
  ["keyboard activation handler", appSource.includes('addEventListener("keydown"') && appSource.includes('button[data-action]') && appSource.includes('[role="button"]:not(button)') && responsiveSource.includes("assertKeyboardActivation")],
  ["field feedback handler", appSource.includes('addEventListener("focusin"') && appSource.includes('addEventListener("input"') && appSource.includes("Focused:") && appSource.includes("Edited:")],
  ["response counter hook", appSource.includes("dataset.zrResponseCount") && appSource.includes("dataset.zrLastResponse")],
  ["command route layer", appSource.includes("routeForCommand") && routesSource.includes("moduleRouteMap") && routesSource.includes("panelRouteMap")],
  ["module-scoped command route layer", routesSource.includes("moduleScopedRouteMap") && routesSource.includes('"material:compile"') && routesSource.includes('"behavior-tree:validate"') && responsiveSource.includes("assertModuleScopedCommandRoutes")],
  ["extension command route layer", routesSource.includes("extensionRouteForCommand") && routesSource.includes("extensionPanelKeyForCommand") && responsiveSource.includes("assertExtensionCommandRoutes") && responsiveSource.includes("module-bottom-shader-editor:validation")],
  ["all top-level toolbar command route audit", responsiveSource.includes("assertAllTopLevelToolbarCommandRoutes") && responsiveSource.includes("allTopLevelToolbarCommandRoutesExpression") && responsiveSource.includes("expectedTopLevelModuleTabs") && responsiveSource.includes("active/hash module mismatch") && responsiveSource.includes("active/hash panel mismatch")],
  ["all extension toolbar command route audit", responsiveSource.includes("assertAllExtensionToolbarCommandRoutes") && responsiveSource.includes("allExtensionToolbarCommandRoutesExpression") && responsiveSource.includes("expectedExtensionCards") && responsiveSource.includes("module-bottom-editor-library:routing-log") && responsiveSource.includes("extensionPanelKeyForToolbarCommand")],
  ["all extension internal control audit", responsiveSource.includes("all extension primary surface controls") && responsiveSource.includes("all extension right panel controls") && responsiveSource.includes("all extension bottom panel controls") && responsiveSource.includes("interactionRouteWrites") && responsiveSource.includes("capturedHistoryStates") && !responsiveSource.includes("representativeExtensionIds")],
  ["dedicated per-control route audit", controlRouteSource.includes("control route audit") && controlRouteSource.includes("auditIndexedControls") && controlRouteSource.includes("routeWrites") && controlRouteSource.includes("no route-state write after")],
  ["per-control route audit spans modules extensions and popup rows", controlRouteSource.includes("top-level module tabs") && controlRouteSource.includes("extension editor cards") && controlRouteSource.includes("module primary surfaces") && controlRouteSource.includes("all extension primary surface controls") && controlRouteSource.includes("popup menu rows")],
  ["command route module targets", routesSource.includes('"asset-browser"') && routesSource.includes('"behavior-tree"') && routesSource.includes('"vfx"') && routesSource.includes('"gameplay-ability"') && routesSource.includes('"render-pipeline"') && routesSource.includes('"hud-editor"')],
  ["command route panel targets", routesSource.includes("module-bottom-{module}:compile-log") && routesSource.includes("asset-right:metadata") && routesSource.includes("module-bottom-gameplay-tags:validation-log")],
  ["exhaustive button audit", responsiveSource.includes("auditIndexedControls") && responsiveSource.includes("responseCount()") && responsiveSource.includes(".zr-module-toolbar button")],
  ["exhaustive editable audit", responsiveSource.includes("editAndExpectResponse") && responsiveSource.includes("input:not([disabled])") && responsiveSource.includes("exerciseControl")],
  ["panel button audit", responsiveSource.includes(".zr-module-right .zr-panel-view.is-active") && responsiveSource.includes(".zr-module-bottom .zr-panel-view.is-active")],
  ["popup row audit", responsiveSource.includes(".zr-popup-layer .zr-menu-row") && responsiveSource.includes("dropdown popup open") && responsiveSource.includes("assertPopupMenuSelection") && responsiveSource.includes("[data-menu-item][data-action]") && responsiveSource.includes("for (const action of menuActions)")],
  ["toggle handler", appSource.includes('dataset.toggle === "switch"')],
  ["radio handler", appSource.includes("[data-radio]")],
  ["panel view handler", appSource.includes("dataset.panelView") && appSource.includes("panelTarget")],
  ["tree selection handler", appSource.includes("[data-tree-row]")],
  ["list/table selection handler", appSource.includes(".zr-module-list-row") && appSource.includes(".zr-module-table-row:not(.is-head)")],
  ["tool active handler", appSource.includes(".zr-topbar-tools .zr-icon-button")],
  ["dropdown placement handler", appSource.includes("getBoundingClientRect") && appSource.includes("popup.style.left")],
  ["aria selected update", appSource.includes('setAttribute("aria-selected", "true")')],
  ["no full workbench screenshot embed", !appHtml.includes("workbench.png")],
  ["no viewport raster embed", !appHtml.includes("workbench-viewport-reference.png")],
  ["module source has no obvious malformed placeholders", !/[<]\.00|1\.\.00|202\.26|2026\.26|graphLink\(27, ,|<\/ texture/.test(modulesSource)],
  ["module css has responsive module regions", modulesCssSource.includes(".zr-module-main") && modulesCssSource.includes("@media (max-width: 720px)")],
  ["module css has no obvious malformed color tuples", !/rgba\(\s*,|,\s*,|,,|\+\}/.test(modulesCssSource)]
];

const failed = checks.filter(([, passed]) => !passed);

for (const [name, passed] of checks) {
  console.log(`${passed ? "ok" : "fail"} ${name}`);
}

if (failed.length > 0) {
  console.error(`Interaction contract failed: ${failed.map(([name]) => name).join(", ")}`);
  process.exit(1);
}
