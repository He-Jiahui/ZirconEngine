import { readFileSync, readdirSync } from "node:fs";
import { inspector, popups, rail, scenePanel, showcase, statusbar, topbar, workbenchWindow } from "./surfaces.js";
import { defaultModuleId, extensionModules, modules, nativeModules, webModuleTabs, moduleWorkspace } from "./modules.js";
import {
  allReferenceSampleSources,
  coreReferenceSamples,
  extensionReferenceSamples,
  supplementalReferenceSamples
} from "./reference-samples.js";
import { extensionBlueprints } from "./extension-blueprints.js";

const appHtml = workbenchWindow([topbar(defaultModuleId), rail(defaultModuleId), moduleWorkspace(defaultModuleId), statusbar("Ready"), popups()]);
const surfacePanelHtml = [scenePanel(), inspector(), showcase()].join("\n");
const libraryHtml = moduleWorkspace("editor-library");
const extensionHtml = moduleWorkspace(extensionModules[0]?.id);
const coreAndLibraryWorkspaceHtml = webModuleTabs.map((module) => moduleWorkspace(module.id)).join("\n");
const extensionWorkspaceHtml = extensionModules.map((module) => moduleWorkspace(module.id)).join("\n");
const appSource = readFileSync(new URL("./app.js", import.meta.url), "utf8");
const indexSource = readFileSync(new URL("./index.html", import.meta.url), "utf8");
const layoutSource = readFileSync(new URL("./layout.js", import.meta.url), "utf8");
const layoutCssSource = readFileSync(new URL("./layout.css", import.meta.url), "utf8");
const surfacesSource = readFileSync(new URL("./surfaces.js", import.meta.url), "utf8");
const modulesSource = readFileSync(new URL("./modules.js", import.meta.url), "utf8");
const coreModuleBottomsSource = readFileSync(new URL("./core-module-bottoms.js", import.meta.url), "utf8");
const coreModuleCentersSource = readFileSync(new URL("./core-module-centers.js", import.meta.url), "utf8");
const coreModuleDetailsSource = readFileSync(new URL("./core-module-details.js", import.meta.url), "utf8");
const coreModuleLeftsSource = readFileSync(new URL("./core-module-lefts.js", import.meta.url), "utf8");
const coreModulesSource = readFileSync(new URL("./core-modules.js", import.meta.url), "utf8");
const extensionConfigsSource = readFileSync(new URL("./extension-configs.js", import.meta.url), "utf8");
const extensionModulesSource = readFileSync(new URL("./extension-modules.js", import.meta.url), "utf8");
const extensionSurfacesSource = readFileSync(new URL("./extension-surfaces.js", import.meta.url), "utf8");
const extensionLibrarySource = readFileSync(new URL("./extension-library.js", import.meta.url), "utf8");
const extensionHandoffSource = readFileSync(new URL("./extension-handoff.js", import.meta.url), "utf8");
const extensionBlueprintsSource = readFileSync(new URL("./extension-blueprints.js", import.meta.url), "utf8");
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
const extensionReferenceIds = extensionReferenceSamples.map((sample) => sample.source.replace(/^ai-|-layout\.png$/g, ""));
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
const nativeRightPanelIds = [
  "scene-right",
  "gameplay-right",
  "ability-right",
  "tags-right",
  "perception-right",
  "material-right",
  "behavior-right",
  "render-right",
  "asset-right",
  "vfx-right",
  "hud-right"
];
const coreBottomRendererNames = [
  "sceneBottom",
  "gameplayBottom",
  "abilityBottom",
  "tagsBottom",
  "perceptionBottom",
  "materialBottom",
  "behaviorBottom",
  "renderPipelineBottom",
  "assetBottom",
  "vfxBottom",
  "hudBottom"
];
const coreLeftRendererNames = [
  "sceneLeft",
  "gameplayLeft",
  "abilityLeft",
  "tagsLeft",
  "perceptionLeft",
  "materialLeft",
  "behaviorLeft",
  "renderPipelineLeft",
  "assetLeft",
  "vfxLeft",
  "hudLeft"
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
  ["extension reference blueprints mapped one-to-one", Object.keys(extensionBlueprints).length === expectedExtensionCount && extensionReferenceIds.every((id) => extensionBlueprints[id]) && Object.keys(extensionBlueprints).every((id) => extensionReferenceIds.includes(id)) && extensionModules.every((module) => module.blueprint === true)],
  ["supplemental reference samples classified", supplementalReferenceSamples.length === 4 && supplementalReferenceSamples.every((sample) => modules.some((module) => module.id === sample.moduleId) && ["variant", "shell-style"].includes(sample.role))],
  ["native module registry has eleven editor modules", nativeModules.length === 11 && ["scene", "gameplay-effect", "gameplay-ability", "gameplay-tags", "ai-perception", "material", "behavior-tree", "render-pipeline", "asset-browser", "vfx", "hud-editor"].every((id) => nativeModules.some((module) => module.id === id))],
  ["extension module registry", extensionModules.length === expectedExtensionCount && modules.length === nativeModules.length + extensionModules.length + 1 && webModuleTabs.length === nativeModules.length + 1 && representativeExtensionIds.every((id) => extensionModules.some((module) => module.id === id))],
  ["core module registry boundary", modulesSource.includes("./core-modules.js") && !modulesSource.includes("const coreModules") && coreModulesSource.includes("export const coreModules") && ["scene", "gameplay-effect", "gameplay-ability", "gameplay-tags", "ai-perception", "material", "behavior-tree", "render-pipeline", "asset-browser", "vfx", "hud-editor"].every((id) => coreModulesSource.includes(`id: "${id}"`)) && ["Left", "Center", "Details", "Bottom"].every((suffix) => coreModulesSource.includes(suffix))],
  ["extension config generator boundary", extensionModulesSource.includes("./extension-configs.js") && !extensionModulesSource.includes("recipeByKind") && !extensionModulesSource.includes("./reference-samples.js") && !extensionModulesSource.includes("./extension-blueprints.js") && extensionConfigsSource.includes("export const extensionModuleConfigs") && extensionConfigsSource.includes("./reference-samples.js") && extensionConfigsSource.includes("./extension-blueprints.js")],
  ["extension module generator boundary", modulesSource.includes("./extension-modules.js") && modulesSource.includes("buildExtensionModules(coreModules, defaultModuleId)") && extensionModulesSource.includes("export function buildExtensionModules") && extensionModulesSource.includes("createExtensionModule") && extensionModulesSource.includes("extensionBottomOutput")],
  ["extension surface renderer boundary", extensionModulesSource.includes("./extension-surfaces.js") && !/function extension(?:Left|Center|Details|BottomOutput|PrimaryPanel|ValidationPanel|ReferencesPanel)\(/.test(extensionModulesSource) && ["extensionLeft", "extensionCenter", "extensionDetails", "extensionBottomOutput"].every((name) => extensionSurfacesSource.includes(`export function ${name}`)) && extensionSurfacesSource.includes("panelGroup(`${config.id}-right`") && extensionSurfacesSource.includes("panelGroup(`module-bottom-${config.id}`") && extensionSurfacesSource.includes('data-extension-blueprint="${config.blueprint ? "reference" : "recipe"}')],
  ["extension library module boundary", extensionModulesSource.includes("./extension-library.js") && extensionLibrarySource.includes("export function createEditorLibraryModule") && extensionLibrarySource.includes("data-library-drilldown=\"reference-blueprints\"")],
  ["extension handoff panel boundary", extensionSurfacesSource.includes("./extension-handoff.js") && extensionHandoffSource.includes("export function extensionHandoffPanel") && extensionHandoffSource.includes("pending .zui workspace")],
  ["extension blueprint data boundary", extensionBlueprintsSource.includes("export const extensionBlueprints") && extensionBlueprintsSource.includes('"shader-editor"') && extensionBlueprintsSource.includes('"source-control"') && extensionBlueprintsSource.includes('"weather-editor"') && extensionBlueprintsSource.includes("function tablePrimary")],
  ["extension module category recipes", extensionConfigsSource.includes("recipeByKind") && ["world", "rendering", "animation", "ui", "production", "diagnostics", "online", "simulation", "data", "gameplay", "runtime", "vfx"].every((kind) => extensionConfigsSource.includes(`${kind}:`))],
  ["extension module varied layouts", extensionHtml.includes("is-extension is-extension-world") && moduleWorkspace("shader-editor").includes("is-extension-rendering") && moduleWorkspace("sequencer").includes("is-extension-animation") && moduleWorkspace("source-control").includes("is-extension-production") && moduleWorkspace("weather-editor").includes("is-extension-world")],
  ["module shared component layer", modulesSource.includes("./module-components.js") && moduleComponentsSource.includes("export function moduleMain") && moduleComponentsSource.includes("export function moduleTable")],
  ["core module left boundary", coreModulesSource.includes("./core-module-lefts.js") && !/left:\s*\(\)\s*=>\s*\[/.test(modulesSource) && !/left:\s*\(\)\s*=>\s*[\s\S]{0,60}panel\(/.test(modulesSource) && coreLeftRendererNames.every((name) => coreModuleLeftsSource.includes(`export function ${name}`)) && coreModuleLeftsSource.includes('panel("Widget Palette"') && coreModuleLeftsSource.includes("panelGroup(\"hud-assets\"")],
  ["core module bottom boundary", coreModulesSource.includes("./core-module-bottoms.js") && !/^function \w+Bottom\(/m.test(modulesSource) && coreBottomRendererNames.every((name) => coreModuleBottomsSource.includes(`export function ${name}`)) && coreModuleBottomsSource.includes("zr-module-output-grid")],
  ["core module center boundary", coreModulesSource.includes("./core-module-centers.js") && !/^function \w+Center\(/m.test(modulesSource) && !/function (?:perceptionMap|hudCanvas)\(/.test(modulesSource) && ["sceneCenter", "gameplayCenter", "abilityCenter", "tagsCenter", "perceptionCenter", "materialCenter", "behaviorCenter", "renderPipelineCenter", "assetCenter", "vfxCenter", "hudCenter"].every((name) => coreModuleCentersSource.includes(`export function ${name}`)) && coreModuleCentersSource.includes("function perceptionMap") && coreModuleCentersSource.includes("function hudCanvas")],
  ["core module detail boundary", coreModulesSource.includes("./core-module-details.js") && !/^function \w+Details\(/m.test(modulesSource) && nativeRightPanelIds.every((panel) => coreModuleDetailsSource.includes(`panelGroup("${panel}"`)) && ["sceneDetails", "gameplayDetails", "abilityDetails", "tagsDetails", "perceptionDetails", "materialDetails", "behaviorDetails", "renderPipelineDetails", "assetDetails", "vfxDetails", "hudDetails"].every((name) => coreModuleDetailsSource.includes(`export function ${name}`))],
  ["shared panel group component", moduleComponentsSource.includes("export function panelGroup") && moduleComponentsSource.includes('data-panel-group="${esc(panel)}"') && moduleComponentsSource.includes("panelGroup(panelId") && extensionLibrarySource.includes('panelGroup("library-drilldown"') && extensionLibrarySource.includes('panelGroup("library-right"') && extensionSurfacesSource.includes("panelGroup(`${config.id}-right`") && extensionSurfacesSource.includes("panelGroup(`module-bottom-${config.id}`")],
  ["bottom output uses shared panel group", moduleComponentsSource.includes("export function bottomOutput") && moduleComponentsSource.includes("panelGroup(panelId") && !moduleComponentsSource.includes("zr-module-bottom-body") && coreAndLibraryWorkspaceHtml.includes('data-panel-group="module-bottom-gameplay-effect"') && coreAndLibraryWorkspaceHtml.includes('class="zr-panel-group is-module-bottom"')],
  ["native right panels use shared panel group", coreModuleDetailsSource.includes("panelGroup") && nativeRightPanelIds.every((panel) => coreAndLibraryWorkspaceHtml.includes(`data-panel-group="${panel}"`) && coreAndLibraryWorkspaceHtml.includes(`data-panel-view="${panel}:`)) && !/function \w+Details\(\) \{\s*return `\$\{panelTabs/.test(coreModuleDetailsSource)],
  ["surface panels use shared panel group", surfacesSource.includes("./module-components.js") && surfacesSource.includes('panelGroup("scene"') && surfacesSource.includes('panelGroup("inspector"') && surfacesSource.includes('panelGroup("showcase"') && !/function panel(?:Tabs|View)\b/.test(surfacesSource) && ["scene", "inspector", "showcase"].every((panel) => surfacePanelHtml.includes(`data-panel-group="${panel}"`) && surfacePanelHtml.includes(`data-panel-host="${panel}"`))],
  ["library and extension right panels use shared panel group", libraryHtml.includes('data-panel-group="library-right"') && libraryHtml.includes('data-panel-view="library-right:routing"') && extensionHtml.includes(`data-panel-group="${extensionModules[0]?.id}-right"`) && extensionWorkspaceHtml.includes('class="zr-panel-group is-extension-right"') && !/panel(?:Tabs|View)\(/.test(extensionLibrarySource) && !/panel(?:Tabs|View)\(/.test(extensionModulesSource) && !/panel(?:Tabs|View)\(/.test(extensionSurfacesSource)],
  ["module embedded card tabs use shared panel group", !modulesSource.includes("panelTabs") && coreModuleLeftsSource.includes('panelGroup("tag-sources"') && coreModuleLeftsSource.includes('panelGroup("hud-assets"') && ["tag-sources:plugins", "tag-sources:native-sets", "hud-assets:screens"].every((target) => coreAndLibraryWorkspaceHtml.includes(`data-panel-tab="${target}"`) && coreAndLibraryWorkspaceHtml.includes(`data-panel-view="${target}"`))],
  ["semantic module table action ids", moduleComponentsSource.includes("function tableRowActionId") && moduleComponentsSource.includes("function tableRowLabel") && moduleComponentsSource.includes("readableCell") && moduleComponentsSource.includes("aria-label") && moduleComponentsSource.includes("!/^\\d+(?:\\.\\d+)?$/")],
  ["module workspace uses layout primitives", appHtml.includes("zr-module-editor-grid") && appHtml.includes("zr-layout") && appHtml.includes("zr-grid")],
  ["toolbar uses cluster primitive", appHtml.includes("zr-topbar-tools") && appHtml.includes("zr-cluster")],
  ["module tabs in topbar", appHtml.includes("zr-module-tabs") && (appHtml.match(/class="zr-module-tab(?:\s|")/g) ?? []).length === webModuleTabs.length],
  ["extension module library cards", libraryHtml.includes("zr-extension-card-grid") && (libraryHtml.match(/data-module-source="extension-library"/g) ?? []).length === extensionModules.length],
  ["extension library blueprint drilldown", libraryHtml.includes('data-library-drilldown="reference-blueprints"') && libraryHtml.includes('data-panel-group="library-drilldown"') && libraryHtml.includes('data-panel-tab="library-drilldown:blueprints"') && libraryHtml.includes('data-panel-view="library-drilldown:components"') && libraryHtml.includes("shader-editor") && libraryHtml.includes("world-state") && extensionLibrarySource.includes("representativeConfigs")],
  ["core secondary bottom panels generated", webModuleTabs.every((module) => {
    const workspace = moduleWorkspace(module.id);
    const tabCount = (workspace.match(new RegExp(`data-panel-tab="module-bottom-${module.id}:[^"]+"`, "g")) ?? []).length;
    const generatedCount = (workspace.match(new RegExp(`data-generated-bottom-panel="module-bottom-${module.id}:[^"]+"`, "g")) ?? []).length;
    return tabCount > 1 && generatedCount === tabCount - 1 && !workspace.includes("zr-module-placeholder");
  }) && moduleComponentsSource.includes("function generatedBottomPanel") && coreAndLibraryWorkspaceHtml.includes("data-generated-bottom-panel")],
  ["extension module surfaces", extensionHtml.includes("zr-module-editor-grid is-extension") && extensionHtml.includes('data-extension-blueprint="reference"') && extensionHtml.includes("Reference Specific") && extensionHtml.includes("Prototype Only") && extensionHtml.includes("More Editors")],
  ["extension bottom panels have specific content", extensionModules.every((module) => {
    const workspace = moduleWorkspace(module.id);
    return ["output", "validation", "references", "handoff"].every((panel) => workspace.includes(`data-panel-view="module-bottom-${module.id}:${panel}"`)) && workspace.includes("Reference assets") && workspace.includes("Native Handoff");
  }) && !extensionWorkspaceHtml.match(/data-panel-view="module-bottom-[^"]+:(validation|references)"[\s\S]{0,900}zr-module-placeholder/)],
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
  ["extension command route layer", routesSource.includes("extensionRouteForCommand") && routesSource.includes("extensionPanelKeyForCommand") && routesSource.includes('"handoff"') && responsiveSource.includes("assertExtensionCommandRoutes") && responsiveSource.includes("module-bottom-shader-editor:validation") && responsiveSource.includes("module-bottom-shader-editor:handoff")],
  ["all top-level toolbar command route audit", responsiveSource.includes("assertAllTopLevelToolbarCommandRoutes") && responsiveSource.includes("allTopLevelToolbarCommandRoutesExpression") && responsiveSource.includes("expectedTopLevelModuleTabs") && responsiveSource.includes("active/hash module mismatch") && responsiveSource.includes("active/hash panel mismatch")],
  ["all extension toolbar command route audit", responsiveSource.includes("assertAllExtensionToolbarCommandRoutes") && responsiveSource.includes("allExtensionToolbarCommandRoutesExpression") && responsiveSource.includes("expectedExtensionCards") && responsiveSource.includes("module-bottom-editor-library:routing-log") && responsiveSource.includes("extensionPanelKeyForToolbarCommand")],
  ["all extension internal control audit", responsiveSource.includes("all extension primary surface controls") && responsiveSource.includes("all extension right panel controls") && responsiveSource.includes("all extension bottom panel controls") && responsiveSource.includes("interactionRouteWrites") && responsiveSource.includes("capturedHistoryStates") && !responsiveSource.includes("representativeExtensionIds")],
  ["dedicated per-control route audit", controlRouteSource.includes("control route audit") && controlRouteSource.includes("auditIndexedControls") && controlRouteSource.includes("routeWrites") && controlRouteSource.includes("no route-state write after")],
  ["per-control route audit has bounded batched execution", controlRouteSource.includes("routeAuditTimeoutMs") && controlRouteSource.includes("checkDeadline") && controlRouteSource.includes("needsRestore") && controlRouteSource.includes("route audit deadline exceeded")],
  ["per-control route audit spans modules extensions and popup rows", controlRouteSource.includes("top-level module tabs") && controlRouteSource.includes("extension editor cards") && controlRouteSource.includes("module primary surfaces") && controlRouteSource.includes("all extension primary surface controls") && controlRouteSource.includes("popup menu rows")],
  ["responsive interaction audit has bounded batched execution", responsiveSource.includes("responsiveInteractionTimeoutMs") && responsiveSource.includes("checkDeadline") && responsiveSource.includes("auditedGroups") && responsiveSource.includes("responsive interaction audit deadline exceeded")],
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
