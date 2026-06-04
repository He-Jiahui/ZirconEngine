import { spawn } from "node:child_process";
import { existsSync, mkdirSync, renameSync, rmSync } from "node:fs";
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
const port = Number.parseInt(process.env.ZIRCON_WORKBENCH_ROUTE_CDP_PORT ?? String(12540 + Math.floor(Math.random() * 500)), 10);
const profile = resolve(tmpdir(), `zircon-workbench-route-cdp-${process.pid}-${Date.now()}`);
const expectedExtensionCards = extensionModules.length;
const expectedTopLevelModuleTabs = webModuleTabs.length;
const routeAuditTimeoutMs = Number.parseInt(process.env.ZIRCON_WORKBENCH_ROUTE_AUDIT_TIMEOUT_MS ?? "480000", 10);
let nextId = 1;

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

try {
  const list = await waitForJson(`http://127.0.0.1:${port}/json/list`);
  const target = list.find((item) => item.type === "page") ?? list[0];
  const cdp = await connect(target.webSocketDebuggerUrl);
  await cdp.send("Page.enable");
  await cdp.send("Runtime.enable");
  await cdp.send("Emulation.setDeviceMetricsOverride", {
    width: 1672,
    height: 941,
    deviceScaleFactor: 1,
    mobile: false,
  });
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);

  const state = JSON.parse(await evaluate(cdp, controlRouteAuditExpression()));
  cdp.close();
  if (!state.ok) {
    throw new Error(`Workbench control route audit failed:\n${state.failures.join("\n")}`);
  }
  console.log(`validated control route audit across ${state.modules} top-level modules`);
  console.log(`validated control route audit across ${state.extensions} extension modules`);
  console.log(`validated ${state.controls} visible control route responses`);
} finally {
  await cleanup();
}

