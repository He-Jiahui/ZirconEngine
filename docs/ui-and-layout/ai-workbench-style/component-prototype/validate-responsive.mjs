import { spawn } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { extensionModules, webModuleTabs } from "./src/modules/modules.js";
import { actionRouteKey } from "./src/foundation/action-paths.js";

const edgeCandidates = [
  "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
  "C:/Program Files/Microsoft/Edge/Application/msedge.exe",
];
const edge = edgeCandidates.find((candidate) => existsSync(candidate));
if (!edge) {
  throw new Error("Microsoft Edge executable not found.");
}

const here = dirname(fileURLToPath(import.meta.url));
const referenceUrl = pathToFileURL(resolve(here, "index.html")).href;
const port = Number.parseInt(process.env.ZIRCON_WORKBENCH_RESPONSIVE_CDP_PORT ?? String(11980 + Math.floor(Math.random() * 500)), 10);
const profile = resolve(tmpdir(), `zircon-workbench-responsive-cdp-${process.pid}-${Date.now()}`);
const expectedExtensionCards = extensionModules.length;
const expectedTopLevelModuleTabs = webModuleTabs.length;
const responsiveInteractionTimeoutMs = Number.parseInt(process.env.ZIRCON_WORKBENCH_RESPONSIVE_INTERACTION_TIMEOUT_MS ?? "480000", 10);
mkdirSync(profile, { recursive: true });
const browser = spawn(
  edge,
  [
    "--headless=new",
    "--disable-gpu",
    "--hide-scrollbars",
    "--allow-file-access-from-files",
    "--edge-skip-compat-layer-relaunch",
    `--remote-debugging-port=${port}`,
    `--user-data-dir=${profile}`,
    "about:blank",
  ],
  { stdio: "ignore" },
);

const staticViewports = [
  ["reference", 1672, 941],
  ["wide", 1440, 900],
  ["desktop", 1200, 820],
  ["compact", 1040, 760],
  ["narrow", 720, 760],
  ["minimum", 640, 720],
];

const resizeSequence = [
  ["resize-reference", 1672, 941],
  ["resize-desktop", 1200, 820],
  ["resize-compact", 1040, 760],
  ["resize-narrow", 720, 760],
  ["resize-minimum", 640, 720],
  ["resize-return", 1360, 860],
];

let nextId = 1;

try {
  validateSourcePolicy();
  const list = await waitForJson(`http://127.0.0.1:${port}/json/list`);
  const target = list.find((item) => item.type === "page") ?? list[0];
  const cdp = await connect(target.webSocketDebuggerUrl);
  await cdp.send("Page.enable");
  await cdp.send("Runtime.enable");

  const failures = [];
  for (const [name, width, height] of staticViewports) {
    await cdp.send("Emulation.setDeviceMetricsOverride", {
      width,
      height,
      deviceScaleFactor: 1,
      mobile: false,
    });
    await cdp.send("Page.navigate", { url: referenceUrl });
    await waitForWorkbench(cdp);
    const state = JSON.parse(await evaluate(cdp, auditExpression(width, height)));
    if (!state.ok) {
      failures.push(`${name} ${width}x${height}: ${state.failures.join("; ")}`);
    }
  }

  await cdp.send("Emulation.setDeviceMetricsOverride", {
    width: 1672,
    height: 941,
    deviceScaleFactor: 1,
    mobile: false,
  });
  await assertModuleClickHistory(cdp);
  await assertDeepLink(cdp, "shader-editor", "shader-editor");
  await assertDeepLink(cdp, "weather-editor", "weather-editor");
  await assertDeepLink(cdp, "missing-module", "gameplay-effect");
  await assertPanelDeepLinks(cdp);
  await assertCommandPanelHistory(cdp);
  await assertModuleScopedCommandRoutes(cdp);
  await assertAllTopLevelToolbarCommandRoutes(cdp);
  await assertExtensionCommandRoutes(cdp);
  await assertAllExtensionToolbarCommandRoutes(cdp);
  await assertCommandState(cdp);
  await assertKeyboardActivation(cdp);
  await assertCollectionRowButtons(cdp);
  await assertCollectionTreeRows(cdp);
  await assertPopupMenuSelection(cdp);
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
  const interactionState = JSON.parse(await evaluate(cdp, interactionAuditExpression()));
  if (!interactionState.ok) {
    failures.push(`interaction audit: ${interactionState.failures.join("; ")}`);
  }
  for (const [name, width, height] of resizeSequence) {
    await cdp.send("Emulation.setDeviceMetricsOverride", {
      width,
      height,
      deviceScaleFactor: 1,
      mobile: false,
    });
    await delay(120);
    await waitForWorkbench(cdp);
    const state = JSON.parse(await evaluate(cdp, auditExpression(width, height)));
    if (!state.ok) {
      failures.push(`${name} ${width}x${height}: ${state.failures.join("; ")}`);
    }
  }

  cdp.close();
  if (failures.length > 0) {
    throw new Error(`Workbench component responsive audit failed:\n${failures.join("\n")}`);
  }
  console.log(`validated workbench component prototype across ${staticViewports.length} responsive viewports`);
  console.log(`validated workbench component prototype through ${resizeSequence.length} live resize steps`);
  console.log("validated bottom-up component runtime has no full-reference screenshot dependency");
} finally {
  await cleanup();
}

