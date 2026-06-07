import { readFileSync, readdirSync } from "node:fs";
import { inspector, popups, rail, scenePanel, showcase, statusbar, topbar, workbenchWindow } from "./src/components/surfaces/surfaces.js";
import { defaultModuleId, extensionModules, modules, nativeModules, webModuleTabs, moduleWorkspace } from "./src/modules/modules.js";
import {
  allReferenceSampleSources,
  coreReferenceSamples,
  extensionReferenceSamples,
  supplementalReferenceSamples
} from "./src/foundation/reference-samples.js";
import { extensionBlueprints } from "./src/modules/extensions/extension-blueprints.js";

const appHtml = workbenchWindow([topbar(defaultModuleId), rail(defaultModuleId), moduleWorkspace(defaultModuleId), statusbar("Ready"), popups()]);
const surfacePanelHtml = [scenePanel(), inspector(), showcase()].join("\n");
const libraryHtml = moduleWorkspace("editor-library");
const extensionHtml = moduleWorkspace(extensionModules[0]?.id);
const coreAndLibraryWorkspaceHtml = webModuleTabs.map((module) => moduleWorkspace(module.id)).join("\n");
const extensionWorkspaceHtml = extensionModules.map((module) => moduleWorkspace(module.id)).join("\n");
const appRolePaths = [
  "./src/app/mount.js",
  "./src/app/controller.js",
  "./src/app/controller/activation.js",
  "./src/app/controller/activation/factory.js",
  "./src/app/controller/activation/module.js",
  "./src/app/controller/activation/panel.js",
  "./src/app/controller/activation/reset.js",
  "./src/app/controller/command-application.js",
  "./src/app/controller/command-application/apply.js",
  "./src/app/controller/command-application/module.js",
  "./src/app/controller/command-application/panel.js",
  "./src/app/controller/command-application/record.js",
  "./src/app/controller/command-application/status.js",
  "./src/app/controller/create-workbench-controller.js",
  "./src/app/controller/command-routing.js",
  "./src/app/controller/command-routing/explicit.js",
  "./src/app/controller/command-routing/fallback.js",
  "./src/app/controller/command-routing/label.js",
  "./src/app/controller/command-routing/resolve.js",
  "./src/app/controller/history.js",
  "./src/app/controller/location-state.js",
  "./src/app/controller/location-state/apply.js",
  "./src/app/controller/location-state/module.js",
  "./src/app/controller/location-state/panel.js",
  "./src/app/controller/location-state/request.js",
  "./src/app/controller/location-state/status.js",
  "./src/app/controller/rendering.js",
  "./src/app/controller/state.js",
  "./src/app/controller/status.js",
  "./src/app/controller/workbench/commands.js",
  "./src/app/controller/workbench/location.js",
  "./src/app/controller/workbench/render-loop.js",
  "./src/app/controller/workbench/route-sync.js",
  "./src/app/route-state.js",
  "./src/app/labels.js",
  "./src/app/interactions/click.js",
  "./src/app/interactions/click/bind.js",
  "./src/app/interactions/click/dispatch.js",
  "./src/app/interactions/click/handlers.js",
  "./src/app/interactions/click/actions.js",
  "./src/app/interactions/click/actions/feedback.js",
  "./src/app/interactions/click/actions/group.js",
  "./src/app/interactions/click/actions/handle.js",
  "./src/app/interactions/click/actions/menu.js",
  "./src/app/interactions/click/actions/target.js",
  "./src/app/interactions/click/dropdowns.js",
  "./src/app/interactions/click/dropdowns/dismissal.js",
  "./src/app/interactions/click/dropdowns/feedback.js",
  "./src/app/interactions/click/dropdowns/placement.js",
  "./src/app/interactions/click/dropdowns/state.js",
  "./src/app/interactions/click/dropdowns/target.js",
  "./src/app/interactions/click/dropdowns/trigger.js",
  "./src/app/interactions/click/generic.js",
  "./src/app/interactions/click/generic/feedback.js",
  "./src/app/interactions/click/generic/handle.js",
  "./src/app/interactions/click/generic/target.js",
  "./src/app/interactions/click/navigation.js",
  "./src/app/interactions/click/navigation/activate.js",
  "./src/app/interactions/click/navigation/handle.js",
  "./src/app/interactions/click/navigation/target.js",
  "./src/app/interactions/click/rows.js",
  "./src/app/interactions/click/rows/data.js",
  "./src/app/interactions/click/rows/feedback.js",
  "./src/app/interactions/click/rows/selection.js",
  "./src/app/interactions/click/rows/tree.js",
  "./src/app/interactions/click/selection.js",
  "./src/app/interactions/click/selection/feedback.js",
  "./src/app/interactions/click/selection/radio.js",
  "./src/app/interactions/click/selection/state.js",
  "./src/app/interactions/click/selection/target.js",
  "./src/app/interactions/click/selection/toggle.js",
  "./src/app/interactions/click/tabs.js",
  "./src/app/interactions/click/tabs/feedback.js",
  "./src/app/interactions/click/tabs/handle.js",
  "./src/app/interactions/click/tabs/panel.js",
  "./src/app/interactions/click/tabs/state.js",
  "./src/app/interactions/click/tabs/target.js",
  "./src/app/interactions/click/toolbar.js",
  "./src/app/interactions/click/toolbar/feedback.js",
  "./src/app/interactions/click/toolbar/rail.js",
  "./src/app/interactions/click/toolbar/state.js",
  "./src/app/interactions/click/toolbar/target.js",
  "./src/app/interactions/click/toolbar/tool.js",
  "./src/app/interactions/click/utils.js",
  "./src/app/interactions/fields.js",
  "./src/app/interactions/fields/bind.js",
  "./src/app/interactions/fields/focus.js",
  "./src/app/interactions/fields/input.js",
  "./src/app/interactions/fields/target.js",
  "./src/app/interactions/keyboard.js",
  "./src/app/interactions/keyboard/activate.js",
  "./src/app/interactions/keyboard/bind.js",
  "./src/app/interactions/keyboard/filter.js",
  "./src/app/interactions/keyboard/target.js",
  "./src/app/interactions/history/bind.js",
  "./src/app/interactions/history/events.js",
  "./src/app/interactions/history.js"
];
const appEntrySource = readFileSync(new URL("./app.js", import.meta.url), "utf8");
const appControllerEntrySource = readFileSync(new URL("./src/app/controller.js", import.meta.url), "utf8");
const appActivationEntrySource = readFileSync(new URL("./src/app/controller/activation.js", import.meta.url), "utf8");
const appCommandApplicationEntrySource = readFileSync(new URL("./src/app/controller/command-application.js", import.meta.url), "utf8");
const appCommandRoutingEntrySource = readFileSync(new URL("./src/app/controller/command-routing.js", import.meta.url), "utf8");
const appClickInteractionsEntrySource = readFileSync(new URL("./src/app/interactions/click.js", import.meta.url), "utf8");
const appActionInteractionsEntrySource = readFileSync(new URL("./src/app/interactions/click/actions.js", import.meta.url), "utf8");
const appDropdownInteractionsEntrySource = readFileSync(new URL("./src/app/interactions/click/dropdowns.js", import.meta.url), "utf8");
const appFieldInteractionsEntrySource = readFileSync(new URL("./src/app/interactions/fields.js", import.meta.url), "utf8");
const appGenericInteractionsEntrySource = readFileSync(new URL("./src/app/interactions/click/generic.js", import.meta.url), "utf8");
const appKeyboardInteractionsEntrySource = readFileSync(new URL("./src/app/interactions/keyboard.js", import.meta.url), "utf8");
const appHistoryInteractionsEntrySource = readFileSync(new URL("./src/app/interactions/history.js", import.meta.url), "utf8");
const appNavigationInteractionsEntrySource = readFileSync(new URL("./src/app/interactions/click/navigation.js", import.meta.url), "utf8");
const appRowInteractionsEntrySource = readFileSync(new URL("./src/app/interactions/click/rows.js", import.meta.url), "utf8");
const appSelectionInteractionsEntrySource = readFileSync(new URL("./src/app/interactions/click/selection.js", import.meta.url), "utf8");
const appTabInteractionsEntrySource = readFileSync(new URL("./src/app/interactions/click/tabs.js", import.meta.url), "utf8");
const appToolbarInteractionsEntrySource = readFileSync(new URL("./src/app/interactions/click/toolbar.js", import.meta.url), "utf8");
const appLocationStateEntrySource = readFileSync(new URL("./src/app/controller/location-state.js", import.meta.url), "utf8");
const appWorkbenchControllerEntrySource = readFileSync(new URL("./src/app/controller/create-workbench-controller.js", import.meta.url), "utf8");
const appSource = [
  appEntrySource,
  ...appRolePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const indexSource = readFileSync(new URL("./index.html", import.meta.url), "utf8");
const layoutSource = readFileSync(new URL("./src/foundation/layout.js", import.meta.url), "utf8");
const layoutCssSource = readFileSync(new URL("./src/foundation/layout.css", import.meta.url), "utf8");
const foundationTokenCssEntrySource = readFileSync(new URL("./src/foundation/tokens.css", import.meta.url), "utf8");
const foundationTokenCssRolePaths = [
  "./tokens/dimensions.css",
  "./tokens/typography.css",
  "./tokens/palette.css",
  "./tokens/effects.css",
  "./tokens/shape-controls.css",
  "./tokens/gaps.css",
  "./tokens/base.css"
];
const foundationTokenCssSource = foundationTokenCssRolePaths
  .map((path) => readFileSync(new URL(`./src/foundation/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const foundationResponsiveCssEntrySource = readFileSync(new URL("./src/foundation/responsive.css", import.meta.url), "utf8");
const foundationResponsiveCssRolePaths = [
  "./responsive/wide-shell.css",
  "./responsive/wide-panels.css",
  "./responsive/tablet-shell.css",
  "./responsive/tablet-panels.css",
  "./responsive/mobile-shell.css",
  "./responsive/mobile-panels.css",
  "./responsive/compact-controls.css"
];
const foundationResponsiveCssSource = foundationResponsiveCssRolePaths
  .map((path) => readFileSync(new URL(`./src/foundation/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const inputAggregatorSource = readFileSync(new URL("./src/components/inputs/atoms.js", import.meta.url), "utf8");
const inputAtomsSource = [
  "./src/components/inputs/atoms.js",
  "./src/components/inputs/input-utils.js",
  "./src/components/inputs/buttons.js",
  "./src/components/inputs/buttons/button.js",
  "./src/components/inputs/buttons/icon-button.js",
  "./src/components/inputs/fields.js",
  "./src/components/inputs/fields/input.js",
  "./src/components/inputs/fields/search-input.js",
  "./src/components/inputs/fields/number-field.js",
  "./src/components/inputs/selection-controls.js",
  "./src/components/inputs/selection-controls/checkbox.js",
  "./src/components/inputs/selection-controls/radio.js",
  "./src/components/inputs/selection-controls/toggle.js",
  "./src/components/inputs/tabs.js",
  "./src/components/inputs/dropdowns.js",
  "./src/components/inputs/dropdowns/select.js",
  "./src/components/inputs/sliders.js",
  "./src/components/inputs/sliders/slider.js",
  "./src/components/inputs/sliders/range-slider.js",
].map((path) => readFileSync(new URL(path, import.meta.url), "utf8")).join("\n");
const surfaceAggregatorSource = readFileSync(new URL("./src/components/surfaces/surfaces.js", import.meta.url), "utf8");
const surfacesSource = [
  "./src/components/surfaces/surfaces.js",
  "./src/components/surfaces/shell/window.js",
  "./src/components/surfaces/shell/chrome.js",
  "./src/components/surfaces/panels/drawer-surface.js",
  "./src/components/surfaces/panels/scene-panel.js",
  "./src/components/surfaces/panels/inspector-panel.js",
  "./src/components/surfaces/panels/showcase-panel.js",
  "./src/components/surfaces/viewport/viewport-surface.js",
  "./src/components/overlays/popup-layer.js",
].map((path) => readFileSync(new URL(path, import.meta.url), "utf8")).join("\n");
const overlayMenuSource = readFileSync(new URL("./src/components/overlays/menu.js", import.meta.url), "utf8");
const overlaySource = [
  overlayMenuSource,
  readFileSync(new URL("./src/components/overlays/menu/row.js", import.meta.url), "utf8"),
].join("\n");
const surfaceShellPanelCssEntrySource = readFileSync(new URL("./src/components/surfaces/surfaces.css", import.meta.url), "utf8");
const surfaceShellPanelCssRolePaths = [
  "./shell/window.css",
  "./shell/topbar.css",
  "./shell/rail.css",
  "./panels/base.css",
  "./panels/scene.css"
];
const surfaceShellPanelCssSource = surfaceShellPanelCssRolePaths
  .map((path) => readFileSync(new URL(`./src/components/surfaces/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const inspectorSurfaceCssEntrySource = readFileSync(new URL("./src/components/surfaces/inspector.css", import.meta.url), "utf8");
const inspectorSurfaceCssRolePaths = [
  "./panels/inspector/layout.css",
  "./panels/inspector/object-header.css",
  "./panels/inspector/sections.css",
  "./panels/inspector/fields.css",
  "./panels/inspector/resources.css"
];
const inspectorSurfaceCssSource = inspectorSurfaceCssRolePaths
  .map((path) => readFileSync(new URL(`./src/components/surfaces/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const showcaseSurfaceCssEntrySource = readFileSync(new URL("./src/components/surfaces/showcase.css", import.meta.url), "utf8");
const showcaseSurfaceCssRolePaths = [
  "./panels/showcase/layout.css",
  "./panels/showcase/grid.css",
  "./panels/showcase/columns.css",
  "./panels/showcase/stacks.css"
];
const showcaseSurfaceCssSource = showcaseSurfaceCssRolePaths
  .map((path) => readFileSync(new URL(`./src/components/surfaces/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const statusSurfaceCssEntrySource = readFileSync(new URL("./src/components/surfaces/status.css", import.meta.url), "utf8");
const statusSurfaceCssRolePaths = [
  "./status/bar.css",
  "./status/groups.css",
  "./status/controls.css",
  "./status/indicators.css"
];
const statusSurfaceCssSource = statusSurfaceCssRolePaths
  .map((path) => readFileSync(new URL(`./src/components/surfaces/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const viewportCssRolePaths = [
  "./src/components/surfaces/viewport/base.css",
  "./src/components/surfaces/viewport/lighting.css",
  "./src/components/surfaces/viewport/structure.css",
  "./src/components/surfaces/viewport/floor.css",
  "./src/components/surfaces/viewport/props.css",
  "./src/components/surfaces/viewport/tools.css"
];
const viewportCssEntrySource = readFileSync(new URL("./src/components/surfaces/viewport.css", import.meta.url), "utf8");
const viewportCssSource = viewportCssRolePaths
  .map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
  .join("\n");
const viewportLightingCssEntrySource = readFileSync(new URL("./src/components/surfaces/viewport/lighting.css", import.meta.url), "utf8");
const viewportLightingCssRolePaths = [
  "./lighting/lightwash.css",
  "./lighting/shadows.css"
];
const viewportLightingCssSource = viewportLightingCssRolePaths
  .map((path) => readFileSync(new URL(`./src/components/surfaces/viewport/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const viewportFloorCssEntrySource = readFileSync(new URL("./src/components/surfaces/viewport/floor.css", import.meta.url), "utf8");
const viewportFloorCssRolePaths = [
  "./floor/base.css",
  "./floor/grid.css",
  "./floor/reflections.css",
  "./floor/grates.css",
  "./floor/panels.css",
  "./floor/seams.css"
];
const viewportFloorCssSource = viewportFloorCssRolePaths
  .map((path) => readFileSync(new URL(`./src/components/surfaces/viewport/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const viewportStructureCssEntrySource = readFileSync(new URL("./src/components/surfaces/viewport/structure.css", import.meta.url), "utf8");
const viewportStructureCssRolePaths = [
  "./structure/wall.css",
  "./structure/ceiling-door.css",
  "./structure/fixtures.css",
  "./structure/side-walls.css",
  "./structure/rails.css"
];
const viewportStructureCssSource = viewportStructureCssRolePaths
  .map((path) => readFileSync(new URL(`./src/components/surfaces/viewport/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const viewportPropsCssEntrySource = readFileSync(new URL("./src/components/surfaces/viewport/props.css", import.meta.url), "utf8");
const viewportPropsCssRolePaths = [
  "./props/cargo.css",
  "./props/crate.css",
  "./props/selection.css",
  "./props/transform.css"
];
const viewportPropsCssSource = viewportPropsCssRolePaths
  .map((path) => readFileSync(new URL(`./src/components/surfaces/viewport/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const viewportToolsCssEntrySource = readFileSync(new URL("./src/components/surfaces/viewport/tools.css", import.meta.url), "utf8");
const viewportToolsCssRolePaths = [
  "./tools/axis-mini.css",
  "./tools/orientation-gizmo.css",
  "./tools/vignette.css",
  "./tools/toolbar.css"
];
const viewportToolsCssSource = viewportToolsCssRolePaths
  .map((path) => readFileSync(new URL(`./src/components/surfaces/viewport/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const viewportCssCombinedSource = [
  viewportCssSource,
  viewportLightingCssSource,
  viewportFloorCssSource,
  viewportStructureCssSource,
  viewportPropsCssSource,
  viewportToolsCssSource
].join("\n");
const workbenchModuleRolePaths = [
  "./src/modules/workbench/registry.js",
  "./src/modules/workbench/navigation.js",
  "./src/modules/workbench/toolbar.js",
  "./src/modules/workbench/rail.js",
  "./src/modules/workbench/workspace.js"
];
const modulesEntrySource = readFileSync(new URL("./src/modules/modules.js", import.meta.url), "utf8");
const modulesSource = [
  modulesEntrySource,
  ...workbenchModuleRolePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreBottomDomainPaths = [
  "./src/modules/core/bottoms/index.js",
  "./src/modules/core/bottoms/gameplay.js",
  "./src/modules/core/bottoms/ai.js",
  "./src/modules/core/bottoms/rendering.js",
  "./src/modules/core/bottoms/assets.js",
  "./src/modules/core/bottoms/ui.js",
  "./src/modules/core/bottoms/routes.js"
];
const coreModuleBottomsEntrySource = readFileSync(new URL("./src/modules/core/core-module-bottoms.js", import.meta.url), "utf8");
const coreGameplayBottomEntrySource = readFileSync(new URL("./src/modules/core/bottoms/gameplay.js", import.meta.url), "utf8");
const coreGameplayBottomModulePaths = [
  "./src/modules/core/bottoms/gameplay/effect.js",
  "./src/modules/core/bottoms/gameplay/ability.js",
  "./src/modules/core/bottoms/gameplay/tags.js"
];
const coreGameplayBottomSource = [
  coreGameplayBottomEntrySource,
  ...coreGameplayBottomModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreAiBottomEntrySource = readFileSync(new URL("./src/modules/core/bottoms/ai.js", import.meta.url), "utf8");
const coreAiBottomModulePaths = [
  "./src/modules/core/bottoms/ai/perception.js",
  "./src/modules/core/bottoms/ai/behavior.js"
];
const coreAiBottomSource = [
  coreAiBottomEntrySource,
  ...coreAiBottomModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreRenderingBottomEntrySource = readFileSync(new URL("./src/modules/core/bottoms/rendering.js", import.meta.url), "utf8");
const coreRenderingBottomModulePaths = [
  "./src/modules/core/bottoms/rendering/material.js",
  "./src/modules/core/bottoms/rendering/render-pipeline.js",
  "./src/modules/core/bottoms/rendering/vfx.js"
];
const coreRenderingBottomSource = [
  coreRenderingBottomEntrySource,
  ...coreRenderingBottomModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreModuleBottomsSource = [
  coreModuleBottomsEntrySource,
  ...coreBottomDomainPaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8")),
  coreGameplayBottomSource,
  coreAiBottomSource,
  coreRenderingBottomSource
].join("\n");
const coreCenterDomainPaths = [
  "./src/modules/core/centers/index.js",
  "./src/modules/core/centers/gameplay.js",
  "./src/modules/core/centers/ai.js",
  "./src/modules/core/centers/rendering.js",
  "./src/modules/core/centers/assets.js",
  "./src/modules/core/centers/ui.js"
];
const coreGameplayCenterEntrySource = readFileSync(new URL("./src/modules/core/centers/gameplay.js", import.meta.url), "utf8");
const coreGameplayCenterModulePaths = [
  "./src/modules/core/centers/gameplay/effect.js",
  "./src/modules/core/centers/gameplay/ability.js",
  "./src/modules/core/centers/gameplay/tags.js"
];
const coreGameplayCenterSource = [
  coreGameplayCenterEntrySource,
  ...coreGameplayCenterModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreAiCenterEntrySource = readFileSync(new URL("./src/modules/core/centers/ai.js", import.meta.url), "utf8");
const coreAiCenterModulePaths = [
  "./src/modules/core/centers/ai/perception.js",
  "./src/modules/core/centers/ai/behavior.js"
];
const coreAiCenterSource = [
  coreAiCenterEntrySource,
  ...coreAiCenterModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreRenderingCenterEntrySource = readFileSync(new URL("./src/modules/core/centers/rendering.js", import.meta.url), "utf8");
const coreRenderingCenterModulePaths = [
  "./src/modules/core/centers/rendering/material.js",
  "./src/modules/core/centers/rendering/render-pipeline.js",
  "./src/modules/core/centers/rendering/vfx.js"
];
const coreRenderingCenterSource = [
  coreRenderingCenterEntrySource,
  ...coreRenderingCenterModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreModuleCentersEntrySource = readFileSync(new URL("./src/modules/core/core-module-centers.js", import.meta.url), "utf8");
const coreModuleCentersSource = [
  coreModuleCentersEntrySource,
  ...coreCenterDomainPaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8")),
  coreGameplayCenterSource,
  coreAiCenterSource,
  coreRenderingCenterSource
].join("\n");
const coreDetailDomainPaths = [
  "./src/modules/core/details/index.js",
  "./src/modules/core/details/gameplay.js",
  "./src/modules/core/details/ai.js",
  "./src/modules/core/details/rendering.js",
  "./src/modules/core/details/assets.js",
  "./src/modules/core/details/ui.js",
  "./src/modules/core/details/routes.js"
];
const coreGameplayDetailEntrySource = readFileSync(new URL("./src/modules/core/details/gameplay.js", import.meta.url), "utf8");
const coreGameplayDetailModulePaths = [
  "./src/modules/core/details/gameplay/effect.js",
  "./src/modules/core/details/gameplay/ability.js",
  "./src/modules/core/details/gameplay/tags.js"
];
const coreGameplayDetailSource = [
  coreGameplayDetailEntrySource,
  ...coreGameplayDetailModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreAiDetailEntrySource = readFileSync(new URL("./src/modules/core/details/ai.js", import.meta.url), "utf8");
const coreAiDetailModulePaths = [
  "./src/modules/core/details/ai/perception.js",
  "./src/modules/core/details/ai/behavior.js"
];
const coreAiDetailSource = [
  coreAiDetailEntrySource,
  ...coreAiDetailModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreRenderingDetailEntrySource = readFileSync(new URL("./src/modules/core/details/rendering.js", import.meta.url), "utf8");
const coreRenderingDetailModulePaths = [
  "./src/modules/core/details/rendering/material.js",
  "./src/modules/core/details/rendering/render-pipeline.js",
  "./src/modules/core/details/rendering/vfx.js"
];
const coreRenderingDetailSource = [
  coreRenderingDetailEntrySource,
  ...coreRenderingDetailModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreModuleDetailsEntrySource = readFileSync(new URL("./src/modules/core/core-module-details.js", import.meta.url), "utf8");
const coreModuleDetailsSource = [
  coreModuleDetailsEntrySource,
  ...coreDetailDomainPaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8")),
  coreGameplayDetailSource,
  coreAiDetailSource,
  coreRenderingDetailSource
].join("\n");
const coreLeftDomainPaths = [
  "./src/modules/core/lefts/index.js",
  "./src/modules/core/lefts/gameplay.js",
  "./src/modules/core/lefts/ai.js",
  "./src/modules/core/lefts/rendering.js",
  "./src/modules/core/lefts/assets.js",
  "./src/modules/core/lefts/ui.js"
];
const coreGameplayLeftEntrySource = readFileSync(new URL("./src/modules/core/lefts/gameplay.js", import.meta.url), "utf8");
const coreGameplayLeftModulePaths = [
  "./src/modules/core/lefts/gameplay/effect.js",
  "./src/modules/core/lefts/gameplay/ability.js",
  "./src/modules/core/lefts/gameplay/tags.js"
];
const coreGameplayLeftSource = [
  coreGameplayLeftEntrySource,
  ...coreGameplayLeftModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreAiLeftEntrySource = readFileSync(new URL("./src/modules/core/lefts/ai.js", import.meta.url), "utf8");
const coreAiLeftModulePaths = [
  "./src/modules/core/lefts/ai/perception.js",
  "./src/modules/core/lefts/ai/behavior.js"
];
const coreAiLeftSource = [
  coreAiLeftEntrySource,
  ...coreAiLeftModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreRenderingLeftEntrySource = readFileSync(new URL("./src/modules/core/lefts/rendering.js", import.meta.url), "utf8");
const coreRenderingLeftModulePaths = [
  "./src/modules/core/lefts/rendering/material.js",
  "./src/modules/core/lefts/rendering/render-pipeline.js",
  "./src/modules/core/lefts/rendering/vfx.js"
];
const coreRenderingLeftSource = [
  coreRenderingLeftEntrySource,
  ...coreRenderingLeftModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreModuleLeftsEntrySource = readFileSync(new URL("./src/modules/core/core-module-lefts.js", import.meta.url), "utf8");
const coreModuleLeftsSource = [
  coreModuleLeftsEntrySource,
  ...coreLeftDomainPaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8")),
  coreGameplayLeftSource,
  coreAiLeftSource,
  coreRenderingLeftSource
].join("\n");
const coreRegistryDomainPaths = [
  "./src/modules/core/registry/index.js",
  "./src/modules/core/registry/gameplay.js",
  "./src/modules/core/registry/ai.js",
  "./src/modules/core/registry/rendering.js",
  "./src/modules/core/registry/assets.js",
  "./src/modules/core/registry/ui.js"
];
const coreModulesEntrySource = readFileSync(new URL("./src/modules/core/core-modules.js", import.meta.url), "utf8");
const coreGameplayRegistryEntrySource = readFileSync(new URL("./src/modules/core/registry/gameplay.js", import.meta.url), "utf8");
const coreGameplayRegistryModulePaths = [
  "./src/modules/core/registry/gameplay/effect.js",
  "./src/modules/core/registry/gameplay/ability.js",
  "./src/modules/core/registry/gameplay/tags.js"
];
const coreGameplayRegistrySource = [
  coreGameplayRegistryEntrySource,
  ...coreGameplayRegistryModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreAiRegistryEntrySource = readFileSync(new URL("./src/modules/core/registry/ai.js", import.meta.url), "utf8");
const coreAiRegistryModulePaths = [
  "./src/modules/core/registry/ai/perception.js",
  "./src/modules/core/registry/ai/behavior.js"
];
const coreAiRegistrySource = [
  coreAiRegistryEntrySource,
  ...coreAiRegistryModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreRenderingRegistryEntrySource = readFileSync(new URL("./src/modules/core/registry/rendering.js", import.meta.url), "utf8");
const coreRenderingRegistryModulePaths = [
  "./src/modules/core/registry/rendering/material.js",
  "./src/modules/core/registry/rendering/render-pipeline.js",
  "./src/modules/core/registry/rendering/vfx.js"
];
const coreRenderingRegistrySource = [
  coreRenderingRegistryEntrySource,
  ...coreRenderingRegistryModulePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const coreModulesSource = [
  coreModulesEntrySource,
  ...coreRegistryDomainPaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8")),
  coreGameplayRegistrySource,
  coreAiRegistrySource,
  coreRenderingRegistrySource
].join("\n");
const extensionConfigRolePaths = [
  "./src/modules/extensions/configs/sources.js",
  "./src/modules/extensions/configs/factory.js",
  "./src/modules/extensions/configs/recipes.js",
  "./src/modules/extensions/configs/recipes/animation.js",
  "./src/modules/extensions/configs/recipes/data.js",
  "./src/modules/extensions/configs/recipes/default.js",
  "./src/modules/extensions/configs/recipes/diagnostics.js",
  "./src/modules/extensions/configs/recipes/gameplay.js",
  "./src/modules/extensions/configs/recipes/online.js",
  "./src/modules/extensions/configs/recipes/production.js",
  "./src/modules/extensions/configs/recipes/rendering.js",
  "./src/modules/extensions/configs/recipes/runtime.js",
  "./src/modules/extensions/configs/recipes/simulation.js",
  "./src/modules/extensions/configs/recipes/ui.js",
  "./src/modules/extensions/configs/recipes/vfx.js",
  "./src/modules/extensions/configs/recipes/world.js",
  "./src/modules/extensions/configs/layout-kind.js",
  "./src/modules/extensions/configs/controls.js",
  "./src/modules/extensions/configs/assets.js",
  "./src/modules/extensions/configs/text.js"
];
const extensionConfigsEntrySource = readFileSync(new URL("./src/modules/extensions/extension-configs.js", import.meta.url), "utf8");
const extensionRecipesEntrySource = readFileSync(new URL("./src/modules/extensions/configs/recipes.js", import.meta.url), "utf8");
const extensionConfigsSource = [
  extensionConfigsEntrySource,
  ...extensionConfigRolePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const extensionModulesSource = readFileSync(new URL("./src/modules/extensions/extension-modules.js", import.meta.url), "utf8");
const extensionSurfaceRolePaths = [
  "./src/modules/extensions/surfaces/left.js",
  "./src/modules/extensions/surfaces/center.js",
  "./src/modules/extensions/surfaces/details.js",
  "./src/modules/extensions/surfaces/bottom.js",
  "./src/modules/extensions/surfaces/primary.js",
  "./src/modules/extensions/surfaces/routes.js",
  "./src/modules/extensions/surfaces/utils.js"
];
const extensionSurfacesEntrySource = readFileSync(new URL("./src/modules/extensions/extension-surfaces.js", import.meta.url), "utf8");
const extensionPrimaryEntrySource = readFileSync(new URL("./src/modules/extensions/surfaces/primary.js", import.meta.url), "utf8");
const extensionPrimaryRolePaths = [
  "./src/modules/extensions/surfaces/primary/panel.js",
  "./src/modules/extensions/surfaces/primary/blueprint.js",
  "./src/modules/extensions/surfaces/primary/layout-kind.js",
  "./src/modules/extensions/surfaces/primary/graph.js"
];
const extensionPrimarySource = [
  extensionPrimaryEntrySource,
  ...extensionPrimaryRolePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const extensionBottomEntrySource = readFileSync(new URL("./src/modules/extensions/surfaces/bottom.js", import.meta.url), "utf8");
const extensionBottomRolePaths = [
  "./src/modules/extensions/surfaces/bottom/panel.js",
  "./src/modules/extensions/surfaces/bottom/output.js",
  "./src/modules/extensions/surfaces/bottom/validation.js",
  "./src/modules/extensions/surfaces/bottom/references.js",
  "./src/modules/extensions/surfaces/bottom/handoff.js"
];
const extensionBottomSource = [
  extensionBottomEntrySource,
  ...extensionBottomRolePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const extensionLeftEntrySource = readFileSync(new URL("./src/modules/extensions/surfaces/left.js", import.meta.url), "utf8");
const extensionLeftRolePaths = [
  "./src/modules/extensions/surfaces/left/panel.js",
  "./src/modules/extensions/surfaces/left/reference.js",
  "./src/modules/extensions/surfaces/left/tools.js",
  "./src/modules/extensions/surfaces/left/assets.js"
];
const extensionLeftSource = [
  extensionLeftEntrySource,
  ...extensionLeftRolePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const extensionCenterEntrySource = readFileSync(new URL("./src/modules/extensions/surfaces/center.js", import.meta.url), "utf8");
const extensionCenterRolePaths = [
  "./src/modules/extensions/surfaces/center/panel.js",
  "./src/modules/extensions/surfaces/center/metrics.js",
  "./src/modules/extensions/surfaces/center/reference-rhythm.js"
];
const extensionCenterSource = [
  extensionCenterEntrySource,
  ...extensionCenterRolePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const extensionDetailsEntrySource = readFileSync(new URL("./src/modules/extensions/surfaces/details.js", import.meta.url), "utf8");
const extensionDetailsRolePaths = [
  "./src/modules/extensions/surfaces/details/panel.js",
  "./src/modules/extensions/surfaces/details/summary.js",
  "./src/modules/extensions/surfaces/details/table.js",
  "./src/modules/extensions/surfaces/details/status.js"
];
const extensionDetailsSource = [
  extensionDetailsEntrySource,
  ...extensionDetailsRolePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const extensionSurfacesSource = [
  extensionSurfacesEntrySource,
  ...extensionSurfaceRolePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8")),
  extensionPrimarySource,
  extensionBottomSource,
  extensionLeftSource,
  extensionCenterSource,
  extensionDetailsSource
].join("\n");
const extensionLibraryRolePaths = [
  "./src/modules/extensions/library/module.js",
  "./src/modules/extensions/library/left.js",
  "./src/modules/extensions/library/center.js",
  "./src/modules/extensions/library/cards.js",
  "./src/modules/extensions/library/drilldown.js",
  "./src/modules/extensions/library/details.js",
  "./src/modules/extensions/library/bottom.js",
  "./src/modules/extensions/library/rows.js",
  "./src/modules/extensions/library/routes.js"
];
const extensionLibraryEntrySource = readFileSync(new URL("./src/modules/extensions/extension-library.js", import.meta.url), "utf8");
const extensionLibrarySource = [
  extensionLibraryEntrySource,
  ...extensionLibraryRolePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const extensionHandoffSource = readFileSync(new URL("./src/modules/extensions/extension-handoff.js", import.meta.url), "utf8");
const componentLabCenterEntrySource = readFileSync(new URL("./src/modules/component-lab/center.js", import.meta.url), "utf8");
const componentLabRolePaths = [
  "./src/modules/component-lab/module.js",
  "./src/modules/component-lab/data.js",
  "./src/modules/component-lab/routes.js",
  "./src/modules/component-lab/left.js",
  "./src/modules/component-lab/center.js",
  "./src/modules/component-lab/center/atom-palette.js",
  "./src/modules/component-lab/center/collection-palette.js",
  "./src/modules/component-lab/center/coverage-matrix.js",
  "./src/modules/component-lab/center/lab-column.js",
  "./src/modules/component-lab/center/layout-grammar.js",
  "./src/modules/component-lab/center/surface-palette.js",
  "./src/modules/component-lab/details.js",
  "./src/modules/component-lab/bottom.js"
];
const componentLabSource = componentLabRolePaths
  .map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
  .join("\n");
const extensionBlueprintDomainPaths = [
  "./src/modules/extensions/blueprints/animation.js",
  "./src/modules/extensions/blueprints/data.js",
  "./src/modules/extensions/blueprints/diagnostics.js",
  "./src/modules/extensions/blueprints/gameplay.js",
  "./src/modules/extensions/blueprints/multiplayer.js",
  "./src/modules/extensions/blueprints/production.js",
  "./src/modules/extensions/blueprints/rendering.js",
  "./src/modules/extensions/blueprints/simulation.js",
  "./src/modules/extensions/blueprints/ui.js",
  "./src/modules/extensions/blueprints/world.js"
];
const extensionBlueprintAnimationRolePaths = [
  "./animation/sequencer.js",
  "./animation/montage-editor.js",
  "./animation/animation-compression.js",
  "./animation/blend-space.js",
  "./animation/control-rig.js",
  "./animation/motion-matching.js",
  "./animation/pose-library.js",
  "./animation/retarget.js"
];
const extensionBlueprintWorldRolePaths = [
  "./world/terrain-editor.js",
  "./world/foliage-editor.js",
  "./world/level-streaming.js",
  "./world/level-variant.js",
  "./world/prefab-editor.js",
  "./world/scatter-editor.js",
  "./world/volume-editor.js",
  "./world/weather-editor.js"
];
const extensionBlueprintAnimationEntrySource = readFileSync(new URL("./src/modules/extensions/blueprints/animation.js", import.meta.url), "utf8");
const extensionBlueprintWorldEntrySource = readFileSync(new URL("./src/modules/extensions/blueprints/world.js", import.meta.url), "utf8");
const extensionBlueprintAnimationSource = extensionBlueprintAnimationRolePaths
  .map((path) => readFileSync(new URL(`./src/modules/extensions/blueprints/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const extensionBlueprintWorldSource = extensionBlueprintWorldRolePaths
  .map((path) => readFileSync(new URL(`./src/modules/extensions/blueprints/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const extensionBlueprintEntrySource = readFileSync(new URL("./src/modules/extensions/extension-blueprints.js", import.meta.url), "utf8");
const extensionBlueprintsSource = [
  extensionBlueprintEntrySource,
  readFileSync(new URL("./src/modules/extensions/blueprints/helpers.js", import.meta.url), "utf8"),
  ...extensionBlueprintDomainPaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8")),
  extensionBlueprintAnimationSource,
  extensionBlueprintWorldSource
].join("\n");
const moduleComponentsEntrySource = readFileSync(new URL("./src/modules/shared/module-components.js", import.meta.url), "utf8");
const moduleComponentsSource = [
  "./src/modules/shared/module-components.js",
  "./src/modules/shared/actions.js",
  "./src/modules/shared/bottom-output.js",
  "./src/modules/shared/panels.js",
  "./src/modules/shared/regions.js",
  "./src/modules/shared/rows.js",
  "./src/modules/shared/utils.js",
  "./src/modules/shared/visuals.js",
].map((path) => readFileSync(new URL(path, import.meta.url), "utf8")).join("\n");
const collectionsSource = readFileSync(new URL("./src/components/data/collections.js", import.meta.url), "utf8");
const collectionDataSource = [
  collectionsSource,
  readFileSync(new URL("./src/components/data/list-view.js", import.meta.url), "utf8"),
  readFileSync(new URL("./src/components/data/list-view/row.js", import.meta.url), "utf8"),
  readFileSync(new URL("./src/components/data/table-view.js", import.meta.url), "utf8"),
  readFileSync(new URL("./src/components/data/table-view/header.js", import.meta.url), "utf8"),
  readFileSync(new URL("./src/components/data/table-view/row.js", import.meta.url), "utf8"),
  readFileSync(new URL("./src/components/data/tree-view.js", import.meta.url), "utf8"),
  readFileSync(new URL("./src/components/data/tree-view/row.js", import.meta.url), "utf8"),
].join("\n");
const collectionCssEntrySource = readFileSync(new URL("./src/components/data/collections.css", import.meta.url), "utf8");
const collectionCssRolePaths = [
  "./collections/panel-group.css",
  "./collections/tree-view.css",
  "./collections/table-view.css",
  "./collections/list-view.css"
];
const collectionCssSource = collectionCssRolePaths
  .map((path) => readFileSync(new URL(`./src/components/data/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const actionPathsSource = readFileSync(new URL("./src/foundation/action-paths.js", import.meta.url), "utf8");
const modulesCssSource = readFileSync(new URL("./src/modules/modules.css", import.meta.url), "utf8");
const moduleShellCssRolePaths = [
  "./shell/top-tabs.css",
  "./shell/toolbar.css",
  "./shell/regions.css",
  "./shell/mainbar.css",
  "./shell/panel-tabs.css",
  "./shell/cards.css",
  "./shell/forms.css"
];
const moduleShellCssSource = moduleShellCssRolePaths
  .map((path) => readFileSync(new URL(`./src/modules/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const moduleOutputCssEntrySource = readFileSync(new URL("./src/modules/module-output.css", import.meta.url), "utf8");
const moduleOutputCssRolePaths = [
  "./output/preview.css",
  "./output/stats-actions.css",
  "./output/layout.css",
  "./output/logs.css",
  "./output/asset-strip.css",
  "./output/timeline.css"
];
const moduleOutputCssSource = moduleOutputCssRolePaths
  .map((path) => readFileSync(new URL(`./src/modules/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const moduleDataCssEntrySource = readFileSync(new URL("./src/modules/module-data.css", import.meta.url), "utf8");
const moduleDataCssRolePaths = [
  "./data/settings.css",
  "./data/collection-rows.css",
  "./data/list-rows.css",
  "./data/tree-rows.css",
  "./data/table-rows.css",
  "./data/tags.css",
  "./data/card-tools.css"
];
const moduleDataCssSource = moduleDataCssRolePaths
  .map((path) => readFileSync(new URL(`./src/modules/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const moduleCanvasesCssEntrySource = readFileSync(new URL("./src/modules/module-canvases.css", import.meta.url), "utf8");
const moduleMapCanvasCssEntrySource = readFileSync(new URL("./src/modules/canvases/map.css", import.meta.url), "utf8");
const moduleMapCanvasCssRolePaths = [
  "./map/base.css",
  "./map/walls.css",
  "./map/points.css",
  "./map/cones.css",
  "./map/paths.css"
];
const moduleMapCanvasCssSource = moduleMapCanvasCssRolePaths
  .map((path) => readFileSync(new URL(`./src/modules/canvases/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const moduleHudCanvasCssEntrySource = readFileSync(new URL("./src/modules/canvases/hud.css", import.meta.url), "utf8");
const moduleHudCanvasCssRolePaths = [
  "./hud/base.css",
  "./hud/widgets.css",
  "./hud/positions.css",
  "./hud/status.css",
  "./hud/crosshair.css"
];
const moduleHudCanvasCssSource = moduleHudCanvasCssRolePaths
  .map((path) => readFileSync(new URL(`./src/modules/canvases/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const moduleFeedbackCssEntrySource = readFileSync(new URL("./src/modules/module-feedback.css", import.meta.url), "utf8");
const workbenchCssEntrySource = readFileSync(new URL("./src/workbench/workbench.css", import.meta.url), "utf8");
const workbenchLowerDemoCssRolePaths = [
  "./lower-demo/layout.css",
  "./lower-demo/alerts.css",
  "./lower-demo/table.css",
  "./lower-demo/toast.css",
  "./lower-demo/effects.css",
  "./lower-demo/tooltip.css"
];
const workbenchLowerDemoCssSource = workbenchLowerDemoCssRolePaths
  .map((path) => readFileSync(new URL(`./src/workbench/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const sidePanelsCssEntrySource = readFileSync(new URL("./src/workbench/side-panels.css", import.meta.url), "utf8");
const sidePanelsCssRolePaths = [
  "./side-panels/menus.css",
  "./side-panels/alt-panel.css",
  "./side-panels/layer-history.css",
  "./side-panels/console.css",
  "./side-panels/inspector-checkboxes.css",
  "./side-panels/form-overrides.css",
  "./side-panels/topbar-overrides.css"
];
const sidePanelsCssSource = sidePanelsCssRolePaths
  .map((path) => readFileSync(new URL(`./src/workbench/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const statusbarTuningCssEntrySource = readFileSync(new URL("./src/workbench/statusbar-tuning.css", import.meta.url), "utf8");
const statusbarTuningCssRolePaths = [
  "./statusbar-tuning/popup-layer.css",
  "./statusbar-tuning/left-group.css",
  "./statusbar-tuning/right-group.css",
  "./statusbar-tuning/frame.css",
  "./statusbar-tuning/controls.css"
];
const statusbarTuningCssSource = statusbarTuningCssRolePaths
  .map((path) => readFileSync(new URL(`./src/workbench/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const showcaseControlCssEntrySource = readFileSync(new URL("./src/workbench/showcase-controls.css", import.meta.url), "utf8");
const showcaseControlCssRolePaths = [
  "./showcase-controls/icon-buttons.css",
  "./showcase-controls/shared-gaps.css",
  "./showcase-controls/button-grid.css",
  "./showcase-controls/fields.css",
  "./showcase-controls/selection-controls.css",
  "./showcase-controls/segmented-controls.css",
  "./showcase-controls/sliders.css",
  "./showcase-controls/tabs.css"
];
const showcaseButtonGridCssEntrySource = readFileSync(new URL("./src/workbench/showcase-controls/button-grid.css", import.meta.url), "utf8");
const showcaseButtonGridCssRolePaths = [
  "./button-grid/layout.css",
  "./button-grid/base-controls.css",
  "./button-grid/state-colors.css",
  "./button-grid/item-overrides.css"
];
const showcaseButtonGridCssSource = showcaseButtonGridCssRolePaths
  .map((path) => readFileSync(new URL(`./src/workbench/showcase-controls/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const showcaseControlCssSource = showcaseControlCssRolePaths
  .map((path) => readFileSync(new URL(`./src/workbench/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .concat(showcaseButtonGridCssSource)
  .join("\n");
const inspectorDetailCssEntrySource = readFileSync(new URL("./src/workbench/inspector-detail.css", import.meta.url), "utf8");
const inspectorDetailCssRolePaths = [
  "./inspector-detail/base.css",
  "./inspector-detail/scene-tree.css",
  "./inspector-detail/forms.css",
  "./inspector-detail/transform-section.css",
  "./inspector-detail/mesh-renderer-section.css"
];
const inspectorDetailCssSource = inspectorDetailCssRolePaths
  .map((path) => readFileSync(new URL(`./src/workbench/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const inspectorTransformCssEntrySource = readFileSync(new URL("./src/workbench/inspector-detail/transform-section.css", import.meta.url), "utf8");
const inspectorTransformCssRolePaths = [
  "./transform-section/section.css",
  "./transform-section/value-boxes.css",
  "./transform-section/vector-rows.css",
  "./transform-section/linked-axis.css",
  "./transform-section/axis-labels.css",
  "./transform-section/controls.css"
];
const inspectorTransformCssSource = inspectorTransformCssRolePaths
  .map((path) => readFileSync(new URL(`./src/workbench/inspector-detail/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const moduleLayoutsCssEntrySource = readFileSync(new URL("./src/modules/module-layouts.css", import.meta.url), "utf8");
const moduleLayoutsCssRolePaths = [
  "./layouts/base.css",
  "./layouts/core.css",
  "./layouts/library.css",
  "./layouts/extensions.css"
];
const moduleLayoutsCssSource = moduleLayoutsCssRolePaths
  .map((path) => readFileSync(new URL(`./src/modules/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const extensionLibraryCssEntrySource = readFileSync(new URL("./src/modules/extension-library.css", import.meta.url), "utf8");
const extensionLibraryCssRolePaths = [
  "./extension-library/card-grid.css",
  "./extension-library/cards.css",
  "./extension-library/drilldown.css",
  "./extension-library/panel-group.css"
];
const extensionLibraryCssSource = extensionLibraryCssRolePaths
  .map((path) => readFileSync(new URL(`./src/modules/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const moduleGraphsCssEntrySource = readFileSync(new URL("./src/modules/module-graphs.css", import.meta.url), "utf8");
const moduleGraphsCssRolePaths = [
  "./graphs/board.css",
  "./graphs/nodes.css",
  "./graphs/links.css",
  "./graphs/minimap.css",
  "./graphs/curves.css"
];
const moduleGraphsCssSource = moduleGraphsCssRolePaths
  .map((path) => readFileSync(new URL(`./src/modules/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const moduleResponsiveCssEntrySource = readFileSync(new URL("./src/modules/module-responsive.css", import.meta.url), "utf8");
const moduleResponsiveCssRolePaths = [
  "./responsive/navigation.css",
  "./responsive/workspace.css",
  "./responsive/tablet-shell.css",
  "./responsive/mobile-stack.css"
];
const moduleResponsiveCssSource = moduleResponsiveCssRolePaths
  .map((path) => readFileSync(new URL(`./src/modules/${path.replace("./", "")}`, import.meta.url), "utf8"))
  .join("\n");
const moduleRoleCssSource = [
  moduleMapCanvasCssEntrySource,
  moduleMapCanvasCssSource,
  moduleHudCanvasCssEntrySource,
  moduleHudCanvasCssSource,
  readFileSync(new URL("./src/modules/feedback/inline-status.css", import.meta.url), "utf8")
].join("\n");
const moduleCssCombinedSource = [
  modulesCssSource,
  moduleShellCssSource,
  moduleLayoutsCssEntrySource,
  moduleLayoutsCssSource,
  extensionLibraryCssEntrySource,
  extensionLibraryCssSource,
  moduleDataCssEntrySource,
  moduleDataCssSource,
  moduleGraphsCssEntrySource,
  moduleGraphsCssSource,
  moduleOutputCssEntrySource,
  moduleOutputCssSource,
  moduleCanvasesCssEntrySource,
  moduleFeedbackCssEntrySource,
  moduleRoleCssSource,
  moduleResponsiveCssEntrySource,
  moduleResponsiveCssSource,
].join("\n");
const routingRolePaths = [
  "./src/routing/commands/module-targets.js",
  "./src/routing/commands/scoped-targets.js",
  "./src/routing/commands/panel-targets.js",
  "./src/routing/commands/extension-targets.js",
  "./src/routing/commands/labels.js",
  "./src/routing/commands/route-for-command.js",
  "./src/routing/panels/activation.js"
];
const routesEntrySource = readFileSync(new URL("./src/routing/routes.js", import.meta.url), "utf8");
const routesSource = [
  routesEntrySource,
  ...routingRolePaths.map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
].join("\n");
const responsiveSource = readFileSync(new URL("./validate-responsive.mjs", import.meta.url), "utf8");
const controlRouteSource = readFileSync(new URL("./validate-control-routes.mjs", import.meta.url), "utf8");
const referencePngSources = readdirSync(new URL("../", import.meta.url))
  .filter((name) => name.toLowerCase().endsWith(".png"))
  .sort();
const rootSourceEntries = new Set(readdirSync(new URL("./", import.meta.url)));
const flatComponentSourceEntries = [
  "action-paths.js",
  "atoms.css",
  "atoms.js",
  "collections.css",
  "collections.js",
  "core-module-bottoms.js",
  "core-module-centers.js",
  "core-module-details.js",
  "core-module-lefts.js",
  "core-modules.js",
  "data.js",
  "extension-blueprints.js",
  "extension-configs.js",
  "extension-handoff.js",
  "extension-library.js",
  "extension-modules.js",
  "extension-surfaces.js",
  "icons.js",
  "layout.css",
  "layout.js",
  "module-components.js",
  "modules.css",
  "modules.js",
  "reference-samples.js",
  "responsive.css",
  "routes.js",
  "surfaces.css",
  "surfaces.js",
  "tokens.css",
  "workbench.css",
].filter((entry) => rootSourceEntries.has(entry));
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
const renderedActionIds = Array.from(
  [
    appHtml,
    surfacePanelHtml,
    libraryHtml,
    coreAndLibraryWorkspaceHtml,
    extensionWorkspaceHtml
  ].join("\n").matchAll(/data-action="([^"]+)"/g),
  (match) => match[1]
);
const invalidRenderedActionIds = renderedActionIds.filter((id) => !/^[a-z0-9_]+(?:\.[a-z0-9_]+)+$/.test(id));

const checks = [
  ["tokens layer loaded first", indexSource.indexOf("tokens.css") < indexSource.indexOf("layout.css")],
  ["foundation tokens split by token role", foundationTokenCssRolePaths.every((path) => foundationTokenCssEntrySource.includes(path))
    && !foundationTokenCssEntrySource.includes(":root")
    && !foundationTokenCssEntrySource.includes(".zr-")
    && foundationTokenCssSource.includes("--ref-w")
    && foundationTokenCssSource.includes("--font")
    && foundationTokenCssSource.includes("--accent")
    && foundationTokenCssSource.includes("--shadow")
    && foundationTokenCssSource.includes("--control-h")
    && foundationTokenCssSource.includes("--gap-4")
    && foundationTokenCssSource.includes(".zr-app")
    && foundationTokenCssSource.includes(".sr-only")],
  ["layout layer before atoms", indexSource.indexOf("layout.css") < indexSource.indexOf("atoms.css")],
  ["atoms layer before collections", indexSource.indexOf("atoms.css") < indexSource.indexOf("collections.css")],
  ["collections layer before surfaces", indexSource.indexOf("collections.css") < indexSource.indexOf("surfaces.css")],
  ["surfaces layer before modules", indexSource.indexOf("surfaces.css") < indexSource.indexOf("modules.css")],
  ["modules layer before workbench", indexSource.indexOf("modules.css") < indexSource.indexOf("workbench.css")],
  ["web component sources live in functional src folders", flatComponentSourceEntries.length === 0 && indexSource.includes("src/components/inputs/atoms.css") && indexSource.includes("src/components/data/collections.css") && indexSource.includes("src/modules/modules.css")],
  ["input atom implementations split by component role", [
    "./buttons.js",
    "./fields.js",
    "./selection-controls.js",
    "./tabs.js",
    "./dropdowns.js",
    "./sliders.js"
  ].every((path) => inputAggregatorSource.includes(path))
    && ["export function button", "export function iconButton", "export function input", "export function searchInput", "export function checkbox", "export function radio", "export function toggle", "export function tabs", "export function select", "export function slider", "export function rangeSlider"].every((needle) => inputAtomsSource.includes(needle))
    && !inputAggregatorSource.includes("export function button")
    && !inputAggregatorSource.includes("export function input")],
  ["button atom implementations split by button role", inputAtomsSource.includes("./buttons/button.js") && inputAtomsSource.includes("./buttons/icon-button.js") && !readFileSync(new URL("./src/components/inputs/buttons.js", import.meta.url), "utf8").includes("export function button") && !readFileSync(new URL("./src/components/inputs/buttons.js", import.meta.url), "utf8").includes("export function iconButton")],
  ["field atom implementations split by field role", inputAtomsSource.includes("./fields/input.js") && inputAtomsSource.includes("./fields/search-input.js") && inputAtomsSource.includes("./fields/number-field.js") && !readFileSync(new URL("./src/components/inputs/fields.js", import.meta.url), "utf8").includes("export function input") && !readFileSync(new URL("./src/components/inputs/fields.js", import.meta.url), "utf8").includes("export function searchInput") && !readFileSync(new URL("./src/components/inputs/fields.js", import.meta.url), "utf8").includes("export function numberField")],
  ["selection atom implementations split by control role", inputAtomsSource.includes("./selection-controls/checkbox.js") && inputAtomsSource.includes("./selection-controls/radio.js") && inputAtomsSource.includes("./selection-controls/toggle.js") && !readFileSync(new URL("./src/components/inputs/selection-controls.js", import.meta.url), "utf8").includes("export function checkbox") && !readFileSync(new URL("./src/components/inputs/selection-controls.js", import.meta.url), "utf8").includes("export function radio") && !readFileSync(new URL("./src/components/inputs/selection-controls.js", import.meta.url), "utf8").includes("export function toggle")],
  ["dropdown atom implementations split by select role", inputAtomsSource.includes("./dropdowns/select.js") && !readFileSync(new URL("./src/components/inputs/dropdowns.js", import.meta.url), "utf8").includes("export function select") && inputAtomsSource.includes("export function select")],
  ["slider atom implementations split by slider role", inputAtomsSource.includes("./sliders/slider.js") && inputAtomsSource.includes("./sliders/range-slider.js") && !readFileSync(new URL("./src/components/inputs/sliders.js", import.meta.url), "utf8").includes("export function slider") && !readFileSync(new URL("./src/components/inputs/sliders.js", import.meta.url), "utf8").includes("export function rangeSlider")],
  ["input atom styles split by component role", ["src/components/inputs/buttons.css", "src/components/inputs/fields.css", "src/components/inputs/selection-controls.css", "src/components/inputs/tabs.css", "src/components/inputs/dropdowns.css", "src/components/inputs/sliders.css"].every((path) => indexSource.includes(path))],
  ["surface implementations split by component role", [
    "./shell/window.js",
    "./shell/chrome.js",
    "./panels/scene-panel.js",
    "./panels/inspector-panel.js",
    "./panels/showcase-panel.js",
    "./viewport/viewport-surface.js",
    "../overlays/popup-layer.js"
  ].every((path) => surfaceAggregatorSource.includes(path))
    && surfacesSource.includes("export function drawerSurface")
    && !surfaceAggregatorSource.includes("export function workbenchWindow(children)")
    && !surfaceAggregatorSource.includes("function drawerSurface")],
  ["collection feedback and overlay styles split by component domain", ["src/components/data/collections.css", "src/components/overlays/menu.css", "src/components/feedback/alerts.css", "src/components/feedback/toast.css", "src/components/feedback/tooltip.css"].every((path) => indexSource.includes(path))],
  ["collection data styles split by collection role", collectionCssRolePaths.every((path) => collectionCssEntrySource.includes(path))
    && !collectionCssEntrySource.includes(".zr-")
    && collectionCssSource.includes(".zr-panel-tabs")
    && collectionCssSource.includes(".zr-tree-row")
    && collectionCssSource.includes(".zr-table-row")
    && collectionCssSource.includes(".zr-list-item")],
  ["list collection implementations split by row role", collectionDataSource.includes("./list-view/row.js") && !readFileSync(new URL("./src/components/data/list-view.js", import.meta.url), "utf8").includes('class="zr-list-item') && collectionDataSource.includes("export function listRow") && collectionDataSource.includes("export function listView")],
  ["table collection implementations split by header and row role", collectionDataSource.includes("./table-view/header.js") && collectionDataSource.includes("./table-view/row.js") && !readFileSync(new URL("./src/components/data/table-view.js", import.meta.url), "utf8").includes("zr-table-head") && !readFileSync(new URL("./src/components/data/table-view.js", import.meta.url), "utf8").includes('class="zr-table-row ${index') && collectionDataSource.includes("export function tableHeader") && collectionDataSource.includes("export function tableRow") && collectionDataSource.includes("export function tableView")],
  ["tree collection implementations split by row role", collectionDataSource.includes("./tree-view/row.js") && !readFileSync(new URL("./src/components/data/tree-view.js", import.meta.url), "utf8").includes('<button class="zr-tree-row') && !readFileSync(new URL("./src/components/data/tree-view.js", import.meta.url), "utf8").includes("function treeRow") && collectionDataSource.includes("export function treeRow") && collectionDataSource.includes("export function treeView")],
  ["popup menu implementations split by row role", overlayMenuSource.includes("./menu/row.js") && !overlayMenuSource.includes('class="zr-menu-row') && overlaySource.includes("export function menuRow") && overlaySource.includes("export function menu") && overlaySource.includes('actionPath("workbench.collection.menu", label)')],
  ["surface shell and panel styles split by role", surfaceShellPanelCssRolePaths.every((path) => surfaceShellPanelCssEntrySource.includes(path))
    && !surfaceShellPanelCssEntrySource.includes(".zr-")
    && surfaceShellPanelCssSource.includes(".zr-window")
    && surfaceShellPanelCssSource.includes(".zr-topbar")
    && surfaceShellPanelCssSource.includes(".zr-rail")
    && surfaceShellPanelCssSource.includes(".zr-panel")
    && surfaceShellPanelCssSource.includes(".zr-scene-panel")
    && surfaceShellPanelCssSource.includes(".zr-panel-toolbar")],
  ["surface inspector styles split by inspector role", inspectorSurfaceCssRolePaths.every((path) => inspectorSurfaceCssEntrySource.includes(path))
    && !inspectorSurfaceCssEntrySource.includes(".zr-")
    && inspectorSurfaceCssSource.includes(".zr-inspector")
    && inspectorSurfaceCssSource.includes(".zr-object-header")
    && inspectorSurfaceCssSource.includes(".zr-section-title")
    && inspectorSurfaceCssSource.includes(".zr-form-row")
    && inspectorSurfaceCssSource.includes(".zr-resource-row")],
  ["surface showcase styles split by showcase role", showcaseSurfaceCssRolePaths.every((path) => showcaseSurfaceCssEntrySource.includes(path))
    && !showcaseSurfaceCssEntrySource.includes(".zr-")
    && showcaseSurfaceCssSource.includes(".zr-showcase")
    && showcaseSurfaceCssSource.includes(".zr-showcase-grid")
    && showcaseSurfaceCssSource.includes(".zr-showcase-col")
    && showcaseSurfaceCssSource.includes(".zr-side-stack")],
  ["surface status styles split by statusbar role", statusSurfaceCssRolePaths.every((path) => statusSurfaceCssEntrySource.includes(path))
    && !statusSurfaceCssEntrySource.includes(".zr-")
    && statusSurfaceCssSource.includes(".zr-statusbar")
    && statusSurfaceCssSource.includes(".zr-status-left")
    && statusSurfaceCssSource.includes(".zr-statusbar .zr-select")
    && statusSurfaceCssSource.includes(".zr-dot")],
  ["foundation responsive styles split by responsive role", foundationResponsiveCssRolePaths.every((path) => foundationResponsiveCssEntrySource.includes(path))
    && !foundationResponsiveCssEntrySource.includes(".zr-")
    && foundationResponsiveCssSource.includes(".zr-topbar")
    && foundationResponsiveCssSource.includes(".zr-inspector")
    && foundationResponsiveCssSource.includes(".zr-viewport")
    && foundationResponsiveCssSource.includes(".zr-viewport-cluster:first-child .zr-select")],
  ["large css layers split by component domain", ["src/components/surfaces/viewport.css", "src/components/surfaces/inspector.css", "src/components/surfaces/showcase.css", "src/components/surfaces/status.css", "src/modules/module-layouts.css", "src/modules/extension-library.css", "src/modules/module-data.css", "src/modules/module-graphs.css", "src/modules/module-output.css", "src/modules/module-canvases.css", "src/modules/module-feedback.css", "src/modules/module-responsive.css", "src/workbench/showcase-base.css", "src/workbench/inspector-detail.css", "src/workbench/showcase-controls.css", "src/workbench/side-panels.css", "src/workbench/statusbar-tuning.css"].every((path) => indexSource.includes(path))],
  ["module shell styles split by shell role", moduleShellCssRolePaths.every((path) => modulesCssSource.includes(path))
    && !modulesCssSource.includes(".zr-")
    && moduleShellCssSource.includes(".zr-module-tabs")
    && moduleShellCssSource.includes(".zr-module-toolbar")
    && moduleShellCssSource.includes(".zr-module-left")
    && moduleShellCssSource.includes(".zr-module-mainbar")
    && moduleShellCssSource.includes(".zr-module-panel-tabs")
    && moduleShellCssSource.includes(".zr-module-card")
    && moduleShellCssSource.includes(".zr-module-filterbar .zr-search")],
  ["module layout styles split by layout role", moduleLayoutsCssRolePaths.every((path) => moduleLayoutsCssEntrySource.includes(path))
    && !moduleLayoutsCssEntrySource.includes(".zr-")
    && moduleLayoutsCssSource.includes(".zr-module-editor-grid")
    && moduleLayoutsCssSource.includes(".zr-module-editor-grid.is-gameplay")
    && moduleLayoutsCssSource.includes(".zr-module-editor-grid.is-library")
    && moduleLayoutsCssSource.includes(".zr-module-editor-grid.is-extension")],
  ["extension library styles split by library role", extensionLibraryCssRolePaths.every((path) => extensionLibraryCssEntrySource.includes(path))
    && !extensionLibraryCssEntrySource.includes(".zr-")
    && extensionLibraryCssSource.includes(".zr-extension-card-grid")
    && extensionLibraryCssSource.includes(".zr-extension-card")
    && extensionLibraryCssSource.includes(".zr-library-drilldown")
    && extensionLibraryCssSource.includes(".zr-panel-group")],
  ["module graph styles split by graph role", moduleGraphsCssRolePaths.every((path) => moduleGraphsCssEntrySource.includes(path))
    && !moduleGraphsCssEntrySource.includes(".zr-")
    && moduleGraphsCssSource.includes(".zr-module-graph")
    && moduleGraphsCssSource.includes(".zr-module-node")
    && moduleGraphsCssSource.includes(".zr-graph-link")
    && moduleGraphsCssSource.includes(".zr-module-minimap")
    && moduleGraphsCssSource.includes(".zr-module-curve")],
  ["module responsive styles split by responsive role", moduleResponsiveCssRolePaths.every((path) => moduleResponsiveCssEntrySource.includes(path))
    && !moduleResponsiveCssEntrySource.includes(".zr-")
    && moduleResponsiveCssSource.includes(".zr-module-tab")
    && moduleResponsiveCssSource.includes(".zr-module-editor-grid.is-gameplay")
    && moduleResponsiveCssSource.includes(".zr-module-left")
    && moduleResponsiveCssSource.includes("@media (max-width: 720px)")],
  ["module output styles split by output role", moduleOutputCssRolePaths.every((path) => moduleOutputCssEntrySource.includes(path))
    && !moduleOutputCssEntrySource.includes(".zr-")
    && moduleOutputCssSource.includes(".zr-module-preview")
    && moduleOutputCssSource.includes(".zr-module-stat-grid")
    && moduleOutputCssSource.includes(".zr-module-output-grid")
    && moduleOutputCssSource.includes(".zr-module-log")
    && moduleOutputCssSource.includes(".zr-module-asset-strip")
    && moduleOutputCssSource.includes(".zr-module-timeline")],
  ["module data styles split by data role", moduleDataCssRolePaths.every((path) => moduleDataCssEntrySource.includes(path))
    && !moduleDataCssEntrySource.includes(".zr-")
    && moduleDataCssSource.includes(".zr-module-setting")
    && moduleDataCssSource.includes(".zr-module-list-row")
    && moduleDataCssSource.includes(".zr-module-tree-row")
    && moduleDataCssSource.includes(".zr-module-table-row")
    && moduleDataCssSource.includes(".zr-module-tag")
    && moduleDataCssSource.includes(".zr-module-card-tools")],
  ["module canvas and feedback styles split by role", moduleCanvasesCssEntrySource.includes("./canvases/map.css")
    && moduleCanvasesCssEntrySource.includes("./canvases/hud.css")
    && !moduleCanvasesCssEntrySource.includes(".zr-")
    && moduleFeedbackCssEntrySource.includes("./feedback/inline-status.css")
    && !moduleFeedbackCssEntrySource.includes(".zr-")
    && moduleRoleCssSource.includes(".zr-module-map")
    && moduleRoleCssSource.includes(".zr-module-hud-canvas")
    && moduleRoleCssSource.includes(".zr-module-progress")
    && moduleRoleCssSource.includes(".zr-action-flash")],
  ["module map canvas styles split by map role", moduleMapCanvasCssRolePaths.every((path) => moduleMapCanvasCssEntrySource.includes(path))
    && !moduleMapCanvasCssEntrySource.includes(".zr-")
    && moduleMapCanvasCssSource.includes(".zr-module-map")
    && moduleMapCanvasCssSource.includes(".zr-map-wall")
    && moduleMapCanvasCssSource.includes(".zr-map-point")
    && moduleMapCanvasCssSource.includes(".zr-map-cone")
    && moduleMapCanvasCssSource.includes(".zr-map-path")],
  ["module hud canvas styles split by hud role", moduleHudCanvasCssRolePaths.every((path) => moduleHudCanvasCssEntrySource.includes(path))
    && !moduleHudCanvasCssEntrySource.includes(".zr-")
    && moduleHudCanvasCssSource.includes(".zr-module-hud-canvas")
    && moduleHudCanvasCssSource.includes(".zr-hud-widget")
    && moduleHudCanvasCssSource.includes(".zr-hud-widget.is-status")
    && moduleHudCanvasCssSource.includes(".zr-hud-crosshair")],
  ["viewport surface styles split by scene role", [
    "./viewport/base.css",
    "./viewport/lighting.css",
    "./viewport/structure.css",
    "./viewport/floor.css",
    "./viewport/props.css",
    "./viewport/tools.css"
  ].every((path) => viewportCssEntrySource.includes(path))
    && !viewportCssEntrySource.includes(".zr-")
    && viewportCssCombinedSource.includes(".zr-scene-shell")
    && viewportCssCombinedSource.includes(".zr-scene-lightwash")
    && viewportCssCombinedSource.includes(".zr-scene-wall")
    && viewportCssCombinedSource.includes(".zr-scene-floor")
    && viewportCssCombinedSource.includes(".zr-scene-cargo")
    && viewportCssCombinedSource.includes(".zr-viewport-tools")],
  ["viewport lighting styles split by lighting role", viewportLightingCssRolePaths.every((path) => viewportLightingCssEntrySource.includes(path))
    && !viewportLightingCssEntrySource.includes(".zr-")
    && viewportLightingCssSource.includes(".zr-scene-lightwash")
    && viewportLightingCssSource.includes(".zr-scene-shadow")],
  ["viewport floor styles split by floor role", viewportFloorCssRolePaths.every((path) => viewportFloorCssEntrySource.includes(path))
    && !viewportFloorCssEntrySource.includes(".zr-")
    && viewportFloorCssSource.includes(".zr-scene-floor")
    && viewportFloorCssSource.includes(".zr-viewport-grid-line")
    && viewportFloorCssSource.includes(".zr-floor-reflection")
    && viewportFloorCssSource.includes(".zr-floor-grate")
    && viewportFloorCssSource.includes(".zr-floor-panel")
    && viewportFloorCssSource.includes(".zr-floor-seam")],
  ["viewport structure styles split by structure role", viewportStructureCssRolePaths.every((path) => viewportStructureCssEntrySource.includes(path))
    && !viewportStructureCssEntrySource.includes(".zr-")
    && viewportStructureCssSource.includes(".zr-scene-wall")
    && viewportStructureCssSource.includes(".zr-scene-ceiling")
    && viewportStructureCssSource.includes(".zr-scene-door")
    && viewportStructureCssSource.includes(".zr-scene-wall-panel")
    && viewportStructureCssSource.includes(".zr-scene-side")
    && viewportStructureCssSource.includes(".zr-scene-handrail")],
  ["viewport props styles split by prop role", viewportPropsCssRolePaths.every((path) => viewportPropsCssEntrySource.includes(path))
    && !viewportPropsCssEntrySource.includes(".zr-")
    && viewportPropsCssSource.includes(".zr-scene-cargo")
    && viewportPropsCssSource.includes(".zr-crate")
    && viewportPropsCssSource.includes(".zr-selection-edge")
    && viewportPropsCssSource.includes(".zr-transform-axis")],
  ["viewport tools styles split by overlay role", viewportToolsCssRolePaths.every((path) => viewportToolsCssEntrySource.includes(path))
    && !viewportToolsCssEntrySource.includes(".zr-")
    && viewportToolsCssSource.includes(".zr-axis-mini")
    && viewportToolsCssSource.includes(".zr-orientation-gizmo")
    && viewportToolsCssSource.includes(".zr-scene-vignette")
    && viewportToolsCssSource.includes(".zr-viewport-tools")],
  ["workbench lower demo styles split by role", workbenchLowerDemoCssRolePaths.every((path) => workbenchCssEntrySource.includes(path))
    && !workbenchCssEntrySource.includes(".zr-")
    && workbenchLowerDemoCssSource.includes(".zr-lower-demo")
    && workbenchLowerDemoCssSource.includes(".zr-alert-stack")
    && workbenchLowerDemoCssSource.includes(".zr-table .zr-table-row")
    && workbenchLowerDemoCssSource.includes(".zr-toast")
    && workbenchLowerDemoCssSource.includes("filter: blur")
    && workbenchLowerDemoCssSource.includes(".zr-tooltip-bubble")],
  ["statusbar tuning styles split by role", statusbarTuningCssRolePaths.every((path) => statusbarTuningCssEntrySource.includes(path))
    && !statusbarTuningCssEntrySource.includes(".zr-")
    && statusbarTuningCssSource.includes(".zr-popup-layer")
    && statusbarTuningCssSource.includes(".zr-status-left")
    && statusbarTuningCssSource.includes(".zr-status-right")
    && statusbarTuningCssSource.includes(".zr-statusbar")
    && statusbarTuningCssSource.includes(".zr-statusbar .zr-select")],
  ["side panel tuning styles split by role", sidePanelsCssRolePaths.every((path) => sidePanelsCssEntrySource.includes(path))
    && !sidePanelsCssEntrySource.includes(".zr-")
    && sidePanelsCssSource.includes(".zr-side-stack")
    && sidePanelsCssSource.includes(".zr-alt-panel")
    && sidePanelsCssSource.includes(".zr-layer-row")
    && sidePanelsCssSource.includes(".zr-console-row")
    && sidePanelsCssSource.includes(".zr-inspector .zr-checkbox")
    && sidePanelsCssSource.includes(".zr-form-row .zr-select")
    && sidePanelsCssSource.includes(".zr-topbar .zr-select:has")],
  ["showcase control styles split by control family", showcaseControlCssRolePaths.every((path) => showcaseControlCssEntrySource.includes(path))
    && !showcaseControlCssEntrySource.includes(".zr-")
    && showcaseControlCssSource.includes(".zr-icon-button.is-lg")
    && showcaseControlCssSource.includes(".zr-control-grid")
    && showcaseControlCssSource.includes(".zr-input:focus")
    && showcaseControlCssSource.includes(".zr-checkbox")
    && showcaseControlCssSource.includes(".zr-segment")
    && showcaseControlCssSource.includes(".zr-slider-track")
    && showcaseControlCssSource.includes(".zr-tab.is-active")],
  ["showcase button grid styles split by button-grid role", showcaseButtonGridCssRolePaths.every((path) => showcaseButtonGridCssEntrySource.includes(path))
    && !showcaseButtonGridCssEntrySource.includes(".zr-")
    && showcaseButtonGridCssSource.includes(".zr-showcase-col:first-child .zr-control-grid")
    && showcaseButtonGridCssSource.includes(".zr-showcase-col:first-child .zr-select")
    && showcaseButtonGridCssSource.includes(".zr-showcase-col:first-child .zr-button:disabled")
    && showcaseButtonGridCssSource.includes(".zr-showcase-col:first-child .zr-control-grid > :nth-child(8) .zr-icon")],
  ["inspector detail styles split by inspector role", inspectorDetailCssRolePaths.every((path) => inspectorDetailCssEntrySource.includes(path))
    && !inspectorDetailCssEntrySource.includes(".zr-")
    && inspectorDetailCssSource.includes(".zr-inspector .zr-button")
    && inspectorDetailCssSource.includes(".zr-scene-panel .zr-tree")
    && inspectorDetailCssSource.includes(".zr-inspector .zr-form-row")
    && inspectorTransformCssSource.includes(".zr-section.is-transform")
    && inspectorDetailCssSource.includes(".zr-section.is-mesh-renderer")],
  ["inspector transform styles split by transform role", inspectorTransformCssRolePaths.every((path) => inspectorTransformCssEntrySource.includes(path))
    && !inspectorTransformCssEntrySource.includes(".zr-")
    && inspectorTransformCssSource.includes(".zr-section.is-transform")
    && inspectorTransformCssSource.includes(".zr-value-box")
    && inspectorTransformCssSource.includes(".zr-vector-row:nth-of-type(2)")
    && inspectorTransformCssSource.includes(".zr-linked-axis")
    && inspectorTransformCssSource.includes("> span:nth-child(2)")
    && inspectorTransformCssSource.includes(".zr-checkbox.is-checked")],
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
  ["extension module registry", extensionModules.length === expectedExtensionCount && modules.length === nativeModules.length + extensionModules.length + 2 && webModuleTabs.length === nativeModules.length + 2 && representativeExtensionIds.every((id) => extensionModules.some((module) => module.id === id))],
  ["component lab web-only module boundary", modulesSource.includes("../component-lab/module.js") && webModuleTabs.some((module) => module.id === "component-lab" && module.webOnly === true) && !nativeModules.some((module) => module.id === "component-lab") && ["export const componentNav", "export const labToolbarPanels", "from \"./bottom.js\"", "from \"./center.js\"", "from \"./details.js\"", "from \"./left.js\""].every((needle) => componentLabSource.includes(needle)) && ["export const componentLabModule", "export function componentLabLeft", "export function componentLabCenter", "export function componentLabDetails", "export function componentLabBottom", "export function componentLabRouteOptions"].every((needle) => componentLabSource.includes(needle))],
  ["core module registry boundary", modulesSource.includes("../core/core-modules.js") && !modulesSource.includes("const coreModules") && coreModulesEntrySource.includes("export const coreModules") && ["scene", "gameplay-effect", "gameplay-ability", "gameplay-tags", "ai-perception", "material", "behavior-tree", "render-pipeline", "asset-browser", "vfx", "hud-editor"].every((id) => coreModulesSource.includes(`id: "${id}"`)) && ["left: ()", "center: ()", "right: ()", "bottom: ()"].every((needle) => coreModulesSource.includes(needle))],
  ["core module registry split by functional domain", coreRegistryDomainPaths.every((path) => coreModulesEntrySource.includes(path.replace("./src/modules/core/", "./"))) && ["indexCoreModules", "gameplayCoreModules", "aiCoreModules", "renderingCoreModules", "assetCoreModules", "hudCoreModules"].every((name) => coreModulesSource.includes(name)) && !coreModulesEntrySource.includes('id: "gameplay-effect"') && !coreModulesEntrySource.includes("../core-module-lefts.js")],
  ["core gameplay registry split by concrete module path", coreGameplayRegistryModulePaths.every((path) => coreGameplayRegistryEntrySource.includes(path.replace("./src/modules/core/registry/", "./"))) && !coreGameplayRegistryEntrySource.includes('id: "gameplay-effect"') && !coreGameplayRegistryEntrySource.includes('id: "gameplay-ability"') && !coreGameplayRegistryEntrySource.includes('id: "gameplay-tags"') && ["gameplayEffectCoreModule", "gameplayAbilityCoreModule", "gameplayTagsCoreModule"].every((name) => coreGameplayRegistrySource.includes(`export const ${name}`))],
  ["core ai registry split by concrete module path", coreAiRegistryModulePaths.every((path) => coreAiRegistryEntrySource.includes(path.replace("./src/modules/core/registry/", "./"))) && !coreAiRegistryEntrySource.includes('id: "ai-perception"') && !coreAiRegistryEntrySource.includes('id: "behavior-tree"') && ["aiPerceptionCoreModule", "behaviorTreeCoreModule"].every((name) => coreAiRegistrySource.includes(`export const ${name}`))],
  ["core rendering registry split by concrete module path", coreRenderingRegistryModulePaths.every((path) => coreRenderingRegistryEntrySource.includes(path.replace("./src/modules/core/registry/", "./"))) && !coreRenderingRegistryEntrySource.includes('id: "material"') && !coreRenderingRegistryEntrySource.includes('id: "render-pipeline"') && !coreRenderingRegistryEntrySource.includes('id: "vfx"') && ["materialCoreModule", "renderPipelineCoreModule", "vfxCoreModule"].every((name) => coreRenderingRegistrySource.includes(`export const ${name}`))],
  ["extension config generator boundary", extensionModulesSource.includes("./extension-configs.js") && !extensionModulesSource.includes("recipeByKind") && !extensionModulesSource.includes("./reference-samples.js") && !extensionModulesSource.includes("./extension-blueprints.js") && extensionConfigsSource.includes("export const extensionModuleConfigs") && extensionConfigsSource.includes("../../../foundation/reference-samples.js") && extensionConfigsSource.includes("../extension-blueprints.js")],
  ["extension configs split by generation role", ["configs/factory.js", "configs/sources.js", "configs/text.js"].every((path) => extensionConfigsEntrySource.includes(path)) && !extensionConfigsEntrySource.includes("recipeByKind") && !extensionConfigsEntrySource.includes("createReferenceExtensionConfig") && ["recipeByKind", "layoutKindFor", "hydrateSettings", "assetsFor", "titleWord"].every((needle) => extensionConfigsSource.includes(needle))],
  ["extension config recipes split by functional domain", ["./recipes/animation.js", "./recipes/data.js", "./recipes/default.js", "./recipes/diagnostics.js", "./recipes/gameplay.js", "./recipes/online.js", "./recipes/production.js", "./recipes/rendering.js", "./recipes/runtime.js", "./recipes/simulation.js", "./recipes/ui.js", "./recipes/vfx.js", "./recipes/world.js"].every((path) => extensionRecipesEntrySource.includes(path)) && !/^\s*(?:world|rendering|animation|ui|production|diagnostics|online|simulation|data|gameplay|runtime|vfx|default):\s*\{/m.test(extensionRecipesEntrySource) && ["export const worldRecipe", "export const renderingRecipe", "export const animationRecipe", "export const uiRecipe", "export const productionRecipe", "export const diagnosticsRecipe", "export const onlineRecipe", "export const simulationRecipe", "export const dataRecipe", "export const gameplayRecipe", "export const runtimeRecipe", "export const vfxRecipe", "export const defaultRecipe"].every((needle) => extensionConfigsSource.includes(needle))],
  ["workbench module shell split by role", ["./workbench/registry.js", "./workbench/navigation.js", "./workbench/toolbar.js", "./workbench/rail.js", "./workbench/workspace.js"].every((path) => modulesEntrySource.includes(path)) && !modulesEntrySource.includes("function moduleTabs") && !modulesEntrySource.includes("function moduleWorkspace") && ["export const webModuleTabs", "export function moduleTabs", "export function moduleToolbar", "export function moduleRail", "export function moduleWorkspace"].every((needle) => modulesSource.includes(needle))],
  ["extension module generator boundary", modulesSource.includes("../extensions/extension-modules.js") && modulesSource.includes("buildExtensionModules(coreModules, defaultModuleId)") && extensionModulesSource.includes("export function buildExtensionModules") && extensionModulesSource.includes("createExtensionModule") && extensionModulesSource.includes("extensionBottomOutput")],
  ["extension surface renderer boundary", extensionModulesSource.includes("./extension-surfaces.js") && !/function extension(?:Left|Center|Details|BottomOutput|PrimaryPanel|ValidationPanel|ReferencesPanel)\(/.test(extensionModulesSource) && ["extensionLeft", "extensionCenter", "extensionDetails", "extensionBottomOutput"].every((name) => extensionSurfacesSource.includes(`export function ${name}`)) && extensionSurfacesSource.includes("panelGroup(`${config.id}-right`") && extensionSurfacesSource.includes("panelGroup(`module-bottom-${config.id}`") && extensionSurfacesSource.includes('data-extension-blueprint="${config.blueprint ? "reference" : "recipe"}')],
  ["extension surfaces split by surface role", ["surfaces/left.js", "surfaces/center.js", "surfaces/details.js", "surfaces/bottom.js"].every((path) => extensionSurfacesEntrySource.includes(path)) && !extensionSurfacesEntrySource.includes("export function extensionLeft") && !extensionSurfacesEntrySource.includes("function extensionPrimaryPanel") && ["export function extensionPrimaryPanel", "export function extensionRouteOptions", "export const esc"].every((needle) => extensionSurfacesSource.includes(needle))],
  ["extension primary surfaces split by primary role", extensionPrimaryEntrySource.includes("./primary/panel.js") && !extensionPrimaryEntrySource.includes("switch (config.layoutKind)") && !extensionPrimaryEntrySource.includes("function blueprintPrimaryPanel") && ["export function extensionPrimaryPanel", "export function blueprintPrimaryPanel", "export function layoutKindPrimaryPanel", "export function graphNodes", "export function extensionLinks"].every((needle) => extensionPrimarySource.includes(needle)) && extensionPrimarySource.includes("./blueprint.js") && extensionPrimarySource.includes("./layout-kind.js") && extensionPrimarySource.includes("./graph.js")],
  ["extension bottom surfaces split by bottom role", extensionBottomEntrySource.includes("./bottom/panel.js") && !extensionBottomEntrySource.includes("function extensionValidationPanel") && !extensionBottomEntrySource.includes("moduleTable(") && ["export function extensionBottomOutput", "export function extensionOutputPanel", "export function extensionValidationPanel", "export function extensionReferencesPanel", "export function extensionBottomHandoffPanel"].every((needle) => extensionBottomSource.includes(needle)) && ["./output.js", "./validation.js", "./references.js", "./handoff.js"].every((path) => extensionBottomSource.includes(path)) && ["workbench.extension.output", "workbench.extension.validation", "workbench.extension.references"].every((scope) => extensionBottomSource.includes(scope)) && extensionHandoffSource.includes("workbench.extension.handoff")],
  ["extension left surfaces split by drawer role", extensionLeftEntrySource.includes("./left/panel.js") && !extensionLeftEntrySource.includes('panel("Reference"') && !extensionLeftEntrySource.includes("searchInput(") && ["export function extensionLeft", "export function extensionReferencePanel", "export function extensionToolsPanel", "export function extensionAssetsPanel"].every((needle) => extensionLeftSource.includes(needle)) && ["./reference.js", "./tools.js", "./assets.js"].every((path) => extensionLeftSource.includes(path)) && ["workbench.extension.tool", "workbench.extension.asset"].every((scope) => extensionLeftSource.includes(scope))],
  ["extension center surfaces split by center role", extensionCenterEntrySource.includes("./center/panel.js") && !extensionCenterEntrySource.includes("extensionPrimaryPanel") && !extensionCenterEntrySource.includes("compactStats(") && ["export function extensionCenter", "export function extensionMetricsPanel", "export function extensionReferenceRhythmPanel"].every((needle) => extensionCenterSource.includes(needle)) && ["./metrics.js", "./reference-rhythm.js"].every((path) => extensionCenterSource.includes(path)) && extensionCenterSource.includes('data-extension-blueprint="${config.blueprint ? "reference" : "recipe"}') && extensionCenterSource.includes("workbench.extension.reference")],
  ["extension right details split by details role", extensionDetailsEntrySource.includes("./details/panel.js") && !extensionDetailsEntrySource.includes("moduleTable(") && !extensionDetailsEntrySource.includes("alerts(") && ["export function extensionDetails", "export function extensionSummaryPanel", "export function extensionDetailTablePanel", "export function extensionDetailStatusPanel"].every((needle) => extensionDetailsSource.includes(needle)) && ["./summary.js", "./table.js", "./status.js"].every((path) => extensionDetailsSource.includes(path)) && extensionDetailsSource.includes("workbench.extension.detail")],
  ["extension library module boundary", extensionModulesSource.includes("./extension-library.js") && extensionLibrarySource.includes("export function createEditorLibraryModule") && extensionLibrarySource.includes("data-library-drilldown=\"reference-blueprints\"")],
  ["extension library split by module role", extensionLibraryEntrySource.includes("./library/module.js") && !extensionLibraryEntrySource.includes("function editorLibraryCenter") && !extensionLibraryEntrySource.includes("panel(") && ["./left.js", "./center.js", "./details.js", "./bottom.js", "./cards.js", "./drilldown.js", "./rows.js", "./routes.js"].every((path) => extensionLibrarySource.includes(path)) && ["export function editorLibraryLeft", "export function editorLibraryCenter", "export function extensionModuleCard", "export function referenceBlueprintDrilldown", "export function editorLibraryDetails", "export function editorLibraryBottom", "export function referenceGroupsList", "export function libraryRouteOptions"].every((needle) => extensionLibrarySource.includes(needle))],
  ["extension handoff panel boundary", extensionSurfacesSource.includes("extension-handoff.js") && extensionHandoffSource.includes("export function extensionHandoffPanel") && extensionHandoffSource.includes("pending .zui workspace")],
  ["component lab center split by palette role", ["./center/atom-palette.js", "./center/collection-palette.js", "./center/coverage-matrix.js", "./center/layout-grammar.js", "./center/surface-palette.js"].every((path) => componentLabCenterEntrySource.includes(path)) && componentLabSource.includes("./lab-column.js") && !componentLabCenterEntrySource.includes("function atomPalette") && !componentLabCenterEntrySource.includes("function collectionPalette") && !componentLabCenterEntrySource.includes("function surfacePalette") && ["export function atomPalette", "export function collectionPalette", "export function componentCoverageMatrix", "export function layoutGrammarPanel", "export function surfacePalette", "export function labColumn"].every((needle) => componentLabSource.includes(needle))],
  ["extension blueprint data boundary", extensionBlueprintEntrySource.includes("export const extensionBlueprints") && extensionBlueprintsSource.includes('"shader-editor"') && extensionBlueprintsSource.includes('"source-control"') && extensionBlueprintsSource.includes('"weather-editor"') && extensionBlueprintsSource.includes("export function tablePrimary")],
  ["extension blueprint data split by functional domain", extensionBlueprintDomainPaths.every((path) => extensionBlueprintEntrySource.includes(path.replace("./src/modules/extensions/", "./"))) && ["animationBlueprints", "dataBlueprints", "diagnosticsBlueprints", "gameplayBlueprints", "multiplayerBlueprints", "productionBlueprints", "renderingBlueprints", "simulationBlueprints", "uiBlueprints", "worldBlueprints"].every((name) => extensionBlueprintsSource.includes(`export const ${name}`)) && !extensionBlueprintEntrySource.includes('"shader-editor"') && !extensionBlueprintEntrySource.includes("export function tablePrimary")],
  ["animation extension blueprints split by editor path", extensionBlueprintAnimationRolePaths.every((path) => extensionBlueprintAnimationEntrySource.includes(path))
    && !extensionBlueprintAnimationEntrySource.includes('"sequencer": blueprint')
    && !extensionBlueprintAnimationEntrySource.includes("timelinePrimary(")
    && ["sequencerBlueprint", "montageEditorBlueprint", "animationCompressionBlueprint", "blendSpaceBlueprint", "controlRigBlueprint", "motionMatchingBlueprint", "poseLibraryBlueprint", "retargetBlueprint"].every((name) => extensionBlueprintAnimationSource.includes(`export const ${name}`))],
  ["world extension blueprints split by editor path", extensionBlueprintWorldRolePaths.every((path) => extensionBlueprintWorldEntrySource.includes(path))
    && !extensionBlueprintWorldEntrySource.includes('"terrain-editor": blueprint')
    && !extensionBlueprintWorldEntrySource.includes("graphPrimary(")
    && ["terrainEditorBlueprint", "foliageEditorBlueprint", "levelStreamingBlueprint", "levelVariantBlueprint", "prefabEditorBlueprint", "scatterEditorBlueprint", "volumeEditorBlueprint", "weatherEditorBlueprint"].every((name) => extensionBlueprintWorldSource.includes(`export const ${name}`))],
  ["extension module category recipes", extensionConfigsSource.includes("recipeByKind") && ["world", "rendering", "animation", "ui", "production", "diagnostics", "online", "simulation", "data", "gameplay", "runtime", "vfx"].every((kind) => extensionConfigsSource.includes(`${kind}:`))],
  ["extension module varied layouts", extensionHtml.includes("is-extension is-extension-world") && moduleWorkspace("shader-editor").includes("is-extension-rendering") && moduleWorkspace("sequencer").includes("is-extension-animation") && moduleWorkspace("source-control").includes("is-extension-production") && moduleWorkspace("weather-editor").includes("is-extension-world")],
  ["module shared component layer", modulesSource.includes("../shared/module-components.js") && moduleComponentsSource.includes("export function moduleMain") && moduleComponentsSource.includes("export function moduleTable")],
  ["module shared components split by component role", [
    "./actions.js",
    "./bottom-output.js",
    "./panels.js",
    "./rows.js",
    "./regions.js",
    "./visuals.js",
    "./utils.js"
  ].every((path) => moduleComponentsEntrySource.includes(path))
    && !moduleComponentsEntrySource.includes("export function moduleMain")
    && !moduleComponentsEntrySource.includes("function generatedBottomRows")
    && ["export function panelGroup", "export function bottomOutput", "export function moduleTable", "export function graphBoard", "export function actionButton"].every((needle) => moduleComponentsSource.includes(needle))],
  ["dotted action path helper", actionPathsSource.includes("export function actionPath") && actionPathsSource.includes("export function actionRouteKey") && routesSource.includes("actionRouteKey") && appSource.includes("normalizeActionId")],
  ["core module left boundary", coreModulesSource.includes("../core-module-lefts.js") && !/left:\s*\(\)\s*=>\s*\[/.test(modulesSource) && !/left:\s*\(\)\s*=>\s*[\s\S]{0,60}panel\(/.test(modulesSource) && coreLeftRendererNames.every((name) => coreModuleLeftsSource.includes(`export function ${name}`)) && coreModuleLeftsSource.includes('panel("Widget Palette"') && coreModuleLeftsSource.includes("panelGroup(\"hud-assets\"")],
  ["core left panels split by functional domain", coreLeftDomainPaths.every((path) => coreModuleLeftsEntrySource.includes(path.replace("./src/modules/core/", "./"))) && !coreModuleLeftsEntrySource.includes("export function sceneLeft") && !coreModuleLeftsEntrySource.includes("panel(\"Widget Palette\"") && ["lefts/gameplay.js", "lefts/ai.js", "lefts/rendering.js", "lefts/assets.js", "lefts/ui.js"].every((path) => coreModuleLeftsEntrySource.includes(path))],
  ["core gameplay left panels split by concrete module path", coreGameplayLeftModulePaths.every((path) => coreGameplayLeftEntrySource.includes(path.replace("./src/modules/core/lefts/", "./"))) && !coreGameplayLeftEntrySource.includes("export function gameplayLeft") && !coreGameplayLeftEntrySource.includes("export function abilityLeft") && !coreGameplayLeftEntrySource.includes("export function tagsLeft") && ["Effect Tools", "Ability Task Palette", "Tag Actions"].every((title) => coreGameplayLeftSource.includes(`panel("${title}"`))],
  ["core ai left panels split by concrete module path", coreAiLeftModulePaths.every((path) => coreAiLeftEntrySource.includes(path.replace("./src/modules/core/lefts/", "./"))) && !coreAiLeftEntrySource.includes("export function perceptionLeft") && !coreAiLeftEntrySource.includes("export function behaviorLeft") && ["Sense Tools", "Node Palette"].every((title) => coreAiLeftSource.includes(`panel("${title}"`))],
  ["core rendering left panels split by concrete module path", coreRenderingLeftModulePaths.every((path) => coreRenderingLeftEntrySource.includes(path.replace("./src/modules/core/lefts/", "./"))) && !coreRenderingLeftEntrySource.includes("export function materialLeft") && !coreRenderingLeftEntrySource.includes("export function renderPipelineLeft") && !coreRenderingLeftEntrySource.includes("export function vfxLeft") && ["Node Palette", "Pass Palette", "Emitter Library"].every((title) => coreRenderingLeftSource.includes(`panel("${title}"`))],
  ["core module bottom boundary", coreModulesSource.includes("../core-module-bottoms.js") && !/^function \w+Bottom\(/m.test(modulesSource) && coreBottomRendererNames.every((name) => coreModuleBottomsSource.includes(`export function ${name}`)) && coreModuleBottomsSource.includes("zr-module-output-grid")],
  ["core bottom outputs split by functional domain", coreBottomDomainPaths.filter((path) => !path.endsWith("/routes.js")).every((path) => coreModuleBottomsEntrySource.includes(path.replace("./src/modules/core/", "./"))) && !coreModuleBottomsEntrySource.includes("export function sceneBottom") && !coreModuleBottomsEntrySource.includes("coreBottomRouteOptions") && ["bottoms/gameplay.js", "bottoms/ai.js", "bottoms/rendering.js", "bottoms/assets.js", "bottoms/ui.js"].every((path) => coreModuleBottomsEntrySource.includes(path)) && coreModuleBottomsSource.includes("export function coreBottomRouteOptions")],
  ["core gameplay bottom outputs split by concrete module path", coreGameplayBottomModulePaths.every((path) => coreGameplayBottomEntrySource.includes(path.replace("./src/modules/core/bottoms/", "./"))) && !coreGameplayBottomEntrySource.includes("export function gameplayBottom") && !coreGameplayBottomEntrySource.includes("export function abilityBottom") && !coreGameplayBottomEntrySource.includes("export function tagsBottom") && ["gameplay-effect", "gameplay-ability", "gameplay-tags", "Apply GE", "Ability Activated", "DefaultGameplayTags.ini"].every((text) => coreGameplayBottomSource.includes(text))],
  ["core ai bottom outputs split by concrete module path", coreAiBottomModulePaths.every((path) => coreAiBottomEntrySource.includes(path.replace("./src/modules/core/bottoms/", "./"))) && !coreAiBottomEntrySource.includes("export function perceptionBottom") && !coreAiBottomEntrySource.includes("export function behaviorBottom") && ["ai-perception", "Hearing stimulus", "BT_Enemy"].every((text) => coreAiBottomSource.includes(text))],
  ["core rendering bottom outputs split by concrete module path", coreRenderingBottomModulePaths.every((path) => coreRenderingBottomEntrySource.includes(path.replace("./src/modules/core/bottoms/", "./"))) && !coreRenderingBottomEntrySource.includes("export function materialBottom") && !coreRenderingBottomEntrySource.includes("export function renderPipelineBottom") && !coreRenderingBottomEntrySource.includes("export function vfxBottom") && ["material", "render-pipeline", "vfx", "M_Rock_Cliff", "Frame 1234 captured", "P_Bolt_01"].every((text) => coreRenderingBottomSource.includes(text))],
  ["core module center boundary", coreModulesSource.includes("../core-module-centers.js") && !/^function \w+Center\(/m.test(modulesSource) && !/function (?:perceptionMap|hudCanvas)\(/.test(modulesSource) && ["sceneCenter", "gameplayCenter", "abilityCenter", "tagsCenter", "perceptionCenter", "materialCenter", "behaviorCenter", "renderPipelineCenter", "assetCenter", "vfxCenter", "hudCenter"].every((name) => coreModuleCentersSource.includes(`export function ${name}`)) && coreModuleCentersSource.includes("function perceptionMap") && coreModuleCentersSource.includes("function hudCanvas")],
  ["core center workspaces split by functional domain", coreCenterDomainPaths.every((path) => coreModuleCentersEntrySource.includes(path.replace("./src/modules/core/", "./"))) && !coreModuleCentersEntrySource.includes("export function sceneCenter") && !coreModuleCentersEntrySource.includes("function perceptionMap") && ["centers/gameplay.js", "centers/ai.js", "centers/rendering.js", "centers/assets.js", "centers/ui.js"].every((path) => coreModuleCentersEntrySource.includes(path))],
  ["core gameplay center workspaces split by concrete module path", coreGameplayCenterModulePaths.every((path) => coreGameplayCenterEntrySource.includes(path.replace("./src/modules/core/centers/", "./"))) && !coreGameplayCenterEntrySource.includes("export function gameplayCenter") && !coreGameplayCenterEntrySource.includes("export function abilityCenter") && !coreGameplayCenterEntrySource.includes("export function tagsCenter") && ["Modifiers", "Ability Graph", "Gameplay Tag Registry"].every((title) => coreGameplayCenterSource.includes(`panel("${title}"`))],
  ["core ai center workspaces split by concrete module path", coreAiCenterModulePaths.every((path) => coreAiCenterEntrySource.includes(path.replace("./src/modules/core/centers/", "./"))) && !coreAiCenterEntrySource.includes("export function perceptionCenter") && !coreAiCenterEntrySource.includes("export function behaviorCenter") && !coreAiCenterEntrySource.includes("function perceptionMap") && ["World Perception Map", "BT_Enemy"].every((title) => coreAiCenterSource.includes(`panel("${title}"`))],
  ["core rendering center workspaces split by concrete module path", coreRenderingCenterModulePaths.every((path) => coreRenderingCenterEntrySource.includes(path.replace("./src/modules/core/centers/", "./"))) && !coreRenderingCenterEntrySource.includes("export function materialCenter") && !coreRenderingCenterEntrySource.includes("export function renderPipelineCenter") && !coreRenderingCenterEntrySource.includes("export function vfxCenter") && ["Material Graph", "Render Graph", "Emitter Stack"].every((title) => coreRenderingCenterSource.includes(`panel("${title}"`))],
  ["core module detail boundary", coreModulesSource.includes("../core-module-details.js") && !/^function \w+Details\(/m.test(modulesSource) && nativeRightPanelIds.every((panel) => coreModuleDetailsSource.includes(`panelGroup("${panel}"`)) && ["sceneDetails", "gameplayDetails", "abilityDetails", "tagsDetails", "perceptionDetails", "materialDetails", "behaviorDetails", "renderPipelineDetails", "assetDetails", "vfxDetails", "hudDetails"].every((name) => coreModuleDetailsSource.includes(`export function ${name}`))],
  ["core detail panels split by functional domain", coreDetailDomainPaths.filter((path) => !path.endsWith("/routes.js")).every((path) => coreModuleDetailsEntrySource.includes(path.replace("./src/modules/core/", "./"))) && !coreModuleDetailsEntrySource.includes("export function sceneDetails") && !coreModuleDetailsEntrySource.includes("coreRightRouteOptions") && ["details/gameplay.js", "details/ai.js", "details/rendering.js", "details/assets.js", "details/ui.js"].every((path) => coreModuleDetailsEntrySource.includes(path)) && coreModuleDetailsSource.includes("export function coreRightRouteOptions")],
  ["core gameplay detail panels split by concrete module path", coreGameplayDetailModulePaths.every((path) => coreGameplayDetailEntrySource.includes(path.replace("./src/modules/core/details/", "./"))) && !coreGameplayDetailEntrySource.includes("export function gameplayDetails") && !coreGameplayDetailEntrySource.includes("export function abilityDetails") && !coreGameplayDetailEntrySource.includes("export function tagsDetails") && ["gameplay-right", "ability-right", "tags-right"].every((panel) => coreGameplayDetailSource.includes(`panelGroup("${panel}"`))],
  ["core ai detail panels split by concrete module path", coreAiDetailModulePaths.every((path) => coreAiDetailEntrySource.includes(path.replace("./src/modules/core/details/", "./"))) && !coreAiDetailEntrySource.includes("export function perceptionDetails") && !coreAiDetailEntrySource.includes("export function behaviorDetails") && ["perception-right", "behavior-right"].every((panel) => coreAiDetailSource.includes(`panelGroup("${panel}"`))],
  ["core rendering detail panels split by concrete module path", coreRenderingDetailModulePaths.every((path) => coreRenderingDetailEntrySource.includes(path.replace("./src/modules/core/details/", "./"))) && !coreRenderingDetailEntrySource.includes("export function materialDetails") && !coreRenderingDetailEntrySource.includes("export function renderPipelineDetails") && !coreRenderingDetailEntrySource.includes("export function vfxDetails") && ["material-right", "render-right", "vfx-right"].every((panel) => coreRenderingDetailSource.includes(`panelGroup("${panel}"`))],
  ["shared panel group component", moduleComponentsSource.includes("export function panelGroup") && moduleComponentsSource.includes('data-panel-group="${esc(panel)}"') && moduleComponentsSource.includes("panelGroup(panelId") && extensionLibrarySource.includes('panelGroup("library-drilldown"') && extensionLibrarySource.includes('panelGroup("library-right"') && extensionSurfacesSource.includes("panelGroup(`${config.id}-right`") && extensionSurfacesSource.includes("panelGroup(`module-bottom-${config.id}`")],
  ["bottom output uses shared panel group", moduleComponentsSource.includes("export function bottomOutput") && moduleComponentsSource.includes("panelGroup(panelId") && !moduleComponentsSource.includes("zr-module-bottom-body") && coreAndLibraryWorkspaceHtml.includes('data-panel-group="module-bottom-gameplay-effect"') && coreAndLibraryWorkspaceHtml.includes('class="zr-panel-group is-module-bottom"')],
  ["native right panels use shared panel group", coreModuleDetailsSource.includes("panelGroup") && nativeRightPanelIds.every((panel) => coreAndLibraryWorkspaceHtml.includes(`data-panel-group="${panel}"`) && coreAndLibraryWorkspaceHtml.includes(`data-panel-view="${panel}:`)) && !/function \w+Details\(\) \{\s*return `\$\{panelTabs/.test(coreModuleDetailsSource)],
  ["surface panels use shared panel group", surfacesSource.includes("../../modules/shared/module-components.js") && surfacesSource.includes('panelGroup("scene"') && surfacesSource.includes('panelGroup("inspector"') && surfacesSource.includes('panelGroup("showcase"') && !/function panel(?:Tabs|View)\b/.test(surfacesSource) && ["scene", "inspector", "showcase"].every((panel) => surfacePanelHtml.includes(`data-panel-group="${panel}"`) && surfacePanelHtml.includes(`data-panel-host="${panel}"`))],
  ["library and extension right panels use shared panel group", libraryHtml.includes('data-panel-group="library-right"') && libraryHtml.includes('data-panel-view="library-right:routing"') && extensionHtml.includes(`data-panel-group="${extensionModules[0]?.id}-right"`) && extensionWorkspaceHtml.includes('class="zr-panel-group is-extension-right"') && !/panel(?:Tabs|View)\(/.test(extensionLibrarySource) && !/panel(?:Tabs|View)\(/.test(extensionModulesSource) && !/panel(?:Tabs|View)\(/.test(extensionSurfacesSource)],
  ["module embedded card tabs use shared panel group", !modulesSource.includes("panelTabs") && coreModuleLeftsSource.includes('panelGroup("tag-sources"') && coreModuleLeftsSource.includes('panelGroup("hud-assets"') && ["tag-sources:plugins", "tag-sources:native-sets", "hud-assets:screens"].every((target) => coreAndLibraryWorkspaceHtml.includes(`data-panel-tab="${target}"`) && coreAndLibraryWorkspaceHtml.includes(`data-panel-view="${target}"`))],
  ["semantic module table action ids", moduleComponentsSource.includes("function tableRowActionId") && moduleComponentsSource.includes("function tableRowLabel") && moduleComponentsSource.includes("readableCell") && moduleComponentsSource.includes("aria-label") && moduleComponentsSource.includes("!/^\\d+(?:\\.\\d+)?$/")],
  ["module workspace uses layout primitives", appHtml.includes("zr-module-editor-grid") && appHtml.includes("zr-layout") && appHtml.includes("zr-grid")],
  ["toolbar uses cluster primitive", appHtml.includes("zr-topbar-tools") && appHtml.includes("zr-cluster")],
  ["module tabs in topbar", appHtml.includes("zr-module-tabs") && (appHtml.match(/class="zr-module-tab(?:\s|")/g) ?? []).length === webModuleTabs.length],
  ["extension module library cards", libraryHtml.includes("zr-extension-card-grid") && (libraryHtml.match(/data-module-source="extension-library"/g) ?? []).length === extensionModules.length],
  ["extension library blueprint drilldown", libraryHtml.includes('data-library-drilldown="reference-blueprints"') && libraryHtml.includes('data-panel-group="library-drilldown"') && libraryHtml.includes('data-panel-tab="library-drilldown:blueprints"') && libraryHtml.includes('data-panel-view="library-drilldown:components"') && libraryHtml.includes("shader-editor") && libraryHtml.includes("world-state") && extensionLibrarySource.includes("representativeConfigs")],
  ["extension library explicit panel routes", extensionLibrarySource.includes("libraryRouteOptions") && ["workbench.library.group", "workbench.library.core", "workbench.library.blueprint", "workbench.library.component", "workbench.library.route", "workbench.library.reference", "workbench.library.catalog", "workbench.library.coverage"].every((scope) => extensionLibrarySource.includes(scope)) && ["module-bottom-editor-library:coverage", "module-bottom-editor-library:reference-notes", "module-bottom-editor-library:routing-log", "library-right:catalog", "library-right:coverage", "library-right:routing"].every((target) => libraryHtml.includes(`data-route-panel="${target}"`))],
  ["component lab explicit panel routes", componentLabSource.includes("componentLabRouteOptions") && ["workbench.component_lab.left", "workbench.component_lab.coverage", "workbench.component_lab.layout", "workbench.component_lab.asset", "workbench.component_lab.atom", "workbench.component_lab.collection", "workbench.component_lab.surface", "workbench.component_lab.responsive", "workbench.component_lab.audit"].every((scope) => componentLabSource.includes(scope)) && ["component-lab-right:inputs", "component-lab-right:layout", "component-lab-right:surfaces", "component-lab-main:collections", "component-lab-main:surfaces", "module-bottom-component-lab:responsive", "module-bottom-component-lab:routes"].every((target) => coreAndLibraryWorkspaceHtml.includes(`data-route-panel="${target}"`) || coreAndLibraryWorkspaceHtml.includes(`data-panel-tab="${target}"`))],
  ["core right detail panels have explicit internal routes", coreModuleDetailsSource.includes("coreRightRouteOptions") && coreModuleDetailsSource.includes("workbench.module.right") && [
    "scene-right:history",
    "gameplay-right:effect-hierarchy",
    "ability-right:graph-outline",
    "ability-right:validation",
    "tags-right:hierarchy",
    "tags-right:references",
    "tags-right:owners",
    "tags-right:redirects",
    "perception-right:world-overview",
    "perception-right:filters",
    "material-right:graph-outline",
    "behavior-right:bt-outline",
    "behavior-right:execution",
    "render-right:passes",
    "render-right:resources",
    "render-right:frame-stages",
    "asset-right:references",
    "vfx-right:system-overview",
    "vfx-right:stages",
    "hud-right:widget-hierarchy",
    "hud-right:bindings"
  ].every((target) => coreAndLibraryWorkspaceHtml.includes(`data-route-panel="${target}"`))],
  ["core primary bottom panels have explicit internal routes", coreModuleBottomsSource.includes("coreBottomRouteOptions") && coreModuleBottomsSource.includes("workbench.module.bottom") && [
    ["scene", "selection"],
    ["gameplay-effect", "simulation-output"],
    ["gameplay-ability", "timeline"],
    ["gameplay-tags", "validation-log"],
    ["ai-perception", "perception-timeline"],
    ["material", "shader-output"],
    ["render-pipeline", "frame-capture-log"],
    ["asset-browser", "queue"],
    ["vfx", "timeline"],
    ["hud-editor", "validation"]
  ].every(([moduleId, panel]) => moduleWorkspace(moduleId).includes(`data-route-panel="module-bottom-${moduleId}:${panel}"`))],
  ["core secondary bottom panels generated", webModuleTabs.every((module) => {
    const workspace = moduleWorkspace(module.id);
    const tabCount = (workspace.match(new RegExp(`data-panel-tab="module-bottom-${module.id}:[^"]+"`, "g")) ?? []).length;
    const generatedCount = (workspace.match(new RegExp(`data-generated-bottom-panel="module-bottom-${module.id}:[^"]+"`, "g")) ?? []).length;
    return tabCount > 1 && generatedCount === tabCount - 1 && !workspace.includes("zr-module-placeholder");
  }) && moduleComponentsSource.includes("function generatedBottomPanel") && coreAndLibraryWorkspaceHtml.includes("data-generated-bottom-panel")],
  ["generated bottom panels have explicit internal routes", moduleComponentsSource.includes("routeOptions = { actionScope, routePanel: route }") && moduleComponentsSource.includes('actionScope = `workbench.generated_bottom') && webModuleTabs.every((module) => {
    const workspace = moduleWorkspace(module.id);
    const generatedRoutes = [...workspace.matchAll(/data-generated-bottom-panel="([^"]+)"/g)].map((match) => match[1]);
    return generatedRoutes.length > 0 && generatedRoutes.every((route) => workspace.includes(`data-route-panel="${route}"`));
  })],
  ["extension module surfaces", extensionHtml.includes("zr-module-editor-grid is-extension") && extensionHtml.includes('data-extension-blueprint="reference"') && extensionHtml.includes("Reference Specific") && extensionHtml.includes("Prototype Only") && extensionHtml.includes("More Editors")],
  ["extension bottom panels have specific content", extensionModules.every((module) => {
    const workspace = moduleWorkspace(module.id);
    return ["output", "validation", "references", "handoff"].every((panel) => workspace.includes(`data-panel-view="module-bottom-${module.id}:${panel}"`)) && workspace.includes("Reference assets") && workspace.includes("Native Handoff");
  }) && !extensionWorkspaceHtml.match(/data-panel-view="module-bottom-[^"]+:(validation|references)"[\s\S]{0,900}zr-module-placeholder/)],
  ["module toolbar actions", appHtml.includes("zr-module-toolbar") && (appHtml.match(/data-action=/g) ?? []).length >= 12],
  ["rendered web action ids use dotted functional paths", renderedActionIds.length >= 3000 && invalidRenderedActionIds.length === 0 && renderedActionIds.every((id) => id.startsWith("workbench.")) && ["workbench.module.toolbar", "workbench.module.table", "workbench.collection.menu", "workbench.generated_bottom"].every((prefix) => renderedActionIds.some((id) => id.startsWith(prefix)))],
  ["module left drawer", appHtml.includes("zr-module-left") && appHtml.includes('data-module-panel="left"')],
  ["module main surface", appHtml.includes("zr-module-main") && appHtml.includes('data-module-panel="main"')],
  ["module right window", appHtml.includes("zr-module-right") && appHtml.includes('data-module-panel="right"')],
  ["module bottom drawer", appHtml.includes("zr-module-bottom") && appHtml.includes('data-module-panel="bottom"')],
  ["module graph surface", appHtml.includes("zr-module-graph") && appHtml.includes("zr-module-node")],
  ["module table/list/tree surfaces", appHtml.includes("zr-module-table") && appHtml.includes("zr-module-list") && appHtml.includes("zr-module-tree")],
  ["module list action row labels", moduleComponentsSource.includes("export function listRows") && moduleComponentsSource.includes('aria-label="${esc(item)}"') && responsiveSource.includes("workbench.module.list.target_tags")],
  ["module status message", appHtml.includes("data-status-message") && appHtml.includes("zr-module-status-message")],
  ["button atom", appHtml.includes("zr-button")],
  ["input atom", appHtml.includes("zr-input")],
  ["checkbox atom", appHtml.includes("zr-checkbox")],
  ["toggle atom", appHtml.includes("zr-switch")],
  ["icon button atom", appHtml.includes("zr-icon-button")],
  ["tabs atom", appHtml.includes("zr-tabs")],
  ["dropdown atom", appHtml.includes("zr-select") && appHtml.includes("data-dropdown")],
  ["list collection", appHtml.includes("zr-list")],
  ["list collection action rows", collectionDataSource.includes('class="zr-list-item') && collectionDataSource.includes('actionPath("workbench.collection.list", item.label)') && collectionDataSource.includes('aria-label="${esc(item.label)}"') && responsiveSource.includes("assertCollectionRowButtons")],
  ["tree view collection", appHtml.includes("zr-module-tree")],
  ["tree collection action rows", collectionDataSource.includes('<button class="zr-tree-row') && collectionDataSource.includes('actionPath("workbench.collection.tree", node.label)') && collectionDataSource.includes('aria-label="${esc(node.label)}"') && responsiveSource.includes("assertCollectionTreeRows")],
  ["table view collection", appHtml.includes("zr-module-table")],
  ["table collection action rows", collectionDataSource.includes('class="zr-table-row') && collectionDataSource.includes('actionPath("workbench.collection.table", row[0])') && collectionDataSource.includes('aria-label="${esc(row[0])}"') && responsiveSource.includes("workbench.module.table.health_regen")],
  ["popup collection", appHtml.includes("zr-popup-layer")],
  ["module panel tab target", appHtml.includes("data-panel-tab") && appHtml.includes("module-bottom-gameplay-effect")],
  ["window surface", appHtml.includes('data-surface="window"')],
  ["drawer surfaces", (appHtml.match(/data-surface="drawer"/g) ?? []).length >= 2],
  ["module window surface", appHtml.includes('class="zr-panel zr-module-right" data-surface="window"')],
  ["panel view surfaces", (appHtml.match(/data-surface="panel-view"/g) ?? []).length >= 4],
  ["tab aria selected", appHtml.includes('role="tab" aria-selected="true"')],
  ["dropdown popup layer", appHtml.includes('id="popup-layer"')],
  ["menu rows are action buttons", appHtml.includes('class="zr-menu-row') && appHtml.includes('data-menu-item="new"') && appHtml.includes('data-action="workbench.collection.menu.new"') && appSource.includes('action.closest(".zr-popup-layer")')],
  ["app interaction layer split by role", appEntrySource.includes("./src/app/mount.js") && !appEntrySource.includes("addEventListener") && !appEntrySource.includes("function activateModule") && appControllerEntrySource.includes("./controller/create-workbench-controller.js") && !appControllerEntrySource.includes("function activateModule") && ["src/app/mount.js", "src/app/controller.js", "src/app/controller/activation.js", "src/app/controller/command-application.js", "src/app/controller/create-workbench-controller.js", "src/app/controller/command-routing.js", "src/app/controller/history.js", "src/app/controller/location-state.js", "src/app/controller/rendering.js", "src/app/controller/state.js", "src/app/controller/status.js", "src/app/route-state.js", "src/app/labels.js", "src/app/interactions/click.js", "src/app/interactions/click/actions.js", "src/app/interactions/click/dropdowns.js", "src/app/interactions/click/generic.js", "src/app/interactions/click/navigation.js", "src/app/interactions/click/rows.js", "src/app/interactions/click/selection.js", "src/app/interactions/click/tabs.js", "src/app/interactions/click/toolbar.js", "src/app/interactions/click/utils.js", "src/app/interactions/fields.js", "src/app/interactions/keyboard.js", "src/app/interactions/history.js"].every((path) => appRolePaths.some((rolePath) => rolePath.endsWith(path.replace("src/app/", "./src/app/")))) && ["export function mountWorkbenchApp", "export function createWorkbenchController", "export function createActivationHandlers", "export function applyCommandRouteForTarget", "export function renderWorkbenchShell", "export function commandRouteForTarget", "export function syncControllerRouteState", "export function applyLocationModuleState", "export function createControllerState", "export function updateStatusMessage", "export function routeHash", "export function commandLabel", "export function bindClickInteractions", "export function handleActionClick", "export function handleDropdownClick", "export function handleGenericCommandClick", "export function handleModuleNavigation", "export function handleTreeRowClick", "export function handleToggleClick", "export function handleTabClick", "export function handleRailClick", "export function bindFieldInteractions", "export function bindKeyboardActivation", "export function bindHistoryInteractions"].every((needle) => appSource.includes(needle))],
  ["click interaction entry split by dispatch role", appClickInteractionsEntrySource.includes("./click/bind.js") && !appClickInteractionsEntrySource.includes("addEventListener") && !appClickInteractionsEntrySource.includes("clickHandlers") && ["export function bindClickInteractions", "export function dispatchClickInteraction", "export const clickHandlers"].every((needle) => appSource.includes(needle))],
  ["workbench controller assembly split by controller role", ["./workbench/render-loop.js", "./workbench/route-sync.js", "./workbench/location.js", "./workbench/commands.js"].every((path) => appWorkbenchControllerEntrySource.includes(path)) && !appWorkbenchControllerEntrySource.includes("function renderWorkbench") && !appWorkbenchControllerEntrySource.includes("function recordCommand") && !appWorkbenchControllerEntrySource.includes("function applyCommandRoute") && ["export function createWorkbenchRenderLoop", "export function createWorkbenchRouteSync", "export function createWorkbenchLocationHandler", "export function createWorkbenchCommandHandler"].every((needle) => appSource.includes(needle))],
  ["controller activation split by activation role", appActivationEntrySource.includes("./activation/factory.js") && !appActivationEntrySource.includes("function activateModule") && !appActivationEntrySource.includes("applyPanelRoute") && ["export function createActivationHandlers", "export function createModuleActivation", "export function createPanelActivation", "export function createPanelReset"].every((needle) => appSource.includes(needle))],
  ["controller command-application split by command effect role", appCommandApplicationEntrySource.includes("./command-application/apply.js") && !appCommandApplicationEntrySource.includes("activateModule") && !appCommandApplicationEntrySource.includes("recordCommand") && ["export function applyCommandRouteForTarget", "export function applyModuleCommandRoute", "export function applyPanelCommandRoute", "export function recordPlainCommandRoute", "export function commandRouteStatusMessage"].every((needle) => appSource.includes(needle))],
  ["controller command-routing split by route role", appCommandRoutingEntrySource.includes("./command-routing/resolve.js") && !appCommandRoutingEntrySource.includes("normalizeActionId") && !appCommandRoutingEntrySource.includes("moduleById") && !appCommandRoutingEntrySource.includes("routeForCommand") && ["export function commandRouteForTarget", "export function explicitRouteForTarget", "export function fallbackCommandRoute", "export function explicitRouteLabel"].every((needle) => appSource.includes(needle))],
  ["controller location-state split by route-state role", appLocationStateEntrySource.includes("./location-state/apply.js") && !appLocationStateEntrySource.includes("moduleById") && !appLocationStateEntrySource.includes("commandIdFromLocation") && ["export function applyLocationModuleState", "export function locationStateRequest", "export function applyLocationModuleChange", "export function applyLocationPanelTarget", "export function locationStatusMessage"].every((needle) => appSource.includes(needle))],
  ["action interactions split by action route role", appActionInteractionsEntrySource.includes("./actions/handle.js") && !appActionInteractionsEntrySource.includes("commandLabel") && !appActionInteractionsEntrySource.includes("classList") && ["export function handleActionClick", "export function actionClickTarget", "export function activateActionGroupState", "export function recordActionFallbackFeedback", "export function closeActionPopupLayer"].every((needle) => appSource.includes(needle)) && appSource.includes('actionPath("workbench.action"')],
  ["dropdown interactions split by popup role", ["./dropdowns/trigger.js", "./dropdowns/dismissal.js"].every((path) => appDropdownInteractionsEntrySource.includes(path)) && !appDropdownInteractionsEntrySource.includes("actionPath") && !appDropdownInteractionsEntrySource.includes("getBoundingClientRect") && !appDropdownInteractionsEntrySource.includes("classList") && ["export function handleDropdownClick", "export function handlePopupDismissal", "export function dropdownTriggerTarget", "export function positionDropdownPopup", "export function toggleDropdownPopupState", "export function closeDropdownPopupState", "export function recordDropdownFeedback"].every((needle) => appSource.includes(needle))],
  ["generic command interactions split by command fallback role", appGenericInteractionsEntrySource.includes("./generic/handle.js") && !appGenericInteractionsEntrySource.includes("actionPath") && !appGenericInteractionsEntrySource.includes("commandLabel") && ["export function handleGenericCommandClick", "export function genericCommandTarget", "export function recordGenericCommandFeedback"].every((needle) => appSource.includes(needle)) && appSource.includes('actionPath("workbench.command"')],
  ["module navigation interactions split by module route role", appNavigationInteractionsEntrySource.includes("./navigation/handle.js") && !appNavigationInteractionsEntrySource.includes("activateModule(") && ["export function handleModuleNavigation", "export function moduleNavigationTarget", "export function activateModuleNavigation"].every((needle) => appSource.includes(needle))],
  ["field interactions split by input feedback role", appFieldInteractionsEntrySource.includes("./fields/bind.js") && !appFieldInteractionsEntrySource.includes("addEventListener") && !appFieldInteractionsEntrySource.includes("actionPath") && ["export function bindFieldInteractions", "export function editableFieldTarget", "export function handleFieldFocus", "export function handleFieldInput"].every((needle) => appSource.includes(needle))],
  ["keyboard interactions split by activation role", appKeyboardInteractionsEntrySource.includes("./keyboard/bind.js") && !appKeyboardInteractionsEntrySource.includes("addEventListener") && !appKeyboardInteractionsEntrySource.includes("closest") && ["export function bindKeyboardActivation", "export function isKeyboardActivationEvent", "export function keyboardActivationTarget", "export function activateKeyboardTarget"].every((needle) => appSource.includes(needle))],
  ["history interactions split by browser event role", appHistoryInteractionsEntrySource.includes("./history/bind.js") && !appHistoryInteractionsEntrySource.includes("addEventListener") && ["export function bindHistoryInteractions", "export const historyInteractionEvents"].every((needle) => appSource.includes(needle))],
  ["row interactions split by data row role", ["./rows/tree.js", "./rows/data.js"].every((path) => appRowInteractionsEntrySource.includes(path)) && !appRowInteractionsEntrySource.includes("actionPath") && !appRowInteractionsEntrySource.includes("querySelectorAll") && ["export function handleTreeRowClick", "export function handleDataRowClick", "export function selectExclusiveRows", "export function applyTreeRowFallback", "export function applyDataRowFallback"].every((needle) => appSource.includes(needle))],
  ["selection interactions split by selection control role", ["./selection/toggle.js", "./selection/radio.js"].every((path) => appSelectionInteractionsEntrySource.includes(path)) && !appSelectionInteractionsEntrySource.includes("actionPath") && !appSelectionInteractionsEntrySource.includes("classList") && ["export function handleToggleClick", "export function handleRadioClick", "export function selectionControlTarget", "export function applyToggleSelectionState", "export function applyRadioSelectionState", "export function recordToggleFeedback", "export function recordRadioFeedback"].every((needle) => appSource.includes(needle))],
  ["tab interactions split by tab route role", appTabInteractionsEntrySource.includes("./tabs/handle.js") && !appTabInteractionsEntrySource.includes("actionPath") && !appTabInteractionsEntrySource.includes("classList") && ["export function handleTabClick", "export function tabClickTarget", "export function activateTabState", "export function applyPanelTabRoute", "export function recordPlainTabFeedback", "export function setTabStatus"].every((needle) => appSource.includes(needle))],
  ["toolbar interactions split by toolbar role", ["./toolbar/rail.js", "./toolbar/tool.js"].every((path) => appToolbarInteractionsEntrySource.includes(path)) && !appToolbarInteractionsEntrySource.includes("actionPath") && !appToolbarInteractionsEntrySource.includes("classList") && ["export function handleRailClick", "export function handleToolClick", "export function toolbarButtonTarget", "export function activateToolbarButtonState", "export function recordRailToolbarFeedback", "export function recordToolToolbarFeedback"].every((needle) => appSource.includes(needle))],
  ["module navigation handler", appSource.includes("[data-module]") && appSource.includes("moduleWorkspace(moduleId)")],
  ["module deep link handler", appSource.includes("moduleIdFromLocation") && appSource.includes("syncModuleHash") && appSource.includes("historyInteractionEvents") && appSource.includes('"popstate"') && appSource.includes("window.addEventListener(eventName") && responsiveSource.includes("assertDeepLink")],
  ["panel deep link handler", appSource.includes("requestedPanelTargetFromLocation") && appSource.includes("activatePanelTarget") && appSource.includes("routeHash") && responsiveSource.includes("assertPanelDeepLinks") && responsiveSource.includes("assertCommandPanelHistory")],
  ["action hash state handler", appSource.includes("commandIdFromLocation") && appSource.includes("recordCommand") && appSource.includes("latestCommandId") && appSource.includes('params.set("action"') && responsiveSource.includes("assertCommandState") && responsiveSource.includes("hashAction")],
  ["action feedback handler", appSource.includes("[data-action]") && appSource.includes("setStatus") && appSource.includes("zr-action-flash")],
  ["setting-row command labels", appSource.includes('target.closest(".zr-module-setting")?.querySelector("span")?.textContent.trim()')],
  ["keyboard activation handler", appSource.includes('addEventListener("keydown"') && appSource.includes('button[data-action]') && appSource.includes('[role="button"]:not(button)') && responsiveSource.includes("assertKeyboardActivation")],
  ["field feedback handler", appSource.includes('addEventListener("focusin"') && appSource.includes('addEventListener("input"') && appSource.includes("Focused:") && appSource.includes("Edited:")],
  ["response counter hook", appSource.includes("dataset.zrResponseCount") && appSource.includes("dataset.zrLastResponse")],
  ["command route layer", appSource.includes("routeForCommand") && routesSource.includes("moduleRouteMap") && routesSource.includes("panelRouteMap")],
  ["routing command and panel layers split by role", ["./commands/route-for-command.js", "./panels/activation.js"].every((path) => routesEntrySource.includes(path)) && !routesEntrySource.includes("moduleRouteMap") && !routesEntrySource.includes("function activateTab") && ["commands/module-targets.js", "commands/scoped-targets.js", "commands/panel-targets.js", "commands/extension-targets.js", "commands/labels.js", "commands/route-for-command.js", "panels/activation.js"].every((path) => routingRolePaths.some((rolePath) => rolePath.includes(path))) && ["export const moduleRouteMap", "export const moduleScopedRouteMap", "export const panelRouteMap", "export function extensionRouteForCommand", "export function routeForCommand", "export function applyPanelRoute"].every((needle) => routesSource.includes(needle))],
  ["module-scoped command route layer", routesSource.includes("moduleScopedRouteMap") && routesSource.includes('"material:compile"') && routesSource.includes('"behavior-tree:validate"') && responsiveSource.includes("assertModuleScopedCommandRoutes")],
  ["extension command route layer", routesSource.includes("extensionRouteForCommand") && routesSource.includes("extensionPanelKeyForCommand") && routesSource.includes('"handoff"') && responsiveSource.includes("assertExtensionCommandRoutes") && responsiveSource.includes("module-bottom-shader-editor:validation") && responsiveSource.includes("module-bottom-shader-editor:handoff")],
  ["explicit extension internal panel routes", appSource.includes("explicitRouteForTarget") && appSource.includes("dataset.routePanel") && moduleComponentsSource.includes("function routeAttrs") && extensionSurfacesSource.includes("extensionRouteOptions") && extensionHandoffSource.includes("handoffRouteOptions") && ["workbench.extension.tool", "workbench.extension.asset", "workbench.extension.reference", "workbench.extension.output", "workbench.extension.validation", "workbench.extension.references", "workbench.extension.graph", "workbench.extension.detail"].every((scope) => extensionSurfacesSource.includes(scope)) && extensionHandoffSource.includes("workbench.extension.handoff") && extensionModules.every((module) => {
    const workspace = moduleWorkspace(module.id);
    return ["output", "validation", "references", "handoff"].every((panel) => workspace.includes(`data-route-panel="module-bottom-${module.id}:${panel}"`)) && workspace.includes(`data-route-panel="${module.id}-right:`);
  })],
  ["all top-level toolbar command route audit", responsiveSource.includes("assertAllTopLevelToolbarCommandRoutes") && responsiveSource.includes("allTopLevelToolbarCommandRoutesExpression") && responsiveSource.includes("expectedTopLevelModuleTabs") && responsiveSource.includes("active/hash module mismatch") && responsiveSource.includes("active/hash panel mismatch")],
  ["all extension toolbar command route audit", responsiveSource.includes("assertAllExtensionToolbarCommandRoutes") && responsiveSource.includes("allExtensionToolbarCommandRoutesExpression") && responsiveSource.includes("expectedExtensionCards") && responsiveSource.includes("module-bottom-editor-library:routing-log") && responsiveSource.includes("extensionPanelKeyForToolbarCommand")],
  ["all extension internal control audit", responsiveSource.includes("all extension primary surface controls") && responsiveSource.includes("all extension right panel controls") && responsiveSource.includes("all extension bottom panel controls") && responsiveSource.includes("interactionRouteWrites") && responsiveSource.includes("capturedHistoryStates") && !responsiveSource.includes("representativeExtensionIds")],
  ["dedicated per-control route audit", controlRouteSource.includes("control route audit") && controlRouteSource.includes("auditIndexedControls") && controlRouteSource.includes("routeWrites") && controlRouteSource.includes("no route-state write after")],
  ["dedicated explicit panel-route browser audit", controlRouteSource.includes("explicitRouteControlCount") && controlRouteSource.includes("activePanelMatches") && controlRouteSource.includes("dataset?.routePanel") && controlRouteSource.includes("explicit route panel mismatch after") && controlRouteSource.includes("validated ${state.explicitRouteControls} explicit panel-route controls")],
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
  ["module css has responsive module regions", moduleCssCombinedSource.includes(".zr-module-main") && moduleCssCombinedSource.includes("@media (max-width: 720px)")],
  ["module css has no obvious malformed color tuples", !/rgba\(\s*,|,\s*,|,,|\+\}/.test(moduleCssCombinedSource)]
];

const failed = checks.filter(([, passed]) => !passed);

for (const [name, passed] of checks) {
  console.log(`${passed ? "ok" : "fail"} ${name}`);
}

if (failed.length > 0) {
  console.error(`Interaction contract failed: ${failed.map(([name]) => name).join(", ")}`);
  process.exit(1);
}