function controlRouteAuditExpression() {
  return `(async () => {
    const failures = [];
    const routeWrites = [];
    const audits = [];
    const auditDeadlineAt = performance.now() + ${routeAuditTimeoutMs};
    let captureHistoryWrites = true;
    let baselineCounter = 0;
    const originalPushState = history.pushState.bind(history);
    const originalReplaceState = history.replaceState.bind(history);
    let moduleCount = 0;
    let extensionCount = 0;
    history.pushState = (state, title, url) => {
      if (captureHistoryWrites) {
        routeWrites.push({ mode: "push", url: String(url || "") });
      }
      return undefined;
    };
    history.replaceState = (state, title, url) => {
      if (captureHistoryWrites) {
        routeWrites.push({ mode: "replace", url: String(url || "") });
      }
      return undefined;
    };

    const settle = () => Promise.resolve();
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
      const recentAudits = audits.slice(-8).map((audit) => audit.context + ":" + audit.count).join(", ");
      throw new Error("route audit deadline exceeded at " + context + "; recent audits: " + recentAudits);
    };
    const clickForRestore = async (selector, context) => {
      const node = document.querySelector(selector);
      if (!node) {
        failures.push("restore missing " + context);
        return false;
      }
      node.click();
      await settle();
      return true;
    };
    const routeBaseline = async (moduleId, panelTarget = "") => {
      baselineCounter += 1;
      if (document.querySelector('.zr-module-tab[data-module="' + attrEscape(moduleId) + '"]')) {
        await clickForRestore('.zr-module-tab[data-module="' + attrEscape(moduleId) + '"]', "module tab " + moduleId);
      } else {
        await clickForRestore('.zr-module-tab[data-module="editor-library"]', "extension library tab");
        await clickForRestore('[data-module-source="extension-library"][data-module="' + attrEscape(moduleId) + '"]', "extension card " + moduleId);
      }
      if (panelTarget) {
        await clickForRestore('[data-panel-tab="' + attrEscape(panelTarget) + '"]', "panel tab " + panelTarget);
      }
      const baselineParams = new URLSearchParams();
      baselineParams.set("module", moduleId);
      if (panelTarget) baselineParams.set("panel", panelTarget);
      baselineParams.set("command", "route-audit-baseline-" + baselineCounter);
      originalReplaceState(
        history.state,
        "",
        location.pathname + location.search + "#" + baselineParams.toString()
      );
      routeWrites.length = 0;
      if (activeModule() !== moduleId) failures.push("route baseline did not activate module " + moduleId);
      if (panelTarget) {
        const activePanel = document.querySelector('.zr-panel-view.is-active[data-panel-view="' + attrEscape(panelTarget) + '"]');
        if (!activePanel) failures.push("route baseline did not activate panel " + panelTarget);
      }
    };
    const visible = (node) => {
      if (!node || node.disabled) return false;
      if (node.closest(".zr-panel-view:not(.is-active)")) return false;
      if (node.closest(".zr-popup-layer:not(.is-open)")) return false;
      const style = getComputedStyle(node);
      const rect = node.getBoundingClientRect();
      return style.display !== "none" && style.visibility !== "hidden" && Number(style.opacity) !== 0 && rect.width > 1 && rect.height > 1;
    };
    const controls = (selector) => [...document.querySelectorAll(selector)].filter(visible);
    const labelFor = (node) => {
      const explicit = node.dataset.action || node.dataset.module || node.dataset.panelTab || node.dataset.dropdown || node.dataset.treeRow || "";
      const fieldLabel = node.getAttribute("placeholder") || node.value || node.closest(".zr-module-setting")?.querySelector("span")?.textContent.trim() || "";
      const label = explicit || node.getAttribute("aria-label") || node.getAttribute("title") || fieldLabel || node.textContent.trim().replace(/\\s+/g, " ");
      return (label || node.tagName.toLowerCase()).replace(/\\s+/g, " ").slice(0, 96);
    };
    const exerciseControl = async (node, context) => {
      checkDeadline(context);
      if (!node) {
        failures.push("control disappeared before exercise for " + context);
        return false;
      }
      const beforeModule = activeModule();
      const beforePanel = activePanel();
      const before = responseCount();
      const beforeHash = location.hash;
      const label = labelFor(node);
      const expectedModule = node.dataset?.module || "";
      const expectedPanel = node.dataset?.panelTab || "";
      routeWrites.length = 0;
      if (node.matches?.("input:not([disabled]), textarea:not([disabled])")) {
        node.focus();
        await settle();
        if (routeWrites.length === 0) {
          node.value = (node.value || "") + " route";
          node.dispatchEvent(new Event("input", { bubbles: true }));
          await settle();
        }
      } else {
        node.click();
        await settle();
      }
      if (responseCount() <= before) failures.push("no response after " + context + ": " + label);
      const reachedExpectedModule = expectedModule && activeModule() === expectedModule;
      const reachedExpectedPanel = expectedPanel && Boolean(document.querySelector('.zr-panel-view.is-active[data-panel-view="' + attrEscape(expectedPanel) + '"]'));
      if (routeWrites.length === 0 && location.hash === beforeHash && !reachedExpectedModule && !reachedExpectedPanel) {
        failures.push("no route-state write after " + context + ": " + label);
      }
      return beforeModule !== activeModule() || beforePanel !== activePanel() || !document.contains(node);
    };
    const auditIndexedControls = async (selector, context, restore) => {
      await restore();
      const available = controls(selector).length;
      if (available === 0) {
        failures.push("no controls found for " + context);
        return 0;
      }
      let needsRestore = false;
      for (let index = 0; index < available; index += 1) {
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
        needsRestore = await exerciseControl(node, context + " #" + (index + 1) + "/" + available);
      }
      audits.push({ context, count: available });
      return available;
    };
    const activePanelTargets = (selector) => [...document.querySelectorAll(selector)]
      .filter(visible)
      .map((tab) => tab.dataset.panelTab)
      .filter(Boolean);
    const openPopup = async () => {
      await routeBaseline("gameplay-effect");
      const dropdown = controls("[data-dropdown]")[0];
      if (!dropdown) {
        failures.push("no dropdown available for popup menu rows");
        return false;
      }
      dropdown.click();
      await settle();
      routeWrites.length = 0;
      if (!document.querySelector(".zr-popup-layer")?.classList.contains("is-open")) {
        failures.push("popup menu rows did not open");
        return false;
      }
      return true;
    };

    try {
      await routeBaseline("gameplay-effect");
      const moduleIds = controls(".zr-module-tab[data-module]").map((button) => button.dataset.module);
      moduleCount = moduleIds.length;
      if (moduleIds.length !== ${expectedTopLevelModuleTabs}) failures.push("expected ${expectedTopLevelModuleTabs} top-level module tabs, found " + moduleIds.length);
      await auditIndexedControls(".zr-module-tab[data-module]", "top-level module tabs", async () => routeBaseline("gameplay-effect"));
      await auditIndexedControls(".zr-rail button:not([disabled])", "rail module buttons", async () => routeBaseline("gameplay-effect"));

      await routeBaseline("editor-library");
      const extensionIds = controls('[data-module-source="extension-library"][data-module]').map((button) => button.dataset.module);
      extensionCount = extensionIds.length;
      if (extensionIds.length !== ${expectedExtensionCards}) failures.push("expected ${expectedExtensionCards} extension editor cards, found " + extensionIds.length);
      await auditIndexedControls('[data-module-source="extension-library"][data-module]', "extension editor cards", async () => routeBaseline("editor-library"));

      await auditIndexedControls(
        ".zr-topbar > .zr-topbar-group:first-child button:not([disabled]), .zr-topbar > .zr-topbar-group:last-child button:not([disabled]), .zr-statusbar button:not([disabled])",
        "global toolbar and status buttons",
        async () => routeBaseline("gameplay-effect")
      );

      for (const id of moduleIds) {
        await auditIndexedControls(".zr-module-toolbar button:not([disabled])", "module toolbar " + id, async () => routeBaseline(id));
        await auditIndexedControls(
          ".zr-module-left button:not([disabled]), .zr-module-left input:not([disabled]), .zr-module-main button:not([disabled]), .zr-module-main [role='button'], .zr-module-main input:not([disabled])",
          "module primary surfaces " + id,
          async () => routeBaseline(id)
        );

        await routeBaseline(id);
        for (const target of activePanelTargets(".zr-module-right .zr-panel-tab")) {
          await auditIndexedControls(
            ".zr-module-right .zr-panel-tab:not([disabled]), .zr-module-right .zr-panel-view.is-active button:not([disabled]), .zr-module-right .zr-panel-view.is-active [role='button'], .zr-module-right .zr-panel-view.is-active input:not([disabled])",
            "module right panel controls " + target,
            async () => routeBaseline(id, target)
          );
        }
        await routeBaseline(id);
        for (const target of activePanelTargets(".zr-module-bottom .zr-panel-tab")) {
          await auditIndexedControls(
            ".zr-module-bottom .zr-panel-tab:not([disabled]), .zr-module-bottom .zr-panel-view.is-active button:not([disabled]), .zr-module-bottom .zr-panel-view.is-active [role='button'], .zr-module-bottom .zr-panel-view.is-active input:not([disabled])",
            "module bottom panel controls " + target,
            async () => routeBaseline(id, target)
          );
        }
      }

      for (const id of extensionIds) {
        await routeBaseline(id);
        if (!document.querySelector('.zr-module-main[data-module-active="' + attrEscape(id) + '"] .zr-module-editor-grid[data-extension-blueprint="reference"]')) {
          failures.push("extension editor did not render reference blueprint: " + id);
        }
        await auditIndexedControls(".zr-module-toolbar button:not([disabled])", "all extension toolbar controls " + id, async () => routeBaseline(id));
        await auditIndexedControls(
          ".zr-module-left button:not([disabled]), .zr-module-left input:not([disabled]), .zr-module-main button:not([disabled]), .zr-module-main [role='button'], .zr-module-main input:not([disabled])",
          "all extension primary surface controls " + id,
          async () => routeBaseline(id)
        );

        await routeBaseline(id);
        for (const target of activePanelTargets(".zr-module-right .zr-panel-tab")) {
          await auditIndexedControls(
            ".zr-module-right .zr-panel-tab:not([disabled]), .zr-module-right .zr-panel-view.is-active button:not([disabled]), .zr-module-right .zr-panel-view.is-active [role='button'], .zr-module-right .zr-panel-view.is-active input:not([disabled])",
            "all extension right panel controls " + target,
            async () => routeBaseline(id, target)
          );
        }
        await routeBaseline(id);
        for (const target of activePanelTargets(".zr-module-bottom .zr-panel-tab")) {
          await auditIndexedControls(
            ".zr-module-bottom .zr-panel-tab:not([disabled]), .zr-module-bottom .zr-panel-view.is-active button:not([disabled]), .zr-module-bottom .zr-panel-view.is-active [role='button'], .zr-module-bottom .zr-panel-view.is-active input:not([disabled])",
            "all extension bottom panel controls " + target,
            async () => routeBaseline(id, target)
          );
        }
      }

      if (await openPopup()) {
        const popupRows = controls(".zr-popup-layer [data-menu-item][data-action]");
        if (popupRows.length === 0) failures.push("no popup menu rows found");
        for (let index = 0; index < popupRows.length; index += 1) {
          await openPopup();
          const list = controls(".zr-popup-layer [data-menu-item][data-action]");
          await exerciseControl(list[index], "popup menu rows #" + (index + 1) + "/" + popupRows.length);
          if (document.querySelector(".zr-popup-layer")?.classList.contains("is-open")) {
            failures.push("popup menu rows did not close after selection #" + (index + 1));
          }
        }
        audits.push({ context: "popup menu rows", count: popupRows.length });
      }
    } catch (error) {
      failures.push(error?.message ?? String(error));
    } finally {
      captureHistoryWrites = false;
      history.pushState = originalPushState;
      history.replaceState = originalReplaceState;
    }

    const controlsAudited = audits.reduce((total, audit) => total + audit.count, 0);
    if (controlsAudited < 200) failures.push("control route audit covered too few controls: " + controlsAudited);
    return JSON.stringify({
      ok: failures.length === 0,
      failures,
      modules: moduleCount,
      extensions: extensionCount,
      controls: controlsAudited,
      audits
    });
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
    throw new Error(result.exceptionDetails.exception?.description ?? result.exceptionDetails.text ?? "Runtime.evaluate exception.");
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
  if (!resolvedProfile.startsWith(tmpWithSeparator) || !resolvedProfile.includes("zircon-workbench-route-cdp-")) {
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