function validateSourcePolicy() {
  const html = readFileSync(resolve(here, "index.html"), "utf8");
  for (const required of [
    "src/foundation/tokens.css",
    "src/foundation/layout.css",
    "src/components/inputs/atoms.css",
    "src/components/inputs/buttons.css",
    "src/components/inputs/fields.css",
    "src/components/inputs/selection-controls.css",
    "src/components/inputs/tabs.css",
    "src/components/inputs/dropdowns.css",
    "src/components/inputs/sliders.css",
    "src/components/data/collections.css",
    "src/components/overlays/menu.css",
    "src/components/feedback/alerts.css",
    "src/components/feedback/toast.css",
    "src/components/feedback/tooltip.css",
    "src/components/surfaces/surfaces.css",
    "src/components/surfaces/viewport.css",
    "src/components/surfaces/inspector.css",
    "src/components/surfaces/showcase.css",
    "src/components/surfaces/status.css",
    "src/modules/modules.css",
    "src/modules/module-layouts.css",
    "src/modules/extension-library.css",
    "src/modules/module-data.css",
    "src/modules/module-graphs.css",
    "src/modules/module-output.css",
    "src/modules/module-canvases.css",
    "src/modules/module-feedback.css",
    "src/modules/module-responsive.css",
    "src/workbench/workbench.css",
    "src/workbench/showcase-base.css",
    "src/workbench/inspector-detail.css",
    "src/workbench/showcase-controls.css",
    "src/workbench/side-panels.css",
    "src/workbench/statusbar-tuning.css",
    "src/foundation/responsive.css",
    "app.js",
  ]) {
    if (!html.includes(required)) {
      throw new Error(`index.html must load ${required}.`);
    }
  }
  if (/https?:\/\//i.test(html)) {
    throw new Error("index.html must not load external resources.");
  }
  const foundationTokenCss = readFileSync(resolve(here, "src/foundation/tokens.css"), "utf8");
  const foundationTokenCssRoles = [
    ["./tokens/dimensions.css", "src/foundation/tokens/dimensions.css", "--ref-w"],
    ["./tokens/typography.css", "src/foundation/tokens/typography.css", "--font"],
    ["./tokens/palette.css", "src/foundation/tokens/palette.css", "--accent"],
    ["./tokens/effects.css", "src/foundation/tokens/effects.css", "--shadow"],
    ["./tokens/shape-controls.css", "src/foundation/tokens/shape-controls.css", "--control-h"],
    ["./tokens/gaps.css", "src/foundation/tokens/gaps.css", "--gap-4"],
    ["./tokens/base.css", "src/foundation/tokens/base.css", ".zr-app"],
  ];
  for (const [importPath, rolePath, selector] of foundationTokenCssRoles) {
    if (!foundationTokenCss.includes(importPath)) {
      throw new Error(`foundation tokens.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (foundationTokenCss.includes(":root") || foundationTokenCss.includes(".zr-")) {
    throw new Error("foundation tokens.css must remain an import-only token style entry.");
  }
  const foundationResponsiveCss = readFileSync(resolve(here, "src/foundation/responsive.css"), "utf8");
  const foundationResponsiveCssRoles = [
    ["./responsive/wide-shell.css", "src/foundation/responsive/wide-shell.css", ".zr-topbar"],
    ["./responsive/wide-panels.css", "src/foundation/responsive/wide-panels.css", ".zr-showcase-grid"],
    ["./responsive/tablet-shell.css", "src/foundation/responsive/tablet-shell.css", ".zr-window"],
    ["./responsive/tablet-panels.css", "src/foundation/responsive/tablet-panels.css", ".zr-inspector"],
    ["./responsive/mobile-shell.css", "src/foundation/responsive/mobile-shell.css", ".zr-statusbar"],
    ["./responsive/mobile-panels.css", "src/foundation/responsive/mobile-panels.css", ".zr-viewport"],
    ["./responsive/compact-controls.css", "src/foundation/responsive/compact-controls.css", ".zr-viewport-cluster:first-child .zr-select"],
  ];
  for (const [importPath, rolePath, selector] of foundationResponsiveCssRoles) {
    if (!foundationResponsiveCss.includes(importPath)) {
      throw new Error(`foundation responsive.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (foundationResponsiveCss.includes(".zr-")) {
    throw new Error("foundation responsive.css must remain an import-only global responsive style entry.");
  }
  const collectionsCss = readFileSync(resolve(here, "src/components/data/collections.css"), "utf8");
  const collectionCssRoles = [
    ["./collections/panel-group.css", "src/components/data/collections/panel-group.css", ".zr-panel-tabs"],
    ["./collections/tree-view.css", "src/components/data/collections/tree-view.css", ".zr-tree-row"],
    ["./collections/table-view.css", "src/components/data/collections/table-view.css", ".zr-table-row"],
    ["./collections/list-view.css", "src/components/data/collections/list-view.css", ".zr-list-item"],
  ];
  for (const [importPath, rolePath, selector] of collectionCssRoles) {
    if (!collectionsCss.includes(importPath)) {
      throw new Error(`collections.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (collectionsCss.includes(".zr-")) {
    throw new Error("collections.css must remain an import-only collection style entry.");
  }
  const surfacesCss = readFileSync(resolve(here, "src/components/surfaces/surfaces.css"), "utf8");
  const surfaceCssRoles = [
    ["./shell/window.css", "src/components/surfaces/shell/window.css", ".zr-window"],
    ["./shell/topbar.css", "src/components/surfaces/shell/topbar.css", ".zr-topbar"],
    ["./shell/rail.css", "src/components/surfaces/shell/rail.css", ".zr-rail"],
    ["./panels/base.css", "src/components/surfaces/panels/base.css", ".zr-panel"],
    ["./panels/scene.css", "src/components/surfaces/panels/scene.css", ".zr-scene-panel"],
  ];
  for (const [importPath, rolePath, selector] of surfaceCssRoles) {
    if (!surfacesCss.includes(importPath)) {
      throw new Error(`surfaces.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (surfacesCss.includes(".zr-")) {
    throw new Error("surfaces.css must remain an import-only surface shell/panel style entry.");
  }
  const inspectorCss = readFileSync(resolve(here, "src/components/surfaces/inspector.css"), "utf8");
  const inspectorCssRoles = [
    ["./panels/inspector/layout.css", "src/components/surfaces/panels/inspector/layout.css", ".zr-inspector"],
    ["./panels/inspector/object-header.css", "src/components/surfaces/panels/inspector/object-header.css", ".zr-object-header"],
    ["./panels/inspector/sections.css", "src/components/surfaces/panels/inspector/sections.css", ".zr-section-title"],
    ["./panels/inspector/fields.css", "src/components/surfaces/panels/inspector/fields.css", ".zr-form-row"],
    ["./panels/inspector/resources.css", "src/components/surfaces/panels/inspector/resources.css", ".zr-resource-row"],
  ];
  for (const [importPath, rolePath, selector] of inspectorCssRoles) {
    if (!inspectorCss.includes(importPath)) {
      throw new Error(`inspector.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (inspectorCss.includes(".zr-")) {
    throw new Error("inspector.css must remain an import-only inspector surface style entry.");
  }
  const showcaseCss = readFileSync(resolve(here, "src/components/surfaces/showcase.css"), "utf8");
  const showcaseCssRoles = [
    ["./panels/showcase/layout.css", "src/components/surfaces/panels/showcase/layout.css", ".zr-showcase"],
    ["./panels/showcase/grid.css", "src/components/surfaces/panels/showcase/grid.css", ".zr-showcase-grid"],
    ["./panels/showcase/columns.css", "src/components/surfaces/panels/showcase/columns.css", ".zr-showcase-col"],
    ["./panels/showcase/stacks.css", "src/components/surfaces/panels/showcase/stacks.css", ".zr-side-stack"],
  ];
  for (const [importPath, rolePath, selector] of showcaseCssRoles) {
    if (!showcaseCss.includes(importPath)) {
      throw new Error(`showcase.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (showcaseCss.includes(".zr-")) {
    throw new Error("showcase.css must remain an import-only showcase surface style entry.");
  }
  const statusCss = readFileSync(resolve(here, "src/components/surfaces/status.css"), "utf8");
  const statusCssRoles = [
    ["./status/bar.css", "src/components/surfaces/status/bar.css", ".zr-statusbar"],
    ["./status/groups.css", "src/components/surfaces/status/groups.css", ".zr-status-left"],
    ["./status/controls.css", "src/components/surfaces/status/controls.css", ".zr-statusbar .zr-select"],
    ["./status/indicators.css", "src/components/surfaces/status/indicators.css", ".zr-dot"],
  ];
  for (const [importPath, rolePath, selector] of statusCssRoles) {
    if (!statusCss.includes(importPath)) {
      throw new Error(`status.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (statusCss.includes(".zr-")) {
    throw new Error("status.css must remain an import-only statusbar surface style entry.");
  }
  const viewportCss = readFileSync(resolve(here, "src/components/surfaces/viewport.css"), "utf8");
  const viewportCssRoles = [
    ["./viewport/base.css", ".zr-scene-shell"],
    ["./viewport/lighting.css", "./lighting/lightwash.css"],
    ["./viewport/structure.css", "./structure/wall.css"],
    ["./viewport/floor.css", "./floor/base.css"],
    ["./viewport/props.css", "./props/cargo.css"],
    ["./viewport/tools.css", "./tools/axis-mini.css"],
  ];
  for (const [importPath, selector] of viewportCssRoles) {
    if (!viewportCss.includes(importPath)) {
      throw new Error(`viewport.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, `src/components/surfaces/${importPath.replace("./", "")}`), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${importPath} must own ${selector}.`);
    }
  }
  if (viewportCss.includes(".zr-")) {
    throw new Error("viewport.css must remain an import-only surface style entry.");
  }
  const viewportLightingCss = readFileSync(resolve(here, "src/components/surfaces/viewport/lighting.css"), "utf8");
  const viewportLightingCssRoles = [
    ["./lighting/lightwash.css", "src/components/surfaces/viewport/lighting/lightwash.css", ".zr-scene-lightwash"],
    ["./lighting/shadows.css", "src/components/surfaces/viewport/lighting/shadows.css", ".zr-scene-shadow"],
  ];
  for (const [importPath, rolePath, selector] of viewportLightingCssRoles) {
    if (!viewportLightingCss.includes(importPath)) {
      throw new Error(`lighting.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (viewportLightingCss.includes(".zr-")) {
    throw new Error("lighting.css must remain an import-only viewport lighting style entry.");
  }
  const viewportFloorCss = readFileSync(resolve(here, "src/components/surfaces/viewport/floor.css"), "utf8");
  const viewportFloorCssRoles = [
    ["./floor/base.css", "src/components/surfaces/viewport/floor/base.css", ".zr-scene-floor"],
    ["./floor/grid.css", "src/components/surfaces/viewport/floor/grid.css", ".zr-viewport-grid-line"],
    ["./floor/reflections.css", "src/components/surfaces/viewport/floor/reflections.css", ".zr-floor-reflection"],
    ["./floor/grates.css", "src/components/surfaces/viewport/floor/grates.css", ".zr-floor-grate"],
    ["./floor/panels.css", "src/components/surfaces/viewport/floor/panels.css", ".zr-floor-panel"],
    ["./floor/seams.css", "src/components/surfaces/viewport/floor/seams.css", ".zr-floor-seam"],
  ];
  for (const [importPath, rolePath, selector] of viewportFloorCssRoles) {
    if (!viewportFloorCss.includes(importPath)) {
      throw new Error(`floor.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (viewportFloorCss.includes(".zr-")) {
    throw new Error("floor.css must remain an import-only viewport floor style entry.");
  }
  const viewportStructureCss = readFileSync(resolve(here, "src/components/surfaces/viewport/structure.css"), "utf8");
  const viewportStructureCssRoles = [
    ["./structure/wall.css", "src/components/surfaces/viewport/structure/wall.css", ".zr-scene-wall"],
    ["./structure/ceiling-door.css", "src/components/surfaces/viewport/structure/ceiling-door.css", ".zr-scene-door"],
    ["./structure/fixtures.css", "src/components/surfaces/viewport/structure/fixtures.css", ".zr-scene-wall-panel"],
    ["./structure/side-walls.css", "src/components/surfaces/viewport/structure/side-walls.css", ".zr-scene-side"],
    ["./structure/rails.css", "src/components/surfaces/viewport/structure/rails.css", ".zr-scene-handrail"],
  ];
  for (const [importPath, rolePath, selector] of viewportStructureCssRoles) {
    if (!viewportStructureCss.includes(importPath)) {
      throw new Error(`structure.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (viewportStructureCss.includes(".zr-")) {
    throw new Error("structure.css must remain an import-only viewport structure style entry.");
  }
  const viewportPropsCss = readFileSync(resolve(here, "src/components/surfaces/viewport/props.css"), "utf8");
  const viewportPropsCssRoles = [
    ["./props/cargo.css", "src/components/surfaces/viewport/props/cargo.css", ".zr-scene-cargo"],
    ["./props/crate.css", "src/components/surfaces/viewport/props/crate.css", ".zr-crate"],
    ["./props/selection.css", "src/components/surfaces/viewport/props/selection.css", ".zr-selection-edge"],
    ["./props/transform.css", "src/components/surfaces/viewport/props/transform.css", ".zr-transform-axis"],
  ];
  for (const [importPath, rolePath, selector] of viewportPropsCssRoles) {
    if (!viewportPropsCss.includes(importPath)) {
      throw new Error(`props.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (viewportPropsCss.includes(".zr-")) {
    throw new Error("props.css must remain an import-only viewport props style entry.");
  }
  const viewportToolsCss = readFileSync(resolve(here, "src/components/surfaces/viewport/tools.css"), "utf8");
  const viewportToolsCssRoles = [
    ["./tools/axis-mini.css", "src/components/surfaces/viewport/tools/axis-mini.css", ".zr-axis-mini"],
    ["./tools/orientation-gizmo.css", "src/components/surfaces/viewport/tools/orientation-gizmo.css", ".zr-orientation-gizmo"],
    ["./tools/vignette.css", "src/components/surfaces/viewport/tools/vignette.css", ".zr-scene-vignette"],
    ["./tools/toolbar.css", "src/components/surfaces/viewport/tools/toolbar.css", ".zr-viewport-tools"],
  ];
  for (const [importPath, rolePath, selector] of viewportToolsCssRoles) {
    if (!viewportToolsCss.includes(importPath)) {
      throw new Error(`tools.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (viewportToolsCss.includes(".zr-")) {
    throw new Error("tools.css must remain an import-only viewport tools style entry.");
  }
  const moduleCanvasCss = readFileSync(resolve(here, "src/modules/module-canvases.css"), "utf8");
  const moduleMapCss = readFileSync(resolve(here, "src/modules/canvases/map.css"), "utf8");
  const moduleHudCss = readFileSync(resolve(here, "src/modules/canvases/hud.css"), "utf8");
  const moduleFeedbackCss = readFileSync(resolve(here, "src/modules/module-feedback.css"), "utf8");
  const moduleCssRoles = [
    [moduleCanvasCss, "./canvases/map.css", "src/modules/canvases/map/map.css", "./map/base.css"],
    [moduleCanvasCss, "./canvases/hud.css", "src/modules/canvases/hud/hud.css", "./hud/base.css"],
    [moduleFeedbackCss, "./feedback/inline-status.css", "src/modules/feedback/inline-status.css", ".zr-action-flash"],
  ];
  for (const [entrySource, importPath, rolePath, selector] of moduleCssRoles) {
    if (!entrySource.includes(importPath)) {
      throw new Error(`module CSS entry must import ${importPath}.`);
    }
    if (selector.startsWith("./")) {
      if (!readFileSync(resolve(here, importPath.replace("./", "src/modules/")), "utf8").includes(selector)) {
        throw new Error(`${importPath} must import ${selector}.`);
      }
    } else {
      const roleSource = readFileSync(resolve(here, rolePath), "utf8");
      if (!roleSource.includes(selector)) {
        throw new Error(`${rolePath} must own ${selector}.`);
      }
    }
  }
  const moduleMapCssRoles = [
    ["./map/base.css", "src/modules/canvases/map/base.css", ".zr-module-map"],
    ["./map/walls.css", "src/modules/canvases/map/walls.css", ".zr-map-wall"],
    ["./map/points.css", "src/modules/canvases/map/points.css", ".zr-map-point"],
    ["./map/cones.css", "src/modules/canvases/map/cones.css", ".zr-map-cone"],
    ["./map/paths.css", "src/modules/canvases/map/paths.css", ".zr-map-path"],
  ];
  const moduleHudCssRoles = [
    ["./hud/base.css", "src/modules/canvases/hud/base.css", ".zr-module-hud-canvas"],
    ["./hud/widgets.css", "src/modules/canvases/hud/widgets.css", ".zr-hud-widget"],
    ["./hud/positions.css", "src/modules/canvases/hud/positions.css", ".zr-hud-widget.is-minimap"],
    ["./hud/status.css", "src/modules/canvases/hud/status.css", ".zr-hud-widget.is-status"],
    ["./hud/crosshair.css", "src/modules/canvases/hud/crosshair.css", ".zr-hud-crosshair"],
  ];
  for (const [importPath, rolePath, selector] of moduleMapCssRoles) {
    if (!moduleMapCss.includes(importPath)) {
      throw new Error(`canvases/map.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  for (const [importPath, rolePath, selector] of moduleHudCssRoles) {
    if (!moduleHudCss.includes(importPath)) {
      throw new Error(`canvases/hud.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (moduleCanvasCss.includes(".zr-") || moduleMapCss.includes(".zr-") || moduleHudCss.includes(".zr-") || moduleFeedbackCss.includes(".zr-")) {
    throw new Error("module canvas and feedback CSS entries must remain import-only.");
  }
  const moduleOutputCss = readFileSync(resolve(here, "src/modules/module-output.css"), "utf8");
  const moduleOutputCssRoles = [
    ["./output/preview.css", "src/modules/output/preview.css", ".zr-module-preview"],
    ["./output/stats-actions.css", "src/modules/output/stats-actions.css", ".zr-module-stat-grid"],
    ["./output/layout.css", "src/modules/output/layout.css", ".zr-module-output-grid"],
    ["./output/logs.css", "src/modules/output/logs.css", ".zr-module-log"],
    ["./output/asset-strip.css", "src/modules/output/asset-strip.css", ".zr-module-asset-strip"],
    ["./output/timeline.css", "src/modules/output/timeline.css", ".zr-module-timeline"],
  ];
  for (const [importPath, rolePath, selector] of moduleOutputCssRoles) {
    if (!moduleOutputCss.includes(importPath)) {
      throw new Error(`module-output.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (moduleOutputCss.includes(".zr-")) {
    throw new Error("module-output.css must remain an import-only output style entry.");
  }
  const moduleGraphsCss = readFileSync(resolve(here, "src/modules/module-graphs.css"), "utf8");
  const moduleGraphCssRoles = [
    ["./graphs/board.css", "src/modules/graphs/board.css", ".zr-module-graph"],
    ["./graphs/nodes.css", "src/modules/graphs/nodes.css", ".zr-module-node"],
    ["./graphs/links.css", "src/modules/graphs/links.css", ".zr-graph-link"],
    ["./graphs/minimap.css", "src/modules/graphs/minimap.css", ".zr-module-minimap"],
    ["./graphs/curves.css", "src/modules/graphs/curves.css", ".zr-module-curve"],
  ];
  for (const [importPath, rolePath, selector] of moduleGraphCssRoles) {
    if (!moduleGraphsCss.includes(importPath)) {
      throw new Error(`module-graphs.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (moduleGraphsCss.includes(".zr-")) {
    throw new Error("module-graphs.css must remain an import-only graph style entry.");
  }
  const moduleDataCss = readFileSync(resolve(here, "src/modules/module-data.css"), "utf8");
  const moduleDataCssRoles = [
    ["./data/settings.css", "src/modules/data/settings.css", ".zr-module-setting"],
    ["./data/collection-rows.css", "src/modules/data/collection-rows.css", ".zr-module-list-row.is-selected"],
    ["./data/list-rows.css", "src/modules/data/list-rows.css", ".zr-module-list-row"],
    ["./data/tree-rows.css", "src/modules/data/tree-rows.css", ".zr-module-tree-row"],
    ["./data/table-rows.css", "src/modules/data/table-rows.css", ".zr-module-table-row"],
    ["./data/tags.css", "src/modules/data/tags.css", ".zr-module-tag"],
    ["./data/card-tools.css", "src/modules/data/card-tools.css", ".zr-module-card-tools"],
  ];
  for (const [importPath, rolePath, selector] of moduleDataCssRoles) {
    if (!moduleDataCss.includes(importPath)) {
      throw new Error(`module-data.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (moduleDataCss.includes(".zr-")) {
    throw new Error("module-data.css must remain an import-only module data style entry.");
  }
  const modulesCss = readFileSync(resolve(here, "src/modules/modules.css"), "utf8");
  const moduleShellCssRoles = [
    ["./shell/top-tabs.css", "src/modules/shell/top-tabs.css", ".zr-module-tabs"],
    ["./shell/toolbar.css", "src/modules/shell/toolbar.css", ".zr-module-toolbar"],
    ["./shell/regions.css", "src/modules/shell/regions.css", ".zr-module-left"],
    ["./shell/mainbar.css", "src/modules/shell/mainbar.css", ".zr-module-mainbar"],
    ["./shell/panel-tabs.css", "src/modules/shell/panel-tabs.css", ".zr-module-panel-tabs"],
    ["./shell/cards.css", "src/modules/shell/cards.css", ".zr-module-card"],
    ["./shell/forms.css", "src/modules/shell/forms.css", ".zr-module-filterbar .zr-search"],
  ];
  for (const [importPath, rolePath, selector] of moduleShellCssRoles) {
    if (!modulesCss.includes(importPath)) {
      throw new Error(`modules.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (modulesCss.includes(".zr-")) {
    throw new Error("modules.css must remain an import-only module shell style entry.");
  }
  const moduleLayoutsCss = readFileSync(resolve(here, "src/modules/module-layouts.css"), "utf8");
  const moduleLayoutCssRoles = [
    ["./layouts/base.css", "src/modules/layouts/base.css", ".zr-module-editor-grid"],
    ["./layouts/core.css", "src/modules/layouts/core.css", ".zr-module-editor-grid.is-gameplay"],
    ["./layouts/library.css", "src/modules/layouts/library.css", ".zr-module-editor-grid.is-library"],
    ["./layouts/extensions.css", "src/modules/layouts/extensions.css", ".zr-module-editor-grid.is-extension"],
  ];
  for (const [importPath, rolePath, selector] of moduleLayoutCssRoles) {
    if (!moduleLayoutsCss.includes(importPath)) {
      throw new Error(`module-layouts.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (moduleLayoutsCss.includes(".zr-")) {
    throw new Error("module-layouts.css must remain an import-only module layout style entry.");
  }
  const extensionLibraryCss = readFileSync(resolve(here, "src/modules/extension-library.css"), "utf8");
  const extensionLibraryCssRoles = [
    ["./extension-library/card-grid.css", "src/modules/extension-library/card-grid.css", ".zr-extension-card-grid"],
    ["./extension-library/cards.css", "src/modules/extension-library/cards.css", ".zr-extension-card"],
    ["./extension-library/drilldown.css", "src/modules/extension-library/drilldown.css", ".zr-library-drilldown"],
    ["./extension-library/panel-group.css", "src/modules/extension-library/panel-group.css", ".zr-panel-group"],
  ];
  for (const [importPath, rolePath, selector] of extensionLibraryCssRoles) {
    if (!extensionLibraryCss.includes(importPath)) {
      throw new Error(`extension-library.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (extensionLibraryCss.includes(".zr-")) {
    throw new Error("extension-library.css must remain an import-only More Editors library style entry.");
  }
  const moduleResponsiveCss = readFileSync(resolve(here, "src/modules/module-responsive.css"), "utf8");
  const moduleResponsiveCssRoles = [
    ["./responsive/navigation.css", "src/modules/responsive/navigation.css", ".zr-module-tab"],
    ["./responsive/workspace.css", "src/modules/responsive/workspace.css", ".zr-module-editor-grid.is-gameplay"],
    ["./responsive/tablet-shell.css", "src/modules/responsive/tablet-shell.css", ".zr-module-left"],
    ["./responsive/mobile-stack.css", "src/modules/responsive/mobile-stack.css", ".zr-module-tabs"],
  ];
  for (const [importPath, rolePath, selector] of moduleResponsiveCssRoles) {
    if (!moduleResponsiveCss.includes(importPath)) {
      throw new Error(`module-responsive.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (moduleResponsiveCss.includes(".zr-")) {
    throw new Error("module-responsive.css must remain an import-only module responsive style entry.");
  }
  const workbenchCss = readFileSync(resolve(here, "src/workbench/workbench.css"), "utf8");
  const workbenchLowerDemoCssRoles = [
    ["./lower-demo/layout.css", "src/workbench/lower-demo/layout.css", ".zr-lower-demo"],
    ["./lower-demo/alerts.css", "src/workbench/lower-demo/alerts.css", ".zr-alert-stack"],
    ["./lower-demo/table.css", "src/workbench/lower-demo/table.css", ".zr-table .zr-table-row"],
    ["./lower-demo/toast.css", "src/workbench/lower-demo/toast.css", ".zr-toast"],
    ["./lower-demo/effects.css", "src/workbench/lower-demo/effects.css", "filter: blur"],
    ["./lower-demo/tooltip.css", "src/workbench/lower-demo/tooltip.css", ".zr-tooltip-bubble"],
  ];
  for (const [importPath, rolePath, selector] of workbenchLowerDemoCssRoles) {
    if (!workbenchCss.includes(importPath)) {
      throw new Error(`workbench.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (workbenchCss.includes(".zr-")) {
    throw new Error("workbench.css must remain an import-only lower-demo style entry.");
  }
  const showcaseControlCss = readFileSync(resolve(here, "src/workbench/showcase-controls.css"), "utf8");
  const showcaseControlCssRoles = [
    ["./showcase-controls/icon-buttons.css", "src/workbench/showcase-controls/icon-buttons.css", ".zr-icon-button.is-lg"],
    ["./showcase-controls/shared-gaps.css", "src/workbench/showcase-controls/shared-gaps.css", ".zr-field-stack"],
    ["./showcase-controls/button-grid.css", "src/workbench/showcase-controls/button-grid.css", "./button-grid/layout.css"],
    ["./showcase-controls/fields.css", "src/workbench/showcase-controls/fields.css", ".zr-input:focus"],
    ["./showcase-controls/selection-controls.css", "src/workbench/showcase-controls/selection-controls.css", ".zr-checkbox"],
    ["./showcase-controls/segmented-controls.css", "src/workbench/showcase-controls/segmented-controls.css", ".zr-segment"],
    ["./showcase-controls/sliders.css", "src/workbench/showcase-controls/sliders.css", ".zr-slider-track"],
    ["./showcase-controls/tabs.css", "src/workbench/showcase-controls/tabs.css", ".zr-tab.is-active"],
  ];
  for (const [importPath, rolePath, selector] of showcaseControlCssRoles) {
    if (!showcaseControlCss.includes(importPath)) {
      throw new Error(`showcase-controls.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (showcaseControlCss.includes(".zr-")) {
    throw new Error("showcase-controls.css must remain an import-only workbench control style entry.");
  }
  const showcaseButtonGridCss = readFileSync(resolve(here, "src/workbench/showcase-controls/button-grid.css"), "utf8");
  const showcaseButtonGridCssRoles = [
    ["./button-grid/layout.css", "src/workbench/showcase-controls/button-grid/layout.css", ".zr-showcase-col:first-child .zr-control-grid"],
    ["./button-grid/base-controls.css", "src/workbench/showcase-controls/button-grid/base-controls.css", ".zr-showcase-col:first-child .zr-select"],
    ["./button-grid/state-colors.css", "src/workbench/showcase-controls/button-grid/state-colors.css", ".zr-showcase-col:first-child .zr-button:disabled"],
    ["./button-grid/item-overrides.css", "src/workbench/showcase-controls/button-grid/item-overrides.css", ".zr-showcase-col:first-child .zr-control-grid > :nth-child(8) .zr-icon"],
  ];
  for (const [importPath, rolePath, selector] of showcaseButtonGridCssRoles) {
    if (!showcaseButtonGridCss.includes(importPath)) {
      throw new Error(`button-grid.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (showcaseButtonGridCss.includes(".zr-")) {
    throw new Error("button-grid.css must remain an import-only button grid style entry.");
  }
  const inspectorDetailCss = readFileSync(resolve(here, "src/workbench/inspector-detail.css"), "utf8");
  const inspectorDetailCssRoles = [
    ["./inspector-detail/base.css", "src/workbench/inspector-detail/base.css", ".zr-inspector .zr-button"],
    ["./inspector-detail/scene-tree.css", "src/workbench/inspector-detail/scene-tree.css", ".zr-scene-panel .zr-tree"],
    ["./inspector-detail/forms.css", "src/workbench/inspector-detail/forms.css", ".zr-inspector .zr-form-row"],
    ["./inspector-detail/transform-section.css", "src/workbench/inspector-detail/transform-section.css", "./transform-section/section.css"],
    ["./inspector-detail/mesh-renderer-section.css", "src/workbench/inspector-detail/mesh-renderer-section.css", ".zr-section.is-mesh-renderer"],
  ];
  for (const [importPath, rolePath, selector] of inspectorDetailCssRoles) {
    if (!inspectorDetailCss.includes(importPath)) {
      throw new Error(`inspector-detail.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (inspectorDetailCss.includes(".zr-")) {
    throw new Error("inspector-detail.css must remain an import-only inspector detail style entry.");
  }
  const inspectorTransformCss = readFileSync(resolve(here, "src/workbench/inspector-detail/transform-section.css"), "utf8");
  const inspectorTransformCssRoles = [
    ["./transform-section/section.css", "src/workbench/inspector-detail/transform-section/section.css", ".zr-section.is-transform"],
    ["./transform-section/value-boxes.css", "src/workbench/inspector-detail/transform-section/value-boxes.css", ".zr-value-box"],
    ["./transform-section/vector-rows.css", "src/workbench/inspector-detail/transform-section/vector-rows.css", ".zr-vector-row:nth-of-type(2)"],
    ["./transform-section/linked-axis.css", "src/workbench/inspector-detail/transform-section/linked-axis.css", ".zr-linked-axis"],
    ["./transform-section/axis-labels.css", "src/workbench/inspector-detail/transform-section/axis-labels.css", "> span:nth-child(2)"],
    ["./transform-section/controls.css", "src/workbench/inspector-detail/transform-section/controls.css", ".zr-checkbox.is-checked"],
  ];
  for (const [importPath, rolePath, selector] of inspectorTransformCssRoles) {
    if (!inspectorTransformCss.includes(importPath)) {
      throw new Error(`transform-section.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (inspectorTransformCss.includes(".zr-")) {
    throw new Error("transform-section.css must remain an import-only inspector transform style entry.");
  }
  const sidePanelsCss = readFileSync(resolve(here, "src/workbench/side-panels.css"), "utf8");
  const sidePanelsCssRoles = [
    ["./side-panels/menus.css", "src/workbench/side-panels/menus.css", ".zr-side-stack"],
    ["./side-panels/alt-panel.css", "src/workbench/side-panels/alt-panel.css", ".zr-alt-panel"],
    ["./side-panels/layer-history.css", "src/workbench/side-panels/layer-history.css", ".zr-layer-row"],
    ["./side-panels/console.css", "src/workbench/side-panels/console.css", ".zr-console-row"],
    ["./side-panels/inspector-checkboxes.css", "src/workbench/side-panels/inspector-checkboxes.css", ".zr-inspector .zr-checkbox"],
    ["./side-panels/form-overrides.css", "src/workbench/side-panels/form-overrides.css", ".zr-form-row .zr-select"],
    ["./side-panels/topbar-overrides.css", "src/workbench/side-panels/topbar-overrides.css", ".zr-topbar .zr-select:has"],
  ];
  for (const [importPath, rolePath, selector] of sidePanelsCssRoles) {
    if (!sidePanelsCss.includes(importPath)) {
      throw new Error(`side-panels.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (sidePanelsCss.includes(".zr-")) {
    throw new Error("side-panels.css must remain an import-only workbench side panel style entry.");
  }
  const statusbarTuningCss = readFileSync(resolve(here, "src/workbench/statusbar-tuning.css"), "utf8");
  const statusbarTuningCssRoles = [
    ["./statusbar-tuning/popup-layer.css", "src/workbench/statusbar-tuning/popup-layer.css", ".zr-popup-layer"],
    ["./statusbar-tuning/left-group.css", "src/workbench/statusbar-tuning/left-group.css", ".zr-status-left"],
    ["./statusbar-tuning/right-group.css", "src/workbench/statusbar-tuning/right-group.css", ".zr-status-right"],
    ["./statusbar-tuning/frame.css", "src/workbench/statusbar-tuning/frame.css", ".zr-statusbar"],
    ["./statusbar-tuning/controls.css", "src/workbench/statusbar-tuning/controls.css", ".zr-statusbar .zr-select"],
  ];
  for (const [importPath, rolePath, selector] of statusbarTuningCssRoles) {
    if (!statusbarTuningCss.includes(importPath)) {
      throw new Error(`statusbar-tuning.css must import ${importPath}.`);
    }
    const roleSource = readFileSync(resolve(here, rolePath), "utf8");
    if (!roleSource.includes(selector)) {
      throw new Error(`${rolePath} must own ${selector}.`);
    }
  }
  if (statusbarTuningCss.includes(".zr-")) {
    throw new Error("statusbar-tuning.css must remain an import-only workbench statusbar tuning style entry.");
  }
  const sources = [
    "app.js",
    "src/app/mount.js",
    "src/app/controller.js",
    "src/app/controller/activation.js",
    "src/app/controller/activation/factory.js",
    "src/app/controller/activation/module.js",
    "src/app/controller/activation/panel.js",
    "src/app/controller/activation/reset.js",
    "src/app/controller/command-application.js",
    "src/app/controller/command-application/apply.js",
    "src/app/controller/command-application/module.js",
    "src/app/controller/command-application/panel.js",
    "src/app/controller/command-application/record.js",
    "src/app/controller/command-application/status.js",
    "src/app/controller/create-workbench-controller.js",
    "src/app/controller/command-routing.js",
    "src/app/controller/command-routing/explicit.js",
    "src/app/controller/command-routing/fallback.js",
    "src/app/controller/command-routing/label.js",
    "src/app/controller/command-routing/resolve.js",
    "src/app/controller/history.js",
    "src/app/controller/location-state.js",
    "src/app/controller/location-state/apply.js",
    "src/app/controller/location-state/module.js",
    "src/app/controller/location-state/panel.js",
    "src/app/controller/location-state/request.js",
    "src/app/controller/location-state/status.js",
    "src/app/controller/rendering.js",
    "src/app/controller/state.js",
    "src/app/controller/status.js",
    "src/app/controller/workbench/commands.js",
    "src/app/controller/workbench/location.js",
    "src/app/controller/workbench/render-loop.js",
    "src/app/controller/workbench/route-sync.js",
    "src/app/route-state.js",
    "src/app/labels.js",
    "src/app/interactions/click.js",
    "src/app/interactions/click/bind.js",
    "src/app/interactions/click/dispatch.js",
    "src/app/interactions/click/handlers.js",
    "src/app/interactions/click/actions.js",
    "src/app/interactions/click/actions/feedback.js",
    "src/app/interactions/click/actions/group.js",
    "src/app/interactions/click/actions/handle.js",
    "src/app/interactions/click/actions/menu.js",
    "src/app/interactions/click/actions/target.js",
    "src/app/interactions/click/dropdowns.js",
    "src/app/interactions/click/dropdowns/dismissal.js",
    "src/app/interactions/click/dropdowns/feedback.js",
    "src/app/interactions/click/dropdowns/placement.js",
    "src/app/interactions/click/dropdowns/state.js",
    "src/app/interactions/click/dropdowns/target.js",
    "src/app/interactions/click/dropdowns/trigger.js",
    "src/app/interactions/click/generic.js",
    "src/app/interactions/click/generic/feedback.js",
    "src/app/interactions/click/generic/handle.js",
    "src/app/interactions/click/generic/target.js",
    "src/app/interactions/click/navigation.js",
    "src/app/interactions/click/navigation/activate.js",
    "src/app/interactions/click/navigation/handle.js",
    "src/app/interactions/click/navigation/target.js",
    "src/app/interactions/click/rows.js",
    "src/app/interactions/click/rows/data.js",
    "src/app/interactions/click/rows/feedback.js",
    "src/app/interactions/click/rows/selection.js",
    "src/app/interactions/click/rows/tree.js",
    "src/app/interactions/click/selection.js",
    "src/app/interactions/click/selection/feedback.js",
    "src/app/interactions/click/selection/radio.js",
    "src/app/interactions/click/selection/state.js",
    "src/app/interactions/click/selection/target.js",
    "src/app/interactions/click/selection/toggle.js",
    "src/app/interactions/click/tabs.js",
    "src/app/interactions/click/tabs/feedback.js",
    "src/app/interactions/click/tabs/handle.js",
    "src/app/interactions/click/tabs/panel.js",
    "src/app/interactions/click/tabs/state.js",
    "src/app/interactions/click/tabs/target.js",
    "src/app/interactions/click/toolbar.js",
    "src/app/interactions/click/toolbar/feedback.js",
    "src/app/interactions/click/toolbar/rail.js",
    "src/app/interactions/click/toolbar/state.js",
    "src/app/interactions/click/toolbar/target.js",
    "src/app/interactions/click/toolbar/tool.js",
    "src/app/interactions/click/utils.js",
    "src/app/interactions/fields.js",
    "src/app/interactions/fields/bind.js",
    "src/app/interactions/fields/focus.js",
    "src/app/interactions/fields/input.js",
    "src/app/interactions/fields/target.js",
    "src/app/interactions/keyboard.js",
    "src/app/interactions/keyboard/activate.js",
    "src/app/interactions/keyboard/bind.js",
    "src/app/interactions/keyboard/filter.js",
    "src/app/interactions/keyboard/target.js",
    "src/app/interactions/history.js",
    "src/app/interactions/history/bind.js",
    "src/app/interactions/history/events.js",
    "src/foundation/tokens.css",
    "src/foundation/tokens/dimensions.css",
    "src/foundation/tokens/typography.css",
    "src/foundation/tokens/palette.css",
    "src/foundation/tokens/effects.css",
    "src/foundation/tokens/shape-controls.css",
    "src/foundation/tokens/gaps.css",
    "src/foundation/tokens/base.css",
    "src/components/inputs/atoms.js",
    "src/components/inputs/input-utils.js",
    "src/components/inputs/buttons.js",
    "src/components/inputs/buttons/button.js",
    "src/components/inputs/buttons/icon-button.js",
    "src/components/inputs/fields.js",
    "src/components/inputs/fields/input.js",
    "src/components/inputs/fields/search-input.js",
    "src/components/inputs/fields/number-field.js",
    "src/components/inputs/selection-controls.js",
    "src/components/inputs/selection-controls/checkbox.js",
    "src/components/inputs/selection-controls/radio.js",
    "src/components/inputs/selection-controls/toggle.js",
    "src/components/inputs/tabs.js",
    "src/components/inputs/dropdowns.js",
    "src/components/inputs/dropdowns/select.js",
    "src/components/inputs/sliders.js",
    "src/components/inputs/sliders/slider.js",
    "src/components/inputs/sliders/range-slider.js",
    "src/components/data/collections.js",
    "src/components/data/collection-utils.js",
    "src/components/data/list-view.js",
    "src/components/data/list-view/row.js",
    "src/components/data/table-view.js",
    "src/components/data/table-view/header.js",
    "src/components/data/table-view/row.js",
    "src/components/data/tree-view.js",
    "src/components/data/tree-view/row.js",
    "src/components/data/collections.css",
    "src/components/data/collections/panel-group.css",
    "src/components/data/collections/tree-view.css",
    "src/components/data/collections/table-view.css",
    "src/components/data/collections/list-view.css",
    "src/components/feedback/alerts.js",
    "src/components/feedback/toast.js",
    "src/components/feedback/tooltip.js",
    "src/components/overlays/menu.js",
    "src/components/overlays/menu/row.js",
    "src/components/overlays/popup-layer.js",
    "src/components/surfaces/surfaces.js",
    "src/components/surfaces/shell/window.js",
    "src/components/surfaces/shell/chrome.js",
    "src/components/surfaces/panels/drawer-surface.js",
    "src/components/surfaces/panels/scene-panel.js",
    "src/components/surfaces/panels/inspector-panel.js",
    "src/components/surfaces/panels/showcase-panel.js",
    "src/components/surfaces/viewport/viewport-surface.js",
    "src/components/surfaces/surfaces.css",
    "src/components/surfaces/shell/window.css",
    "src/components/surfaces/shell/topbar.css",
    "src/components/surfaces/shell/rail.css",
    "src/components/surfaces/panels/base.css",
    "src/components/surfaces/panels/scene.css",
    "src/components/surfaces/panels/inspector/layout.css",
    "src/components/surfaces/panels/inspector/object-header.css",
    "src/components/surfaces/panels/inspector/sections.css",
    "src/components/surfaces/panels/inspector/fields.css",
    "src/components/surfaces/panels/inspector/resources.css",
    "src/components/surfaces/panels/showcase/layout.css",
    "src/components/surfaces/panels/showcase/grid.css",
    "src/components/surfaces/panels/showcase/columns.css",
    "src/components/surfaces/panels/showcase/stacks.css",
    "src/components/surfaces/status.css",
    "src/components/surfaces/status/bar.css",
    "src/components/surfaces/status/groups.css",
    "src/components/surfaces/status/controls.css",
    "src/components/surfaces/status/indicators.css",
    "src/components/surfaces/viewport.css",
    "src/components/surfaces/viewport/base.css",
    "src/components/surfaces/viewport/lighting.css",
    "src/components/surfaces/viewport/lighting/lightwash.css",
    "src/components/surfaces/viewport/lighting/shadows.css",
    "src/components/surfaces/viewport/structure.css",
    "src/components/surfaces/viewport/structure/wall.css",
    "src/components/surfaces/viewport/structure/ceiling-door.css",
    "src/components/surfaces/viewport/structure/fixtures.css",
    "src/components/surfaces/viewport/structure/side-walls.css",
    "src/components/surfaces/viewport/structure/rails.css",
    "src/components/surfaces/viewport/floor.css",
    "src/components/surfaces/viewport/floor/base.css",
    "src/components/surfaces/viewport/floor/grid.css",
    "src/components/surfaces/viewport/floor/reflections.css",
    "src/components/surfaces/viewport/floor/grates.css",
    "src/components/surfaces/viewport/floor/panels.css",
    "src/components/surfaces/viewport/floor/seams.css",
    "src/components/surfaces/viewport/props.css",
    "src/components/surfaces/viewport/props/cargo.css",
    "src/components/surfaces/viewport/props/crate.css",
    "src/components/surfaces/viewport/props/selection.css",
    "src/components/surfaces/viewport/props/transform.css",
    "src/components/surfaces/viewport/tools.css",
    "src/components/surfaces/viewport/tools/axis-mini.css",
    "src/components/surfaces/viewport/tools/orientation-gizmo.css",
    "src/components/surfaces/viewport/tools/vignette.css",
    "src/components/surfaces/viewport/tools/toolbar.css",
    "src/foundation/responsive.css",
    "src/foundation/responsive/wide-shell.css",
    "src/foundation/responsive/wide-panels.css",
    "src/foundation/responsive/tablet-shell.css",
    "src/foundation/responsive/tablet-panels.css",
    "src/foundation/responsive/mobile-shell.css",
    "src/foundation/responsive/mobile-panels.css",
    "src/foundation/responsive/compact-controls.css",
    "src/modules/module-canvases.css",
    "src/modules/canvases/map.css",
    "src/modules/canvases/map/base.css",
    "src/modules/canvases/map/walls.css",
    "src/modules/canvases/map/points.css",
    "src/modules/canvases/map/cones.css",
    "src/modules/canvases/map/paths.css",
    "src/modules/canvases/hud.css",
    "src/modules/canvases/hud/base.css",
    "src/modules/canvases/hud/widgets.css",
    "src/modules/canvases/hud/positions.css",
    "src/modules/canvases/hud/status.css",
    "src/modules/canvases/hud/crosshair.css",
    "src/modules/module-feedback.css",
    "src/modules/feedback/inline-status.css",
    "src/modules/module-data.css",
    "src/modules/data/settings.css",
    "src/modules/data/collection-rows.css",
    "src/modules/data/list-rows.css",
    "src/modules/data/tree-rows.css",
    "src/modules/data/table-rows.css",
    "src/modules/data/tags.css",
    "src/modules/data/card-tools.css",
    "src/modules/module-graphs.css",
    "src/modules/graphs/board.css",
    "src/modules/graphs/nodes.css",
    "src/modules/graphs/links.css",
    "src/modules/graphs/minimap.css",
    "src/modules/graphs/curves.css",
    "src/modules/module-output.css",
    "src/modules/output/preview.css",
    "src/modules/output/stats-actions.css",
    "src/modules/output/layout.css",
    "src/modules/output/logs.css",
    "src/modules/output/asset-strip.css",
    "src/modules/output/timeline.css",
    "src/modules/modules.css",
    "src/modules/shell/top-tabs.css",
    "src/modules/shell/toolbar.css",
    "src/modules/shell/regions.css",
    "src/modules/shell/mainbar.css",
    "src/modules/shell/panel-tabs.css",
    "src/modules/shell/cards.css",
    "src/modules/shell/forms.css",
    "src/modules/module-layouts.css",
    "src/modules/layouts/base.css",
    "src/modules/layouts/core.css",
    "src/modules/layouts/library.css",
    "src/modules/layouts/extensions.css",
    "src/modules/extension-library.css",
    "src/modules/extension-library/card-grid.css",
    "src/modules/extension-library/cards.css",
    "src/modules/extension-library/drilldown.css",
    "src/modules/extension-library/panel-group.css",
    "src/modules/module-responsive.css",
    "src/modules/responsive/navigation.css",
    "src/modules/responsive/workspace.css",
    "src/modules/responsive/tablet-shell.css",
    "src/modules/responsive/mobile-stack.css",
    "src/workbench/workbench.css",
    "src/workbench/lower-demo/layout.css",
    "src/workbench/lower-demo/alerts.css",
    "src/workbench/lower-demo/table.css",
    "src/workbench/lower-demo/toast.css",
    "src/workbench/lower-demo/effects.css",
    "src/workbench/lower-demo/tooltip.css",
    "src/workbench/showcase-controls.css",
    "src/workbench/showcase-controls/icon-buttons.css",
    "src/workbench/showcase-controls/shared-gaps.css",
    "src/workbench/showcase-controls/button-grid.css",
    "src/workbench/showcase-controls/button-grid/layout.css",
    "src/workbench/showcase-controls/button-grid/base-controls.css",
    "src/workbench/showcase-controls/button-grid/state-colors.css",
    "src/workbench/showcase-controls/button-grid/item-overrides.css",
    "src/workbench/showcase-controls/fields.css",
    "src/workbench/showcase-controls/selection-controls.css",
    "src/workbench/showcase-controls/segmented-controls.css",
    "src/workbench/showcase-controls/sliders.css",
    "src/workbench/showcase-controls/tabs.css",
    "src/workbench/inspector-detail.css",
    "src/workbench/inspector-detail/base.css",
    "src/workbench/inspector-detail/scene-tree.css",
    "src/workbench/inspector-detail/forms.css",
    "src/workbench/inspector-detail/transform-section.css",
    "src/workbench/inspector-detail/transform-section/section.css",
    "src/workbench/inspector-detail/transform-section/value-boxes.css",
    "src/workbench/inspector-detail/transform-section/vector-rows.css",
    "src/workbench/inspector-detail/transform-section/linked-axis.css",
    "src/workbench/inspector-detail/transform-section/axis-labels.css",
    "src/workbench/inspector-detail/transform-section/controls.css",
    "src/workbench/inspector-detail/mesh-renderer-section.css",
    "src/workbench/side-panels.css",
    "src/workbench/side-panels/menus.css",
    "src/workbench/side-panels/alt-panel.css",
    "src/workbench/side-panels/layer-history.css",
    "src/workbench/side-panels/console.css",
    "src/workbench/side-panels/inspector-checkboxes.css",
    "src/workbench/side-panels/form-overrides.css",
    "src/workbench/side-panels/topbar-overrides.css",
    "src/workbench/statusbar-tuning.css",
    "src/workbench/statusbar-tuning/popup-layer.css",
    "src/workbench/statusbar-tuning/left-group.css",
    "src/workbench/statusbar-tuning/right-group.css",
    "src/workbench/statusbar-tuning/frame.css",
    "src/workbench/statusbar-tuning/controls.css",
    "src/modules/modules.js",
    "src/modules/workbench/registry.js",
    "src/modules/workbench/navigation.js",
    "src/modules/workbench/toolbar.js",
    "src/modules/workbench/rail.js",
    "src/modules/workbench/workspace.js",
    "src/modules/component-lab/module.js",
    "src/modules/component-lab/data.js",
    "src/modules/component-lab/routes.js",
    "src/modules/component-lab/left.js",
    "src/modules/component-lab/center.js",
    "src/modules/component-lab/center/atom-palette.js",
    "src/modules/component-lab/center/collection-palette.js",
    "src/modules/component-lab/center/coverage-matrix.js",
    "src/modules/component-lab/center/lab-column.js",
    "src/modules/component-lab/center/layout-grammar.js",
    "src/modules/component-lab/center/surface-palette.js",
    "src/modules/component-lab/details.js",
    "src/modules/component-lab/bottom.js",
    "src/modules/shared/module-components.js",
    "src/modules/shared/actions.js",
    "src/modules/shared/bottom-output.js",
    "src/modules/shared/panels.js",
    "src/modules/shared/regions.js",
    "src/modules/shared/rows.js",
    "src/modules/shared/utils.js",
    "src/modules/shared/visuals.js",
    "src/modules/core/core-modules.js",
    "src/modules/core/registry/index.js",
    "src/modules/core/registry/gameplay.js",
    "src/modules/core/registry/gameplay/effect.js",
    "src/modules/core/registry/gameplay/ability.js",
    "src/modules/core/registry/gameplay/tags.js",
    "src/modules/core/registry/ai.js",
    "src/modules/core/registry/ai/perception.js",
    "src/modules/core/registry/ai/behavior.js",
    "src/modules/core/registry/rendering.js",
    "src/modules/core/registry/rendering/material.js",
    "src/modules/core/registry/rendering/render-pipeline.js",
    "src/modules/core/registry/rendering/vfx.js",
    "src/modules/core/registry/assets.js",
    "src/modules/core/registry/ui.js",
    "src/modules/core/core-module-details.js",
    "src/modules/core/details/index.js",
    "src/modules/core/details/gameplay.js",
    "src/modules/core/details/gameplay/effect.js",
    "src/modules/core/details/gameplay/ability.js",
    "src/modules/core/details/gameplay/tags.js",
    "src/modules/core/details/ai.js",
    "src/modules/core/details/ai/perception.js",
    "src/modules/core/details/ai/behavior.js",
    "src/modules/core/details/rendering.js",
    "src/modules/core/details/rendering/material.js",
    "src/modules/core/details/rendering/render-pipeline.js",
    "src/modules/core/details/rendering/vfx.js",
    "src/modules/core/details/assets.js",
    "src/modules/core/details/ui.js",
    "src/modules/core/details/routes.js",
    "src/modules/core/core-module-lefts.js",
    "src/modules/core/lefts/index.js",
    "src/modules/core/lefts/gameplay.js",
    "src/modules/core/lefts/gameplay/effect.js",
    "src/modules/core/lefts/gameplay/ability.js",
    "src/modules/core/lefts/gameplay/tags.js",
    "src/modules/core/lefts/ai.js",
    "src/modules/core/lefts/ai/perception.js",
    "src/modules/core/lefts/ai/behavior.js",
    "src/modules/core/lefts/rendering.js",
    "src/modules/core/lefts/rendering/material.js",
    "src/modules/core/lefts/rendering/render-pipeline.js",
    "src/modules/core/lefts/rendering/vfx.js",
    "src/modules/core/lefts/assets.js",
    "src/modules/core/lefts/ui.js",
    "src/modules/core/core-module-centers.js",
    "src/modules/core/centers/index.js",
    "src/modules/core/centers/gameplay.js",
    "src/modules/core/centers/gameplay/effect.js",
    "src/modules/core/centers/gameplay/ability.js",
    "src/modules/core/centers/gameplay/tags.js",
    "src/modules/core/centers/ai.js",
    "src/modules/core/centers/ai/perception.js",
    "src/modules/core/centers/ai/behavior.js",
    "src/modules/core/centers/rendering.js",
    "src/modules/core/centers/rendering/material.js",
    "src/modules/core/centers/rendering/render-pipeline.js",
    "src/modules/core/centers/rendering/vfx.js",
    "src/modules/core/centers/assets.js",
    "src/modules/core/centers/ui.js",
    "src/modules/core/core-module-bottoms.js",
    "src/modules/core/bottoms/index.js",
    "src/modules/core/bottoms/gameplay.js",
    "src/modules/core/bottoms/gameplay/effect.js",
    "src/modules/core/bottoms/gameplay/ability.js",
    "src/modules/core/bottoms/gameplay/tags.js",
    "src/modules/core/bottoms/ai.js",
    "src/modules/core/bottoms/ai/perception.js",
    "src/modules/core/bottoms/ai/behavior.js",
    "src/modules/core/bottoms/rendering.js",
    "src/modules/core/bottoms/rendering/material.js",
    "src/modules/core/bottoms/rendering/render-pipeline.js",
    "src/modules/core/bottoms/rendering/vfx.js",
    "src/modules/core/bottoms/assets.js",
    "src/modules/core/bottoms/ui.js",
    "src/modules/core/bottoms/routes.js",
    "src/modules/extensions/extension-modules.js",
    "src/modules/extensions/extension-configs.js",
    "src/modules/extensions/configs/sources.js",
    "src/modules/extensions/configs/factory.js",
    "src/modules/extensions/configs/recipes.js",
    "src/modules/extensions/configs/recipes/animation.js",
    "src/modules/extensions/configs/recipes/data.js",
    "src/modules/extensions/configs/recipes/default.js",
    "src/modules/extensions/configs/recipes/diagnostics.js",
    "src/modules/extensions/configs/recipes/gameplay.js",
    "src/modules/extensions/configs/recipes/online.js",
    "src/modules/extensions/configs/recipes/production.js",
    "src/modules/extensions/configs/recipes/rendering.js",
    "src/modules/extensions/configs/recipes/runtime.js",
    "src/modules/extensions/configs/recipes/simulation.js",
    "src/modules/extensions/configs/recipes/ui.js",
    "src/modules/extensions/configs/recipes/vfx.js",
    "src/modules/extensions/configs/recipes/world.js",
    "src/modules/extensions/configs/layout-kind.js",
    "src/modules/extensions/configs/controls.js",
    "src/modules/extensions/configs/assets.js",
    "src/modules/extensions/configs/text.js",
    "src/modules/extensions/extension-handoff.js",
    "src/modules/extensions/extension-library.js",
    "src/modules/extensions/library/module.js",
    "src/modules/extensions/library/left.js",
    "src/modules/extensions/library/center.js",
    "src/modules/extensions/library/cards.js",
    "src/modules/extensions/library/drilldown.js",
    "src/modules/extensions/library/details.js",
    "src/modules/extensions/library/bottom.js",
    "src/modules/extensions/library/rows.js",
    "src/modules/extensions/library/routes.js",
    "src/modules/extensions/extension-surfaces.js",
    "src/modules/extensions/surfaces/left.js",
    "src/modules/extensions/surfaces/left/panel.js",
    "src/modules/extensions/surfaces/left/reference.js",
    "src/modules/extensions/surfaces/left/tools.js",
    "src/modules/extensions/surfaces/left/assets.js",
    "src/modules/extensions/surfaces/center.js",
    "src/modules/extensions/surfaces/center/panel.js",
    "src/modules/extensions/surfaces/center/metrics.js",
    "src/modules/extensions/surfaces/center/reference-rhythm.js",
    "src/modules/extensions/surfaces/details.js",
    "src/modules/extensions/surfaces/details/panel.js",
    "src/modules/extensions/surfaces/details/summary.js",
    "src/modules/extensions/surfaces/details/table.js",
    "src/modules/extensions/surfaces/details/status.js",
    "src/modules/extensions/surfaces/bottom.js",
    "src/modules/extensions/surfaces/bottom/panel.js",
    "src/modules/extensions/surfaces/bottom/output.js",
    "src/modules/extensions/surfaces/bottom/validation.js",
    "src/modules/extensions/surfaces/bottom/references.js",
    "src/modules/extensions/surfaces/bottom/handoff.js",
    "src/modules/extensions/surfaces/primary.js",
    "src/modules/extensions/surfaces/primary/panel.js",
    "src/modules/extensions/surfaces/primary/blueprint.js",
    "src/modules/extensions/surfaces/primary/layout-kind.js",
    "src/modules/extensions/surfaces/primary/graph.js",
    "src/modules/extensions/surfaces/routes.js",
    "src/modules/extensions/surfaces/utils.js",
    "src/modules/extensions/extension-blueprints.js",
    "src/modules/extensions/blueprints/helpers.js",
    "src/modules/extensions/blueprints/animation.js",
    "src/modules/extensions/blueprints/animation/animation-compression.js",
    "src/modules/extensions/blueprints/animation/blend-space.js",
    "src/modules/extensions/blueprints/animation/control-rig.js",
    "src/modules/extensions/blueprints/animation/montage-editor.js",
    "src/modules/extensions/blueprints/animation/motion-matching.js",
    "src/modules/extensions/blueprints/animation/pose-library.js",
    "src/modules/extensions/blueprints/animation/retarget.js",
    "src/modules/extensions/blueprints/animation/sequencer.js",
    "src/modules/extensions/blueprints/data.js",
    "src/modules/extensions/blueprints/diagnostics.js",
    "src/modules/extensions/blueprints/gameplay.js",
    "src/modules/extensions/blueprints/multiplayer.js",
    "src/modules/extensions/blueprints/production.js",
    "src/modules/extensions/blueprints/rendering.js",
    "src/modules/extensions/blueprints/simulation.js",
    "src/modules/extensions/blueprints/ui.js",
    "src/modules/extensions/blueprints/world.js",
    "src/modules/extensions/blueprints/world/foliage-editor.js",
    "src/modules/extensions/blueprints/world/level-streaming.js",
    "src/modules/extensions/blueprints/world/level-variant.js",
    "src/modules/extensions/blueprints/world/prefab-editor.js",
    "src/modules/extensions/blueprints/world/scatter-editor.js",
    "src/modules/extensions/blueprints/world/terrain-editor.js",
    "src/modules/extensions/blueprints/world/volume-editor.js",
    "src/modules/extensions/blueprints/world/weather-editor.js",
    "src/routing/routes.js",
    "src/routing/commands/module-targets.js",
    "src/routing/commands/scoped-targets.js",
    "src/routing/commands/panel-targets.js",
    "src/routing/commands/extension-targets.js",
    "src/routing/commands/labels.js",
    "src/routing/commands/route-for-command.js",
    "src/routing/panels/activation.js",
    "src/foundation/icons.js",
  ].map((file) => readFileSync(resolve(here, file), "utf8")).join("\n");
  if (sources.includes("workbench.png")) {
    throw new Error("Component prototype must not embed the full workbench reference screenshot.");
  }
  if (sources.includes("workbench-viewport-reference.png")) {
    throw new Error("Component prototype viewport must be CSS/DOM, not a cropped raster reference.");
  }
}

function auditExpression(width, height) {
  return `(() => {
    const width = ${width};
    const height = ${height};
    const failures = [];
    const app = document.querySelector(".zr-app");
    const windowNode = document.querySelector(".zr-window");
    const topbar = document.querySelector(".zr-topbar");
    const rail = document.querySelector(".zr-rail");
    const viewport = document.querySelector(".zr-viewport");
    const showcase = document.querySelector(".zr-module-bottom");
    const moduleLeft = document.querySelector(".zr-module-left");
    const moduleMain = document.querySelector(".zr-module-main");
    const moduleRight = document.querySelector(".zr-module-right");
    const statusbar = document.querySelector(".zr-statusbar");
    if (!app || !windowNode || !topbar || !rail || !viewport || !showcase || !moduleMain || !moduleRight || !statusbar) {
      return JSON.stringify({ ok: false, failures: ["missing core workbench regions"] });
    }

    const appRect = app.getBoundingClientRect();
    const windowRect = windowNode.getBoundingClientRect();
    const topbarRect = topbar.getBoundingClientRect();
    const railRect = rail.getBoundingClientRect();
    const viewportRect = viewport.getBoundingClientRect();
    const showcaseRect = showcase.getBoundingClientRect();
    const moduleMainRect = moduleMain.getBoundingClientRect();
    const statusbarRect = statusbar.getBoundingClientRect();
    const scroll = document.scrollingElement;

    if (Math.ceil(appRect.width) > width + 1) failures.push("app wider than viewport");
    if (Math.ceil(topbarRect.width) > width + 1) failures.push("topbar wider than viewport");
    if (topbarRect.top < -1 || topbarRect.bottom > Math.max(height, scroll.clientHeight) + 1) failures.push("topbar escapes visible shell");
    if (railRect.left < -1 || railRect.right > width + 1) failures.push("rail escapes viewport");
    if (viewportRect.width < 220) failures.push("viewport collapsed below 220px");
    if (moduleMainRect.width < 220) failures.push("module main collapsed below 220px");
    if (showcaseRect.width < 220) failures.push("component drawer collapsed below 220px");
    if (statusbarRect.left < -1 || statusbarRect.right > width + 1) failures.push("statusbar escapes viewport width");
    if (scroll.scrollWidth > Math.max(width, 640) + 1) failures.push("document horizontal overflow exceeds responsive floor");
    if (width >= 900) {
      const originalHash = window.location.hash;
      history.replaceState(history.state, "", window.location.pathname + window.location.search + "#module=hud-editor&action=workbench.command.tree_canvas_panel");
      window.dispatchEvent(new HashChangeEvent("hashchange"));
      const hudMain = document.querySelector('.zr-module-main[data-module-active="hud-editor"]');
      const hudLeft = document.querySelector('.zr-module-left[data-panel-host="hud-editor"]');
      const hudRight = document.querySelector('.zr-module-right[data-panel-host="hud-editor"]');
      const hudBottom = document.querySelector('.zr-module-bottom[data-panel-host="module-bottom-hud-editor"]');
      const hudGrid = hudMain?.querySelector(".zr-module-editor-grid.is-hud");
      const hudCanvasCard = hudGrid?.querySelector(".zr-module-card:first-child");
      const hudCanvas = hudGrid?.querySelector(".zr-module-hud-canvas");
      if (!hudMain || !hudLeft || !hudRight || !hudBottom || !hudGrid || !hudCanvasCard || !hudCanvas) {
        failures.push("hud stretch route is missing expected module regions");
      } else {
        const leftRect = hudLeft.getBoundingClientRect();
        const mainRect = hudMain.getBoundingClientRect();
        const rightRect = hudRight.getBoundingClientRect();
        const bottomRect = hudBottom.getBoundingClientRect();
        const gridRect = hudGrid.getBoundingClientRect();
        const cardRect = hudCanvasCard.getBoundingClientRect();
        const canvasRect = hudCanvas.getBoundingClientRect();
        if (Math.abs(mainRect.left - leftRect.right) > 2) failures.push("hud main does not start after left drawer");
        if (width > 1040) {
          if (Math.abs(mainRect.right - rightRect.left) > 2) failures.push("hud main does not stretch to right panel");
          if (Math.abs(mainRect.bottom - bottomRect.top) > 2) failures.push("hud main does not stretch to bottom drawer");
        } else {
          if (Math.abs(mainRect.right - windowRect.right) > 2) failures.push("hud compact main does not stretch to shell right edge");
          if (mainRect.bottom - bottomRect.top > 2) failures.push("hud compact main overlaps bottom drawer");
          if (bottomRect.top - mainRect.bottom > 8) failures.push("hud compact main leaves excess gap before bottom drawer");
        }
        if (gridRect.width < mainRect.width - 20) failures.push("hud editor grid does not fill module main width");
        if (gridRect.height < mainRect.height - 54) failures.push("hud editor grid does not fill module main height");
        if (cardRect.height < gridRect.height - 18) failures.push("hud canvas card does not fill editor grid height");
        if (canvasRect.height < cardRect.height - 58) failures.push("hud canvas does not fill card body height");
      }
      history.replaceState(history.state, "", window.location.pathname + window.location.search + (originalHash || "#module=gameplay-effect"));
      window.dispatchEvent(new HashChangeEvent("hashchange"));
    }

    const requiredComponents = [
      ".zr-button",
      ".zr-input",
      ".zr-checkbox",
      ".zr-switch",
      ".zr-icon-button",
      ".zr-tabs",
      ".zr-list",
      ".zr-tree",
      ".zr-table",
      ".zr-popup-layer",
      ".zr-select",
      ".zr-module-tabs",
      ".zr-module-toolbar",
      ".zr-module-left",
      ".zr-module-main",
      ".zr-module-right",
      ".zr-module-bottom",
      ".zr-module-card",
      ".zr-module-table",
      ".zr-module-list",
      ".zr-module-tree",
      ".zr-module-graph",
      ".zr-module-node",
      ".zr-module-status-message",
      "[data-module]",
      "[data-action]",
      '[data-surface="drawer"]',
      '[data-surface="window"]'
    ];
    for (const selector of requiredComponents) {
      if (!document.querySelector(selector)) failures.push("missing component " + selector);
    }

    const fullReferenceImages = [...document.images]
      .map((image) => image.getAttribute("src") || "")
      .filter((src) => /(?:^|\\/)workbench\\.png$/i.test(src));
    if (fullReferenceImages.length > 0) failures.push("runtime embeds full workbench reference screenshot");

    const visibleOutliers = [...document.querySelectorAll("body *")].flatMap((node) => {
      const style = getComputedStyle(node);
      if (style.display === "none" || style.visibility === "hidden" || Number(style.opacity) === 0) return [];
      const rect = node.getBoundingClientRect();
      if (rect.width < 2 || rect.height < 2) return [];
      if (rect.left < -4 || rect.right > Math.max(width, 640) + 4) {
        const label = node.className ? "." + String(node.className).trim().replace(/\\s+/g, ".") : node.tagName.toLowerCase();
        return [label + " [" + Math.round(rect.left) + "," + Math.round(rect.right) + "]"];
      }
      return [];
    }).slice(0, 8);
    if (visibleOutliers.length > 0) failures.push("visible horizontal outliers: " + visibleOutliers.join(", "));

    return JSON.stringify({ ok: failures.length === 0, failures });
  })()`;
}

function interactionAuditExpression() {
  return `(async () => {
    const failures = [];
    const settle = () => Promise.resolve();
    const auditDeadlineAt = performance.now() + ${responsiveInteractionTimeoutMs};
    const auditedGroups = [];
    const capturedHistoryStates = [];
    const originalPushState = history.pushState.bind(history);
    const originalReplaceState = history.replaceState.bind(history);
    const captureHistory = (state, _title, url, mode) => {
      capturedHistoryStates.push({ mode, state, url: String(url || "") });
    };
    history.pushState = (state, title, url) => captureHistory(state, title, url, "push");
    history.replaceState = (state, title, url) => captureHistory(state, title, url, "replace");
    const responseCount = () => Number.parseInt(document.documentElement.dataset.zrResponseCount || "0", 10);
    const activeModule = () => document.querySelector(".zr-module-main")?.dataset.moduleActive || "";
    const activePanel = () => {
      const panelTarget = new URLSearchParams(location.hash.replace(/^#/, "")).get("panel") || "";
      if (!panelTarget) return "";
      return document.querySelector('.zr-panel-view.is-active[data-panel-view="' + attrEscape(panelTarget) + '"]')
        ? panelTarget
        : "";
    };
    const attrEscape = (value) => String(value).replace(/["\\\\]/g, "\\\\$&");
    const checkDeadline = (context) => {
      if (performance.now() <= auditDeadlineAt) return;
      const recentGroups = auditedGroups.slice(-8).map((group) => group.context + ":" + group.count).join(", ");
      throw new Error("responsive interaction audit deadline exceeded at " + context + "; recent groups: " + recentGroups);
    };
    const visible = (node) => {
      if (!node || node.disabled) return false;
      if (node.closest(".zr-panel-view:not(.is-active)")) return false;
      if (node.closest(".zr-popup-layer:not(.is-open)")) return false;
      const style = getComputedStyle(node);
      const rect = node.getBoundingClientRect();
      return style.display !== "none" && style.visibility !== "hidden" && Number(style.opacity) !== 0 && rect.width > 1 && rect.height > 1;
    };
    const labelFor = (node) => {
      const explicit = node.dataset.action || node.dataset.module || node.dataset.panelTab || node.dataset.dropdown || node.dataset.treeRow || "";
      const fieldLabel = node.getAttribute("placeholder") || node.value || node.closest(".zr-module-setting")?.querySelector("span")?.textContent.trim() || "";
      const label = explicit || node.getAttribute("aria-label") || node.getAttribute("title") || fieldLabel || node.textContent.trim().replace(/\\s+/g, " ");
      return (label || node.tagName.toLowerCase()).replace(/\\s+/g, " ").slice(0, 80);
    };
    const controls = (selector) => [...document.querySelectorAll(selector)].filter(visible);
    const routeKeyForAction = (value) => {
      const normalized = String(value || "").trim().toLowerCase();
      const leaf = /^[a-z0-9_]+(?:\\.[a-z0-9_]+)+$/.test(normalized)
        ? normalized.split(".").filter(Boolean).at(-1)
        : normalized;
      return leaf
        .replace(/['’]/g, "")
        .replace(/&/g, " and ")
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/-+/g, "-")
        .replace(/^-|-$/g, "");
    };
    const clickActionByRouteKey = (routeKey) => {
      const node = [...document.querySelectorAll("[data-action]")]
        .find((candidate) => routeKeyForAction(candidate.dataset.action) === routeKey);
      node?.click();
      return Boolean(node);
    };
    const isEditableControl = (node) => node.matches?.("input:not([disabled]), textarea:not([disabled])");
    const clickAndExpectResponse = async (node, context) => {
      const before = responseCount();
      const label = labelFor(node);
      node.click();
      await settle();
      const after = responseCount();
      if (after <= before) {
        failures.push("no response after " + context + ": " + label);
      }
      return after > before;
    };
    const editAndExpectResponse = async (node, context) => {
      const before = responseCount();
      const label = labelFor(node);
      node.focus();
      await settle();
      if (responseCount() <= before) {
        node.value = (node.value || "") + " *";
        node.dispatchEvent(new Event("input", { bubbles: true }));
        await settle();
      }
      const after = responseCount();
      if (after <= before) {
        failures.push("no response after " + context + ": " + label);
      }
      return after > before;
    };
    const exerciseControl = async (node, context) => {
      checkDeadline(context);
      const beforeModule = activeModule();
      const beforePanel = activePanel();
      const responded = isEditableControl(node)
        ? await editAndExpectResponse(node, context)
        : await clickAndExpectResponse(node, context);
      return {
        responded,
        needsRestore: beforeModule !== activeModule() || beforePanel !== activePanel() || !document.contains(node)
      };
    };
    const activateModule = async (id, context = "module tab") => {
      const button = document.querySelector('.zr-module-tab[data-module="' + id + '"]');
      if (!button) {
        failures.push("missing module button after rerender: " + id);
        return false;
      }
      await clickAndExpectResponse(button, context + " " + id);
      if (activeModule() !== id) failures.push("module did not activate: " + id);
      return activeModule() === id;
    };
    const activateExtensionModule = async (id, context = "extension editor") => {
      await activateModule("editor-library", context + " library");
      const card = document.querySelector('[data-module-source="extension-library"][data-module="' + id + '"]');
      if (!card) {
        failures.push("missing extension editor card: " + id);
        return false;
      }
      await clickAndExpectResponse(card, context + " " + id);
      if (activeModule() !== id) failures.push("extension editor did not activate: " + id);
      if (!document.querySelector('.zr-module-main[data-module-active="' + id + '"] .zr-module-editor-grid[data-extension-blueprint="reference"]')) {
        failures.push("extension editor did not render reference blueprint: " + id);
      }
      const activeMoreTab = document.querySelector('.zr-module-tab[data-module="editor-library"]');
      if (!activeMoreTab?.classList.contains("is-active")) failures.push("more tab not active for extension editor: " + id);
      return activeModule() === id;
    };
    const activatePanel = async (target, context) => {
      const tab = document.querySelector('[data-panel-tab="' + target.replace(/"/g, '\\\\"') + '"]');
      if (!tab) {
        failures.push("missing panel tab " + target);
        return false;
      }
      await clickAndExpectResponse(tab, context + " panel");
      const view = document.querySelector('[data-panel-view="' + target.replace(/"/g, '\\\\"') + '"]');
      if (!view?.classList.contains("is-active")) failures.push("panel did not activate: " + target);
      return view?.classList.contains("is-active") ?? false;
    };
    const auditIndexedControls = async (selector, context, restore, limit = Number.POSITIVE_INFINITY) => {
      await restore();
      const available = controls(selector).length;
      const total = Math.min(available, limit);
      if (available === 0) {
        failures.push("no controls found for " + context);
        return 0;
      }
      let needsRestore = false;
      for (let index = 0; index < total; index += 1) {
        checkDeadline(context + " #" + (index + 1) + "/" + available);
        if (needsRestore) {
          await restore();
          needsRestore = false;
        }
        let list = controls(selector);
        let node = list[index];
        if (!node) {
          await restore();
          list = controls(selector);
          node = list[index];
        }
        if (!node) {
          failures.push("control disappeared for " + context + " #" + (index + 1));
          continue;
        }
        const result = await exerciseControl(node, context + " #" + (index + 1) + "/" + available);
        needsRestore = result.needsRestore;
      }
      auditedGroups.push({ context, count: total });
      return total;
    };

    try {
      const moduleIds = [...document.querySelectorAll(".zr-module-tab[data-module]")].map((button) => button.dataset.module);
      if (moduleIds.length < 6) failures.push("expected at least six module tabs");
      for (const id of moduleIds) {
        await activateModule(id);
      }

      for (const id of moduleIds) {
        const railButton = document.querySelector('.zr-rail-module[data-module="' + id + '"]');
        if (!railButton) {
          failures.push("missing rail module button: " + id);
          continue;
        }
        await clickAndExpectResponse(railButton, "rail module " + id);
        if (activeModule() !== id) failures.push("rail module did not activate: " + id);
      }

      await activateModule("editor-library", "extension library");
      const extensionIds = [...document.querySelectorAll('[data-module-source="extension-library"]')].map((button) => button.dataset.module);
      if (extensionIds.length !== ${expectedExtensionCards}) failures.push("expected ${expectedExtensionCards} extension editor cards, found " + extensionIds.length);
      for (const id of extensionIds) {
        await activateExtensionModule(id, "extension editor card");
      }

      for (const id of extensionIds) {
        await activateExtensionModule(id, "extension full control audit");
        await auditIndexedControls(".zr-module-toolbar button:not([disabled])", "all extension toolbar controls " + id, async () => {
          await activateExtensionModule(id, "restore all extension toolbar controls");
        });
        await auditIndexedControls(
          ".zr-module-left button:not([disabled]), .zr-module-left input:not([disabled]), .zr-module-main button:not([disabled]), .zr-module-main [role='button'], .zr-module-main input:not([disabled])",
          "all extension primary surface controls " + id,
          async () => {
            await activateExtensionModule(id, "restore all extension primary surface controls");
          }
        );

        await activateExtensionModule(id, "all extension panel discovery");
        const rightPanelTargets = [...document.querySelectorAll(".zr-module-right .zr-panel-tab")].map((tab) => tab.dataset.panelTab);
        const bottomPanelTargets = [...document.querySelectorAll(".zr-module-bottom .zr-panel-tab")].map((tab) => tab.dataset.panelTab);
        for (const target of rightPanelTargets) {
          await auditIndexedControls(
            ".zr-module-right .zr-panel-tab:not([disabled]), .zr-module-right .zr-panel-view.is-active button:not([disabled]), .zr-module-right .zr-panel-view.is-active [role='button'], .zr-module-right .zr-panel-view.is-active input:not([disabled])",
            "all extension right panel controls " + target,
            async () => {
              await activateExtensionModule(id, "restore all extension right panel module");
              await activatePanel(target, "restore all extension right panel");
            }
          );
        }
        for (const target of bottomPanelTargets) {
          await auditIndexedControls(
            ".zr-module-bottom .zr-panel-tab:not([disabled]), .zr-module-bottom .zr-panel-view.is-active button:not([disabled]), .zr-module-bottom .zr-panel-view.is-active [role='button'], .zr-module-bottom .zr-panel-view.is-active input:not([disabled])",
            "all extension bottom panel controls " + target,
            async () => {
              await activateExtensionModule(id, "restore all extension bottom panel module");
              await activatePanel(target, "restore all extension bottom panel");
            }
          );
        }
      }

      await activateModule("gameplay-effect", "restore default");
      await auditIndexedControls(
        ".zr-topbar > .zr-topbar-group:first-child button:not([disabled]), .zr-topbar > .zr-topbar-group:last-child button:not([disabled]), .zr-rail button:not([disabled]), .zr-statusbar button:not([disabled])",
        "global toolbar/status controls",
        async () => {
          await activateModule("gameplay-effect", "restore global");
        }
      );

      for (const id of moduleIds) {
        await activateModule(id, "module audit");
        await auditIndexedControls(".zr-module-toolbar button:not([disabled])", "module toolbar " + id, async () => {
          await activateModule(id, "restore module toolbar");
        });
        await auditIndexedControls(
          ".zr-module-left button:not([disabled]), .zr-module-left input:not([disabled]), .zr-module-main button:not([disabled]), .zr-module-main [role='button'], .zr-module-main input:not([disabled])",
          "module primary surfaces " + id,
          async () => {
            await activateModule(id, "restore primary surfaces");
          }
        );

        await activateModule(id, "module panel discovery");
        const rightPanelTargets = [...document.querySelectorAll(".zr-module-right .zr-panel-tab")].map((tab) => tab.dataset.panelTab);
        const bottomPanelTargets = [...document.querySelectorAll(".zr-module-bottom .zr-panel-tab")].map((tab) => tab.dataset.panelTab);
        for (const target of rightPanelTargets) {
          await auditIndexedControls(
            ".zr-module-right .zr-panel-tab:not([disabled]), .zr-module-right .zr-panel-view.is-active button:not([disabled]), .zr-module-right .zr-panel-view.is-active [role='button'], .zr-module-right .zr-panel-view.is-active input:not([disabled])",
            "right panel " + target,
            async () => {
              await activateModule(id, "restore right panel module");
              await activatePanel(target, "restore right");
            }
          );
        }
        for (const target of bottomPanelTargets) {
          await auditIndexedControls(
            ".zr-module-bottom .zr-panel-tab:not([disabled]), .zr-module-bottom .zr-panel-view.is-active button:not([disabled]), .zr-module-bottom .zr-panel-view.is-active [role='button'], .zr-module-bottom .zr-panel-view.is-active input:not([disabled])",
            "bottom panel " + target,
            async () => {
              await activateModule(id, "restore bottom panel module");
              await activatePanel(target, "restore bottom");
            }
          );
        }
      }

      await activateModule("gameplay-effect", "route restore");
      clickActionByRouteKey("browse");
      await settle();
      if (document.querySelector(".zr-module-main")?.dataset.moduleActive !== "asset-browser") failures.push("browse did not route to asset browser");
      await activateModule("gameplay-effect", "route compile restore");
      clickActionByRouteKey("compile");
      await settle();
      const compilePanel = document.querySelector('[data-panel-view="module-bottom-gameplay-effect:compile-log"]');
      if (!compilePanel?.classList.contains("is-active")) failures.push("compile did not route to compile log");
      await activateModule("material", "route material restore");
      clickActionByRouteKey("texture-sample");
      await settle();
      if (document.querySelector(".zr-module-main")?.dataset.moduleActive !== "material") failures.push("texture sample did not stay on material module");
      await activateModule("gameplay-effect", "popup restore");
      const dropdown = controls("[data-dropdown]")[0];
      if (!dropdown) {
        failures.push("no dropdown available for popup audit");
      } else {
        await clickAndExpectResponse(dropdown, "dropdown popup open");
        const menuRows = controls(".zr-popup-layer .zr-menu-row");
        if (menuRows.length === 0) failures.push("popup menu did not expose menu rows");
        for (let index = 0; index < menuRows.length; index += 1) {
          await clickAndExpectResponse(menuRows[index], "popup menu row " + (index + 1));
        }
      }
    } catch (error) {
      failures.push(error?.message ?? String(error));
    } finally {
      history.pushState = originalPushState;
      history.replaceState = originalReplaceState;
    }

    const interactionRouteWrites = capturedHistoryStates.length;
    if (interactionRouteWrites === 0) failures.push("interaction audit captured no route-state writes");
    return JSON.stringify({ ok: failures.length === 0, failures, interactionRouteWrites, auditedGroups });
  })()`;
}

async function waitForWorkbench(cdp, attempts = 80) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    const ready = await evaluate(cdp, `(() => {
      const activeModule = document.querySelector(".zr-module-main")?.dataset.moduleActive || "";
      const hashModule = new URLSearchParams(location.hash.replace(/^#/, "")).get("module") || "";
      return Boolean(
        document.querySelector(".zr-window")
        && activeModule
        && hashModule === activeModule
        && document.querySelector('[data-module-panel="bottom"]')
      );
    })()`);
    if (ready) return;
    await delay(100);
  }
  throw new Error("Timed out waiting for workbench component prototype.");
}

async function assertModuleClickHistory(cdp) {
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
  await clickModuleAndWait(cdp, "editor-library");
  await clickModuleAndWait(cdp, "terrain-editor");
  await clickModuleAndWait(cdp, "editor-library");
  await clickModuleAndWait(cdp, "weather-editor");
  await waitForHistoryState(cdp, "weather-editor");
  await cdp.send("Runtime.evaluate", {
    expression: "history.back()",
    awaitPromise: true,
    returnByValue: true,
  });
  await waitForHistoryState(cdp, "editor-library");
  await cdp.send("Runtime.evaluate", {
    expression: "history.forward()",
    awaitPromise: true,
    returnByValue: true,
  });
  await waitForHistoryState(cdp, "weather-editor");
}

async function assertPanelDeepLinks(cdp) {
  await assertDeepLink(cdp, "gameplay-effect", "gameplay-effect", "module-bottom-gameplay-effect:compile-log");
  await assertDeepLink(cdp, "asset-browser", "asset-browser", "asset-right:metadata");
  await assertDeepLink(cdp, "shader-editor", "shader-editor", "shader-editor-right:resources");
  await assertDeepLink(cdp, "missing-module", "gameplay-effect", "module-bottom-gameplay-effect:compile-log", "");
  await assertDeepLink(cdp, "gameplay-effect", "gameplay-effect", "asset-right:metadata", "");
}

async function assertCommandPanelHistory(cdp) {
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
  await clickActionAndWait(cdp, "compile", "gameplay-effect", "module-bottom-gameplay-effect:compile-log", "compile");
  await clickActionAndWait(cdp, "browse", "asset-browser", "", "browse");
  await clickPanelTabAndWait(cdp, "asset-right:metadata", "asset-browser", "workbench.panel.select.asset_right_metadata");
  await cdp.send("Runtime.evaluate", {
    expression: "history.back()",
    awaitPromise: true,
    returnByValue: true,
  });
  await waitForHistoryState(cdp, "asset-browser", "", "browse");
  await cdp.send("Runtime.evaluate", {
    expression: "history.forward()",
    awaitPromise: true,
    returnByValue: true,
  });
  await waitForHistoryState(cdp, "asset-browser", "asset-right:metadata", "workbench.panel.select.asset_right_metadata");
}

async function assertModuleScopedCommandRoutes(cdp) {
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
  await clickModuleAndWait(cdp, "material");
  await clickActiveModuleActionAndWait(cdp, "compile", "material", "module-bottom-material:shader-output", "compile");
  await clickActiveModuleActionAndWait(cdp, "preview", "material", "module-bottom-material:preview-variants", "preview");
  await clickActiveModuleActionAndWait(cdp, "build", "material", "module-bottom-material:warnings", "build");
  await clickModuleAndWait(cdp, "behavior-tree");
  await clickActiveModuleActionAndWait(cdp, "play", "behavior-tree", "behavior-right:execution", "play");
  await clickActiveModuleActionAndWait(cdp, "validate", "behavior-tree", "module-bottom-behavior-tree:validation-issues", "validate");
  await clickModuleAndWait(cdp, "asset-browser");
  await clickActiveModuleActionAndWait(cdp, "validate", "asset-browser", "module-bottom-asset-browser:validation", "validate");
  await clickActiveModuleActionAndWait(cdp, "build", "asset-browser", "module-bottom-asset-browser:cook", "build");
  await clickModuleAndWait(cdp, "vfx");
  await clickActiveModuleActionAndWait(cdp, "compile", "vfx", "module-bottom-vfx:compile-output", "compile");
}

async function assertExtensionCommandRoutes(cdp) {
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
  await clickExtensionModuleAndWait(cdp, "shader-editor");
  await clickActiveModuleActionAndWait(cdp, "native-handoff", "shader-editor", "module-bottom-shader-editor:handoff", "native-handoff");
  await clickActiveModuleActionAndWait(cdp, "compile-shader", "shader-editor", "module-bottom-shader-editor:validation", "compile-shader");
  await clickActiveModuleActionAndWait(cdp, "save-shader", "shader-editor", "module-bottom-shader-editor:references", "save-shader");
  await clickActiveModuleActionAndWait(cdp, "preview-shader", "shader-editor", "module-bottom-shader-editor:output", "preview-shader");
  await clickExtensionModuleAndWait(cdp, "source-control");
  await clickActiveModuleActionAndWait(cdp, "review-source-control", "source-control", "module-bottom-source-control:references", "review-source-control");
  await clickActiveModuleActionAndWait(cdp, "run-source-control", "source-control", "module-bottom-source-control:output", "run-source-control");
  await clickExtensionModuleAndWait(cdp, "weather-editor");
  await clickActiveModuleActionAndWait(cdp, "build-weather", "weather-editor", "module-bottom-weather-editor:validation", "build-weather");
  await clickActiveModuleActionAndWait(cdp, "preview-weather", "weather-editor", "module-bottom-weather-editor:output", "preview-weather");
}

async function assertAllTopLevelToolbarCommandRoutes(cdp) {
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
  const state = JSON.parse(await evaluate(cdp, allTopLevelToolbarCommandRoutesExpression()));
  if (!state.ok) {
    throw new Error(`Top-level toolbar route audit failed:\n${state.failures.join("\n")}`);
  }
}

function allTopLevelToolbarCommandRoutesExpression() {
  return `(async () => {
    const expectedTopLevelModuleTabs = ${expectedTopLevelModuleTabs};
    const failures = [];
    const settle = () => Promise.resolve();
    const escapeCss = (value) => globalThis.CSS?.escape
      ? globalThis.CSS.escape(value)
      : String(value).replace(/["\\\\]/g, "\\\\$&");
    const responseCount = () => Number.parseInt(document.documentElement.dataset.zrResponseCount || "0", 10);
    const activeModule = () => document.querySelector(".zr-module-main")?.dataset.moduleActive || "";
    const routeState = () => {
      const params = new URLSearchParams(location.hash.replace(/^#/, ""));
      const hashPanel = params.get("panel") || "";
      const activePanel = hashPanel && [...document.querySelectorAll(".zr-panel-view.is-active")]
        .some((view) => view.dataset.panelView === hashPanel)
        ? hashPanel
        : "";
      return {
        activeModule: activeModule(),
        hashModule: params.get("module") || "",
        activePanel,
        hashPanel,
        hashAction: params.get("action") || params.get("command") || ""
      };
    };
    const click = async (selector, context) => {
      const node = document.querySelector(selector);
      if (!node) {
        failures.push("missing " + context + ": " + selector);
        return false;
      }
      node.click();
      await settle();
      return true;
    };
    const openModule = async (id) => {
      if (!await click('.zr-module-tab[data-module="' + escapeCss(id) + '"]', "module tab " + id)) return false;
      if (activeModule() !== id) failures.push("top-level module did not activate: " + id);
      return activeModule() === id;
    };
    const assertRoute = (id, action, before) => {
      const state = routeState();
      if (responseCount() <= before) failures.push("no response after top-level toolbar action " + id + "/" + action);
      if (state.activeModule !== state.hashModule) {
        failures.push("active/hash module mismatch after " + id + "/" + action + ": " + state.activeModule + "/" + state.hashModule);
      }
      if (state.activePanel !== state.hashPanel) {
        failures.push("active/hash panel mismatch after " + id + "/" + action + ": " + state.activePanel + "/" + state.hashPanel);
      }
      if (state.hashAction !== action) {
        failures.push("action mismatch after " + id + "/" + action + ": " + state.hashAction);
      }
    };

    const moduleIds = [...document.querySelectorAll(".zr-module-tab[data-module]")]
      .map((button) => button.dataset.module)
      .filter(Boolean);
    if (moduleIds.length !== expectedTopLevelModuleTabs) {
      failures.push("expected " + expectedTopLevelModuleTabs + " top-level module tabs, found " + moduleIds.length);
    }

    for (const id of moduleIds) {
      if (!await openModule(id)) continue;
      const actions = [...document.querySelectorAll(".zr-module-toolbar [data-action]")]
        .map((button) => button.dataset.action)
        .filter(Boolean);
      const uniqueActions = new Set(actions);
      if (actions.length === 0) failures.push("no toolbar actions for top-level module " + id);
      if (uniqueActions.size !== actions.length) failures.push("duplicate toolbar action ids for top-level module " + id);

      for (const action of actions) {
        if (!await openModule(id)) continue;
        const before = responseCount();
        if (!await click('.zr-module-toolbar [data-action="' + escapeCss(action) + '"]', "top-level toolbar action " + id + "/" + action)) continue;
        assertRoute(id, action, before);
      }
    }

    return JSON.stringify({ ok: failures.length === 0, failures });
  })()`;
}

async function assertAllExtensionToolbarCommandRoutes(cdp) {
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
  const state = JSON.parse(await evaluate(cdp, allExtensionToolbarCommandRoutesExpression()));
  if (!state.ok) {
    throw new Error(`Extension toolbar route audit failed:\n${state.failures.join("\n")}`);
  }
}

function allExtensionToolbarCommandRoutesExpression() {
  return `(async () => {
    const expectedExtensionCards = ${expectedExtensionCards};
    const failures = [];
    const settle = () => Promise.resolve();
    let observedHash = location.hash;
    const captureHistoryHash = (_state, _title, url) => {
      if (url) {
        observedHash = new URL(url, location.href).hash;
      }
    };
    history.pushState = captureHistoryHash;
    history.replaceState = captureHistoryHash;
    const escapeCss = (value) => globalThis.CSS?.escape
      ? globalThis.CSS.escape(value)
      : String(value).replace(/["\\\\]/g, "\\\\$&");
    const responseCount = () => Number.parseInt(document.documentElement.dataset.zrResponseCount || "0", 10);
    const activeModule = () => document.querySelector(".zr-module-main")?.dataset.moduleActive || "";
    const routeKeyForAction = (command) => {
      const normalized = String(command || "").trim().toLowerCase();
      const leaf = /^[a-z0-9_]+(?:\\.[a-z0-9_]+)+$/.test(normalized)
        ? normalized.split(".").filter(Boolean).at(-1)
        : normalized;
      return leaf.replace(/_/g, "-");
    };
    const extensionPanelKeyForToolbarCommand = (command) => {
      const tokens = routeKeyForAction(command).split("-").filter(Boolean);
      const verb = tokens[0] ?? "";
      if (["native", "handoff", "promote", "promotion", "matrix", "gate", "zui", "retained"].includes(verb)
        || tokens.some((token) => ["native", "handoff", "promotion", "matrix", "gate", "zui", "retained"].includes(token))) {
        return "handoff";
      }
      if (["validate", "compile", "build", "check", "audit", "open"].includes(verb)
        || tokens.some((token) => ["issue", "issues", "warning", "warnings", "error", "errors"].includes(token))) {
        return "validation";
      }
      if (["reference", "references", "browse", "history", "review", "save", "export", "publish", "migrate", "load"].includes(verb)) {
        return "references";
      }
      return "output";
    };
    const routeState = () => {
      const params = new URLSearchParams(observedHash.replace(/^#/, ""));
      const hashPanel = params.get("panel") || "";
      const activePanel = hashPanel && [...document.querySelectorAll(".zr-panel-view.is-active")]
        .some((view) => view.dataset.panelView === hashPanel)
        ? hashPanel
        : "";
      return {
        activeModule: activeModule(),
        hashModule: params.get("module") || "",
        activePanel,
        hashPanel,
        hashAction: params.get("action") || params.get("command") || ""
      };
    };
    const click = async (selector, context) => {
      const node = document.querySelector(selector);
      if (!node) {
        failures.push("missing " + context + ": " + selector);
        return false;
      }
      node.click();
      await settle();
      return true;
    };
    const openExtension = async (id) => {
      if (!await click('.zr-module-tab[data-module="editor-library"]', "More Editors tab before " + id)) return false;
      if (!await click('[data-module-source="extension-library"][data-module="' + escapeCss(id) + '"]', "extension card " + id)) return false;
      if (activeModule() !== id) failures.push("extension editor did not activate: " + id);
      if (!document.querySelector('.zr-module-main[data-module-active="' + escapeCss(id) + '"] .zr-module-editor-grid[data-extension-blueprint="reference"]')) {
        failures.push("extension editor did not render reference blueprint: " + id);
      }
      return activeModule() === id;
    };
    const assertRoute = (id, action, expectedModule, expectedPanel) => {
      const state = routeState();
      if (
        state.activeModule !== expectedModule
        || state.hashModule !== expectedModule
        || state.activePanel !== expectedPanel
        || state.hashPanel !== expectedPanel
        || state.hashAction !== action
      ) {
        failures.push(
          "toolbar route mismatch " + id + "/" + action
          + ": expected " + expectedModule + "/" + expectedPanel + "/" + action
          + ", got active=" + state.activeModule + "/" + state.activePanel
          + " hash=" + state.hashModule + "/" + state.hashPanel + "/" + state.hashAction
        );
      }
    };

    await click('.zr-module-tab[data-module="editor-library"]', "More Editors tab");
    const extensionIds = [...document.querySelectorAll('[data-module-source="extension-library"][data-module]')]
      .map((card) => card.dataset.module)
      .filter(Boolean);
    if (extensionIds.length !== expectedExtensionCards) {
      failures.push("expected " + expectedExtensionCards + " extension editor cards, found " + extensionIds.length);
    }

    for (const id of extensionIds) {
      if (!await openExtension(id)) continue;
      const actions = [...document.querySelectorAll(".zr-module-toolbar [data-action]")]
        .map((button) => button.dataset.action)
        .filter(Boolean);
      const uniqueActions = new Set(actions);
      if (actions.length < 5) failures.push("expected at least five toolbar actions for " + id + ", found " + actions.length);
      if (!actions.some((action) => routeKeyForAction(action) === "more-editors")) failures.push("missing More Editors toolbar route for " + id);
      if (uniqueActions.size !== actions.length) failures.push("duplicate toolbar action ids for " + id);

      for (const action of actions) {
        if (!await openExtension(id)) continue;
        const before = responseCount();
        if (!await click('.zr-module-toolbar [data-action="' + escapeCss(action) + '"]', "toolbar action " + id + "/" + action)) continue;
        if (responseCount() <= before) failures.push("no response after toolbar action " + id + "/" + action);
        if (routeKeyForAction(action) === "more-editors") {
          assertRoute(id, action, "editor-library", "module-bottom-editor-library:routing-log");
        } else {
          assertRoute(id, action, id, "module-bottom-" + id + ":" + extensionPanelKeyForToolbarCommand(action));
        }
      }
    }

    return JSON.stringify({ ok: failures.length === 0, failures });
  })()`;
}

async function assertCommandState(cdp) {
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
  await clickActionAndWait(cdp, "save", "gameplay-effect", "", "save");
  await clickTreeRowAndWait(cdp, "ge-healthregen", "gameplay-effect", "", "workbench.tree.select.ge_health_regen");
  await editFieldAndWait(cdp, ".zr-module-left input[placeholder='Search assets...']:not([disabled])", "gameplay-effect", "", "workbench.field.edit.search_assets");
}

async function assertKeyboardActivation(cdp) {
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
  await assertElementAttribute(
    cdp,
    '.zr-module-table-row[role="button"][data-action$=".health_regen"]',
    "aria-label",
    "HealthRegen",
  );
  await assertElementAttribute(
    cdp,
    '.zr-module-table-row[role="button"][data-action$=".incoming_healing"]',
    "aria-label",
    "IncomingHealing",
  );
  await pressActionKeyAndWait(
    cdp,
    '.zr-module-table-row[role="button"][data-action$=".health_regen"]',
    "Enter",
    "gameplay-effect",
    "",
    "workbench.module.table.health_regen",
  );
  await pressActionKeyAndWait(
    cdp,
    '.zr-module-table-row[role="button"][data-action$=".incoming_healing"]',
    " ",
    "gameplay-effect",
    "",
    "workbench.module.table.incoming_healing",
  );
}

async function assertCollectionRowButtons(cdp) {
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
  await assertElementAttribute(
    cdp,
    '.zr-module-left .zr-module-list-row[data-action$=".target_tags"]',
    "type",
    "button",
  );
  await assertElementAttribute(
    cdp,
    '.zr-module-table-row[role="button"][data-action$=".health_regen"]',
    "aria-label",
    "HealthRegen",
  );
  await pressActionKeyAndWait(
    cdp,
    '.zr-module-left .zr-module-list-row[data-action$=".target_tags"]',
    "Enter",
    "gameplay-effect",
    "",
    "workbench.module.list.target_tags",
  );
  await pressActionKeyAndWait(
    cdp,
    '.zr-module-table-row[role="button"][data-action$=".health_regen"]',
    " ",
    "gameplay-effect",
    "",
    "workbench.module.table.health_regen",
  );
}

async function assertCollectionTreeRows(cdp) {
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
  const mounted = JSON.parse(await evaluate(cdp, `(async () => {
    const { treeView } = await import(new URL("./src/components/data/collections.js", location.href).href);
    document.querySelector("#zr-tree-contract-host")?.remove();
    const host = document.createElement("section");
    host.id = "zr-tree-contract-host";
    host.innerHTML = treeView([{
      id: "contract-root",
      label: "Contract Root",
      icon: "cube",
      children: [{ id: "contract-child", label: "Contract Child", icon: "cube" }]
    }]);
    document.body.append(host);
    const root = host.querySelector('[data-tree-row="contract-root"]');
    const child = host.querySelector('[data-tree-row="contract-child"]');
    return JSON.stringify({
      rootTag: root?.tagName ?? "",
      rootType: root?.getAttribute("type") ?? "",
      rootAction: root?.dataset.action ?? "",
      rootLabel: root?.getAttribute("aria-label") ?? "",
      childTag: child?.tagName ?? "",
      childType: child?.getAttribute("type") ?? "",
      childAction: child?.dataset.action ?? "",
      childLabel: child?.getAttribute("aria-label") ?? ""
    });
  })()`));

  const expected = {
    rootTag: "BUTTON",
    rootType: "button",
    rootAction: "workbench.collection.tree.contract_root",
    rootLabel: "Contract Root",
    childTag: "BUTTON",
    childType: "button",
    childAction: "workbench.collection.tree.contract_child",
    childLabel: "Contract Child",
  };
  for (const [key, value] of Object.entries(expected)) {
    if (mounted[key] !== value) {
      throw new Error(`Expected tree contract ${key}=${value}, got ${mounted[key]}.`);
    }
  }

  await cdp.send("Runtime.evaluate", {
    expression: `document.querySelector('#zr-tree-contract-host [data-tree-row="contract-child"]')?.click()`,
    awaitPromise: true,
    returnByValue: true,
  });
  await waitForHistoryState(cdp, "gameplay-effect", "", "workbench.collection.tree.contract_child");
}

async function assertPopupMenuSelection(cdp) {
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);

  const menuActions = JSON.parse(await evaluate(cdp, `(() => JSON.stringify([...document.querySelectorAll(".zr-popup-layer [data-menu-item][data-action]")]
    .map((row) => row.dataset.action)
    .filter(Boolean)))()`));
  if (menuActions.length === 0) {
    throw new Error("Expected popup layer to expose action-backed menu rows.");
  }

  for (const action of menuActions) {
    await cdp.send("Page.navigate", { url: referenceUrl });
    await waitForWorkbench(cdp);
    await cdp.send("Runtime.evaluate", {
      expression: `document.querySelector("[data-dropdown]")?.click()`,
      awaitPromise: true,
      returnByValue: true,
    });
    const opened = await evaluate(cdp, `document.querySelector(".zr-popup-layer")?.classList.contains("is-open") ?? false`);
    if (!opened) {
      throw new Error(`Expected popup layer to open before selecting ${action}.`);
    }

    await cdp.send("Runtime.evaluate", {
      expression: `document.querySelector('.zr-popup-layer [data-action="${cssEscape(action)}"]')?.click()`,
      awaitPromise: true,
      returnByValue: true,
    });
    const closed = await evaluate(cdp, `!(document.querySelector(".zr-popup-layer")?.classList.contains("is-open") ?? false)`);
    if (!closed) {
      throw new Error(`Expected popup layer to close after selecting ${action}.`);
    }
    await waitForHistoryState(cdp, "gameplay-effect", "", action);
  }
}

async function assertElementAttribute(cdp, selector, attributeName, expectedValue) {
  const actualValue = await evaluate(
    cdp,
    `document.querySelector(${JSON.stringify(selector)})?.getAttribute(${JSON.stringify(attributeName)}) ?? ""`,
  );
  if (actualValue !== expectedValue) {
    throw new Error(`Expected ${selector} ${attributeName}=${expectedValue}, got ${actualValue}.`);
  }
}

async function clickModuleAndWait(cdp, moduleId) {
  const selector = `[data-module="${cssEscape(moduleId)}"]`;
  await cdp.send("Runtime.evaluate", {
    expression: `document.querySelector(${JSON.stringify(selector)})?.click()`,
    awaitPromise: true,
    returnByValue: true,
  });
  await waitForHistoryState(cdp, moduleId);
}

async function clickExtensionModuleAndWait(cdp, moduleId) {
  await clickModuleAndWait(cdp, "editor-library");
  const selector = `[data-module-source="extension-library"][data-module="${cssEscape(moduleId)}"]`;
  await cdp.send("Runtime.evaluate", {
    expression: `document.querySelector(${JSON.stringify(selector)})?.click()`,
    awaitPromise: true,
    returnByValue: true,
  });
  await waitForHistoryState(cdp, moduleId);
}

async function clickActionAndWait(cdp, action, expectedModuleId, expectedPanelTarget = "", expectedCommandId = "", attempts = 80) {
  const selector = actionSelector(action);
  await cdp.send("Runtime.evaluate", {
    expression: `document.querySelector(${JSON.stringify(selector)})?.click()`,
    awaitPromise: true,
    returnByValue: true,
  });
  await waitForHistoryState(cdp, expectedModuleId, expectedPanelTarget, expectedCommandId, attempts);
}

async function clickActiveModuleActionAndWait(cdp, action, expectedModuleId, expectedPanelTarget = "", expectedCommandId = "", attempts = 80) {
  const selector = scopedActionSelector(".zr-module-toolbar", action);
  await cdp.send("Runtime.evaluate", {
    expression: `document.querySelector(${JSON.stringify(selector)})?.click()`,
    awaitPromise: true,
    returnByValue: true,
  });
  await waitForHistoryState(cdp, expectedModuleId, expectedPanelTarget, expectedCommandId, attempts);
}

async function clickPanelTabAndWait(cdp, panelTarget, expectedModuleId, expectedCommandIdOrAttempts = "", attempts = 80) {
  const selector = `[data-panel-tab="${cssEscape(panelTarget)}"]`;
  await cdp.send("Runtime.evaluate", {
    expression: `document.querySelector(${JSON.stringify(selector)})?.click()`,
    awaitPromise: true,
    returnByValue: true,
  });
  await waitForHistoryState(cdp, expectedModuleId, panelTarget, expectedCommandIdOrAttempts, attempts);
}

async function clickTreeRowAndWait(cdp, treeRowId, expectedModuleId, expectedPanelTarget = "", expectedCommandId = "", attempts = 80) {
  const selector = `[data-tree-row="${cssEscape(treeRowId)}"]`;
  await cdp.send("Runtime.evaluate", {
    expression: `document.querySelector(${JSON.stringify(selector)})?.click()`,
    awaitPromise: true,
    returnByValue: true,
  });
  await waitForHistoryState(cdp, expectedModuleId, expectedPanelTarget, expectedCommandId, attempts);
}

async function editFieldAndWait(cdp, selector, expectedModuleId, expectedPanelTarget = "", expectedCommandId = "", attempts = 80) {
  await cdp.send("Runtime.evaluate", {
    expression: `(() => {
      const field = document.querySelector(${JSON.stringify(selector)});
      if (!field) return false;
      field.focus();
      field.value = (field.value || "") + " query";
      field.dispatchEvent(new Event("input", { bubbles: true }));
      return true;
    })()`,
    awaitPromise: true,
    returnByValue: true,
  });
  await waitForHistoryState(cdp, expectedModuleId, expectedPanelTarget, expectedCommandId, attempts);
}

async function pressActionKeyAndWait(cdp, selector, key, expectedModuleId, expectedPanelTarget = "", expectedCommandId = "", attempts = 80) {
  await cdp.send("Runtime.evaluate", {
    expression: `(() => {
      const target = document.querySelector(${JSON.stringify(selector)});
      if (!target) return false;
      target.focus();
      target.dispatchEvent(new KeyboardEvent("keydown", {
        key: ${JSON.stringify(key)},
        bubbles: true,
        cancelable: true
      }));
      return true;
    })()`,
    awaitPromise: true,
    returnByValue: true,
  });
  await waitForHistoryState(cdp, expectedModuleId, expectedPanelTarget, expectedCommandId, attempts);
}

async function waitForHistoryState(cdp, expectedModuleId, expectedPanelTarget = "", expectedCommandIdOrAttempts = "", attempts = 80) {
  const expectedCommandId = typeof expectedCommandIdOrAttempts === "number" ? "" : expectedCommandIdOrAttempts;
  const maxAttempts = typeof expectedCommandIdOrAttempts === "number" ? expectedCommandIdOrAttempts : attempts;
  for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
    const state = JSON.parse(await evaluate(cdp, moduleHistoryStateExpression()));
    if (
      state.activeModule === expectedModuleId
      && state.hashModule === expectedModuleId
      && state.activePanel === expectedPanelTarget
      && state.hashPanel === expectedPanelTarget
      && commandMatches(state.hashAction, expectedCommandId)
    ) {
      return state;
    }
    await delay(100);
  }
  const state = JSON.parse(await evaluate(cdp, moduleHistoryStateExpression()));
  throw new Error(`Expected history state ${expectedModuleId}/${expectedPanelTarget}/${expectedCommandId}, got active=${state.activeModule}/${state.activePanel} hash=${state.hashModule}/${state.hashPanel}/${state.hashAction}.`);
}

async function assertDeepLink(cdp, requestedModuleId, expectedModuleId, requestedPanelTarget = "", expectedPanelTarget = requestedPanelTarget) {
  const panelParam = requestedPanelTarget ? `&panel=${encodeURIComponent(requestedPanelTarget)}` : "";
  await cdp.send("Page.navigate", { url: `${referenceUrl}#module=${encodeURIComponent(requestedModuleId)}${panelParam}` });
  await waitForWorkbench(cdp);
  const state = await waitForHistoryState(cdp, expectedModuleId, expectedPanelTarget);
  if (state.activeModule !== expectedModuleId) {
    throw new Error(`Deep link ${requestedModuleId} activated ${state.activeModule}, expected ${expectedModuleId}.`);
  }
  if (state.hashModule !== expectedModuleId) {
    throw new Error(`Deep link ${requestedModuleId} left hash ${state.hashModule}, expected ${expectedModuleId}.`);
  }
  if (state.activePanel !== expectedPanelTarget) {
    throw new Error(`Deep link ${requestedModuleId} activated panel ${state.activePanel}, expected ${expectedPanelTarget}.`);
  }
  if (state.hashPanel !== expectedPanelTarget) {
    throw new Error(`Deep link ${requestedModuleId} left panel hash ${state.hashPanel}, expected ${expectedPanelTarget}.`);
  }
}

function moduleHistoryStateExpression() {
  return `(() => {
    const activeModule = document.querySelector(".zr-module-main")?.dataset.moduleActive || "";
    const hashModule = new URLSearchParams(location.hash.replace(/^#/, "")).get("module") || "";
    const hashPanel = new URLSearchParams(location.hash.replace(/^#/, "")).get("panel") || "";
    const params = new URLSearchParams(location.hash.replace(/^#/, ""));
    const hashAction = params.get("action") || params.get("command") || "";
    const activePanel = hashPanel && [...document.querySelectorAll(".zr-panel-view.is-active")]
      .some((view) => view.dataset.panelView === hashPanel)
      ? hashPanel
      : "";
    return JSON.stringify({ activeModule, hashModule, activePanel, hashPanel, hashAction });
  })()`;
}

function cssEscape(value) {
  return String(value).replace(/["\\]/g, "\\$&");
}

function actionSelector(action) {
  const leaf = actionRouteKey(action).replace(/-/g, "_");
  return `[data-action="${cssEscape(action)}"], [data-action$=".${cssEscape(leaf)}"]`;
}

function scopedActionSelector(scope, action) {
  return actionSelector(action)
    .split(", ")
    .map((selector) => `${scope} ${selector}`)
    .join(", ");
}

function commandMatches(actual, expected) {
  if (!expected) return actual === "";
  if (actual === expected) return true;
  return actionRouteKey(actual) === actionRouteKey(expected);
}

async function waitForJson(url, attempts = 80) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      const response = await fetch(url);
      if (response.ok) return response.json();
    } catch (_) {
      // The debug endpoint takes a moment to open.
    }
    await delay(100);
  }
  throw new Error(`Timed out waiting for ${url}.`);
}

function connect(wsUrl) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(wsUrl);
    const pending = new Map();
    ws.addEventListener("open", () => {
      resolve({
        send(method, params = {}) {
          const id = nextId;
          nextId += 1;
          ws.send(JSON.stringify({ id, method, params }));
          return new Promise((resolveSend, rejectSend) => {
            pending.set(id, { method, resolveSend, rejectSend });
          });
        },
        close() {
          ws.close();
        },
      });
    });
    ws.addEventListener("message", (event) => {
      const message = JSON.parse(event.data);
      if (!message.id || !pending.has(message.id)) return;
      const request = pending.get(message.id);
      pending.delete(message.id);
      if (message.error) {
        request.rejectSend(new Error(`${request.method}: ${message.error.message}`));
      } else {
        request.resolveSend(message.result);
      }
    });
    ws.addEventListener("error", reject);
  });
}

async function evaluate(cdp, expression) {
  const result = await cdp.send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  if (result.exceptionDetails) {
    throw new Error(result.exceptionDetails.text ?? "Runtime.evaluate exception.");
  }
  return result.result.value;
}

async function cleanup() {
  await terminateBrowser();
  assertSafeTemporaryProfile(profile);
  await removeTemporaryProfile(profile);
}

async function terminateBrowser() {
  if (process.platform === "win32") {
    if (browser.exitCode === null && browser.signalCode === null && browser.pid) {
      await taskkillProcessTree(browser.pid);
    }
    await stopWindowsEdgeProfileProcesses(profile);
  } else if (browser.exitCode === null && browser.signalCode === null) {
    browser.kill();
  }
  await waitForExit(browser);
}

async function taskkillProcessTree(pid) {
  await new Promise((resolveKill) => {
    const killer = spawn("taskkill", ["/pid", String(pid), "/T", "/F"], { stdio: "ignore" });
    const timer = setTimeout(resolveKill, 3000);
    killer.once("exit", () => {
      clearTimeout(timer);
      resolveKill();
    });
    killer.once("error", () => {
      clearTimeout(timer);
      browser.kill();
      resolveKill();
    });
  });
}

async function stopWindowsEdgeProfileProcesses(profilePath) {
  const escapedProfile = profilePath.replaceAll("'", "''");
  const command = [
    `$profile = '${escapedProfile}'`,
    "Get-CimInstance Win32_Process |",
    "Where-Object { $_.Name -like 'msedge*.exe' -and $_.CommandLine -and $_.CommandLine.Contains($profile) } |",
    "ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }",
  ].join(" ");
  await runProcess("powershell.exe", ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", command]);
  await delay(500);
}

function waitForExit(child) {
  if (child.exitCode !== null || child.signalCode !== null) return Promise.resolve();
  return new Promise((resolveExit) => child.once("exit", () => resolveExit()));
}

function runProcess(command, args) {
  return new Promise((resolveRun) => {
    const child = spawn(command, args, { stdio: "ignore" });
    child.once("exit", () => resolveRun());
    child.once("error", () => resolveRun());
  });
}

function assertSafeTemporaryProfile(profilePath) {
  const resolvedProfile = resolve(profilePath);
  const resolvedTmp = resolve(tmpdir());
  const tmpWithSeparator = resolvedTmp.endsWith(sep) ? resolvedTmp : `${resolvedTmp}${sep}`;
  if (!resolvedProfile.startsWith(tmpWithSeparator) || !resolvedProfile.includes("zircon-workbench-responsive-cdp-")) {
    throw new Error(`Refusing to remove unexpected temporary profile path: ${profilePath}`);
  }
}

async function removeTemporaryProfile(profilePath, attempts = 40) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      rmSync(profilePath, { recursive: true, force: true });
      return;
    } catch (error) {
      if (!["EBUSY", "EPERM", "EACCES"].includes(error.code) || attempt === attempts - 1) {
        if (attempt === attempts - 1) {
          scheduleTemporaryProfileRemoval(profilePath);
          return;
        }
        throw error;
      }
      await delay(250 + attempt * 25);
    }
  }
}

function scheduleTemporaryProfileRemoval(profilePath) {
  const retryPath = `${profilePath}.delete-${Date.now()}`;
  try {
    renameSync(profilePath, retryPath);
  } catch (_) {
    // Leave the locked profile in temp; the next OS cleanup can remove it.
  }
  const targetPath = existsSync(retryPath) ? retryPath : profilePath;
  console.warn(`warning: deferred temporary browser profile cleanup for ${targetPath}`);
  const command = [
    "Start-Sleep -Seconds 3;",
    "Remove-Item -LiteralPath $args[0] -Recurse -Force -ErrorAction SilentlyContinue",
  ].join(" ");
  spawn("powershell.exe", ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", command, targetPath], {
    detached: true,
    stdio: "ignore",
    windowsHide: true,
  }).unref();
}

function delay(ms) {
  return new Promise((resolveDelay) => setTimeout(resolveDelay, ms));
}
