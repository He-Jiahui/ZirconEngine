import { spawn } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { extensionModules, webModuleTabs } from "./modules.js";

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
  for (const required of ["tokens.css", "layout.css", "atoms.css", "collections.css", "surfaces.css", "modules.css", "workbench.css", "responsive.css", "app.js"]) {
    if (!html.includes(required)) {
      throw new Error(`index.html must load ${required}.`);
    }
  }
  if (/https?:\/\//i.test(html)) {
    throw new Error("index.html must not load external resources.");
  }
  const sources = ["app.js", "atoms.js", "collections.js", "surfaces.js", "modules.js", "routes.js", "icons.js"].map((file) => readFileSync(resolve(here, file), "utf8")).join("\n");
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
      history.replaceState(history.state, "", window.location.pathname + window.location.search + "#module=hud-editor&command=tree-canvas-panel");
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
    const exerciseControl = async (node, context) => (
      isEditableControl(node)
        ? editAndExpectResponse(node, context)
        : clickAndExpectResponse(node, context)
    );
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
      for (let index = 0; index < total; index += 1) {
        await restore();
        const list = controls(selector);
        const node = list[index];
        if (!node) {
          failures.push("control disappeared for " + context + " #" + (index + 1));
          continue;
        }
        await exerciseControl(node, context + " #" + (index + 1) + "/" + available);
      }
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
      document.querySelector('[data-action="browse"]')?.click();
      await settle();
      if (document.querySelector(".zr-module-main")?.dataset.moduleActive !== "asset-browser") failures.push("browse did not route to asset browser");
      await activateModule("gameplay-effect", "route compile restore");
      document.querySelector('[data-action="compile"]')?.click();
      await settle();
      const compilePanel = document.querySelector('[data-panel-view="module-bottom-gameplay-effect:compile-log"]');
      if (!compilePanel?.classList.contains("is-active")) failures.push("compile did not route to compile log");
      await activateModule("material", "route material restore");
      document.querySelector('[data-action="texture-sample"]')?.click();
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
    } finally {
      history.pushState = originalPushState;
      history.replaceState = originalReplaceState;
    }

    const interactionRouteWrites = capturedHistoryStates.length;
    if (interactionRouteWrites === 0) failures.push("interaction audit captured no route-state writes");
    return JSON.stringify({ ok: failures.length === 0, failures, interactionRouteWrites });
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
  await clickPanelTabAndWait(cdp, "asset-right:metadata", "asset-browser", "panel-asset-rightmetadata");
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
  await waitForHistoryState(cdp, "asset-browser", "asset-right:metadata", "panel-asset-rightmetadata");
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
  await clickActiveModuleActionAndWait(cdp, "compile-shader-editor", "shader-editor", "module-bottom-shader-editor:validation", "compile-shader-editor");
  await clickActiveModuleActionAndWait(cdp, "save-shader-editor", "shader-editor", "module-bottom-shader-editor:references", "save-shader-editor");
  await clickActiveModuleActionAndWait(cdp, "preview-shader-editor", "shader-editor", "module-bottom-shader-editor:output", "preview-shader-editor");
  await clickExtensionModuleAndWait(cdp, "source-control");
  await clickActiveModuleActionAndWait(cdp, "review-source-control", "source-control", "module-bottom-source-control:references", "review-source-control");
  await clickActiveModuleActionAndWait(cdp, "run-source-control", "source-control", "module-bottom-source-control:output", "run-source-control");
  await clickExtensionModuleAndWait(cdp, "weather-editor");
  await clickActiveModuleActionAndWait(cdp, "build-weather-editor", "weather-editor", "module-bottom-weather-editor:validation", "build-weather-editor");
  await clickActiveModuleActionAndWait(cdp, "preview-weather-editor", "weather-editor", "module-bottom-weather-editor:output", "preview-weather-editor");
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
        hashCommand: params.get("command") || ""
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
      if (state.hashCommand !== action) {
        failures.push("command mismatch after " + id + "/" + action + ": " + state.hashCommand);
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
    const extensionPanelKeyForToolbarCommand = (command) => {
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
        hashCommand: params.get("command") || ""
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
      return activeModule() === id;
    };
    const assertRoute = (id, action, expectedModule, expectedPanel) => {
      const state = routeState();
      if (
        state.activeModule !== expectedModule
        || state.hashModule !== expectedModule
        || state.activePanel !== expectedPanel
        || state.hashPanel !== expectedPanel
        || state.hashCommand !== action
      ) {
        failures.push(
          "toolbar route mismatch " + id + "/" + action
          + ": expected " + expectedModule + "/" + expectedPanel + "/" + action
          + ", got active=" + state.activeModule + "/" + state.activePanel
          + " hash=" + state.hashModule + "/" + state.hashPanel + "/" + state.hashCommand
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
      if (!uniqueActions.has("more-editors")) failures.push("missing More Editors toolbar route for " + id);
      if (uniqueActions.size !== actions.length) failures.push("duplicate toolbar action ids for " + id);

      for (const action of actions) {
        if (!await openExtension(id)) continue;
        const before = responseCount();
        if (!await click('.zr-module-toolbar [data-action="' + escapeCss(action) + '"]', "toolbar action " + id + "/" + action)) continue;
        if (responseCount() <= before) failures.push("no response after toolbar action " + id + "/" + action);
        if (action === "more-editors") {
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
  await clickTreeRowAndWait(cdp, "ge-healthregen", "gameplay-effect", "", "tree-ge-healthregen");
  await editFieldAndWait(cdp, ".zr-module-left input[placeholder='Search assets...']:not([disabled])", "gameplay-effect", "", "edit-search-assets");
}

async function assertKeyboardActivation(cdp) {
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
  await assertElementAttribute(
    cdp,
    '.zr-module-table-row[role="button"][data-action="healthregen"]',
    "aria-label",
    "HealthRegen",
  );
  await assertElementAttribute(
    cdp,
    '.zr-module-table-row[role="button"][data-action="incominghealing"]',
    "aria-label",
    "IncomingHealing",
  );
  await pressActionKeyAndWait(
    cdp,
    '.zr-module-table-row[role="button"][data-action="healthregen"]',
    "Enter",
    "gameplay-effect",
    "",
    "row-healthregen",
  );
  await pressActionKeyAndWait(
    cdp,
    '.zr-module-table-row[role="button"][data-action="incominghealing"]',
    " ",
    "gameplay-effect",
    "",
    "row-incominghealing",
  );
}

async function assertCollectionRowButtons(cdp) {
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
  await assertElementAttribute(
    cdp,
    '.zr-module-left .zr-module-list-row[data-action="target-tags"]',
    "type",
    "button",
  );
  await assertElementAttribute(
    cdp,
    '.zr-module-table-row[role="button"][data-action="healthregen"]',
    "aria-label",
    "HealthRegen",
  );
  await pressActionKeyAndWait(
    cdp,
    '.zr-module-left .zr-module-list-row[data-action="target-tags"]',
    "Enter",
    "gameplay-effect",
    "",
    "row-target-tags",
  );
  await pressActionKeyAndWait(
    cdp,
    '.zr-module-table-row[role="button"][data-action="healthregen"]',
    " ",
    "gameplay-effect",
    "",
    "row-healthregen",
  );
}

async function assertCollectionTreeRows(cdp) {
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
  const mounted = JSON.parse(await evaluate(cdp, `(async () => {
    const { treeView } = await import(new URL("./collections.js", location.href).href);
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
    rootAction: "contract-root",
    rootLabel: "Contract Root",
    childTag: "BUTTON",
    childType: "button",
    childAction: "contract-child",
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
  await waitForHistoryState(cdp, "gameplay-effect", "", "tree-contract-child");
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
  const selector = `[data-action="${cssEscape(action)}"]`;
  await cdp.send("Runtime.evaluate", {
    expression: `document.querySelector(${JSON.stringify(selector)})?.click()`,
    awaitPromise: true,
    returnByValue: true,
  });
  await waitForHistoryState(cdp, expectedModuleId, expectedPanelTarget, expectedCommandId, attempts);
}

async function clickActiveModuleActionAndWait(cdp, action, expectedModuleId, expectedPanelTarget = "", expectedCommandId = "", attempts = 80) {
  const selector = `.zr-module-toolbar [data-action="${cssEscape(action)}"]`;
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
      && state.hashCommand === expectedCommandId
    ) {
      return state;
    }
    await delay(100);
  }
  const state = JSON.parse(await evaluate(cdp, moduleHistoryStateExpression()));
  throw new Error(`Expected history state ${expectedModuleId}/${expectedPanelTarget}/${expectedCommandId}, got active=${state.activeModule}/${state.activePanel} hash=${state.hashModule}/${state.hashPanel}/${state.hashCommand}.`);
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
    const hashCommand = new URLSearchParams(location.hash.replace(/^#/, "")).get("command") || "";
    const activePanel = hashPanel && [...document.querySelectorAll(".zr-panel-view.is-active")]
      .some((view) => view.dataset.panelView === hashPanel)
      ? hashPanel
      : "";
    return JSON.stringify({ activeModule, hashModule, activePanel, hashPanel, hashCommand });
  })()`;
}

function cssEscape(value) {
  return String(value).replace(/["\\]/g, "\\$&");
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
