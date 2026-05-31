import { spawn } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { DASHBOARD_PAGE_ID, EXPORTS_LIST } from "./page-registry.mjs";

const edgeCandidates = [
  "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
  "C:/Program Files/Microsoft/Edge/Application/msedge.exe",
];
const edge = edgeCandidates.find((candidate) => existsSync(candidate));
if (!edge) {
  throw new Error("Microsoft Edge executable not found.");
}

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "../../..");
const referenceUrl = pathToFileURL(resolve(here, "index.html")).href;
const port = Number.parseInt(
  process.env.ZIRCON_HUB_WEB_REFERENCE_CDP_PORT ?? String(10433 + Math.floor(Math.random() * 500)),
  10,
);
const profile = mkdtempSync(join(tmpdir(), "zircon-hub-responsive-cdp-"));
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

const pages = [DASHBOARD_PAGE_ID, ...EXPORTS_LIST.map(([pageId]) => pageId)];

const viewports = [
  ["reference", 1568, 1003],
  ["large-preview", 1920, 1080],
  ["layout-review-crop", 1915, 508],
  ["wide", 1600, 1024],
  ["desktop", 1280, 900],
  ["compact", 1024, 720],
  ["narrow", 900, 720],
  ["small", 760, 680],
  ["minimum", 640, 640],
];

const resizePages = [
  DASHBOARD_PAGE_ID,
  "hub-projects-browser",
  "hub-projects-detail",
  "hub-projects-new",
  "hub-assets",
  "hub-settings",
  "hub-state-error",
];

const resizeSequence = [
  ["resize-reference", 1568, 1003],
  ["resize-compact", 1024, 720],
  ["resize-small", 760, 680],
  ["resize-minimum", 640, 640],
  ["resize-return", 1280, 900],
];

let nextId = 1;

try {
  validateRuntimeDependencies();
  validateLayoutModel();

  const list = await waitForJson(`http://127.0.0.1:${port}/json/list`);
  const target = list.find((item) => item.type === "page") ?? list[0];
  const cdp = await connect(target.webSocketDebuggerUrl);
  await cdp.send("Page.enable");
  await cdp.send("Runtime.enable");
  await cdp.send("DOM.enable");

  const failures = [];
  for (const [viewportName, width, height] of viewports) {
    await cdp.send("Emulation.setDeviceMetricsOverride", {
      width,
      height,
      deviceScaleFactor: 1,
      mobile: false,
    });
    for (const pageId of pages) {
      await cdp.send("Page.navigate", { url: `${referenceUrl}?page=${encodeURIComponent(pageId)}` });
      await waitForPage(cdp, pageId);
      const state = JSON.parse(await evaluate(cdp, responsiveAuditExpression(width, height)));
      if (!state.ok) {
        failures.push(`${viewportName} ${width}x${height} ${pageId}: ${state.failures.join("; ")}`);
      }
    }
  }

  for (const pageId of resizePages) {
    await cdp.send("Emulation.setDeviceMetricsOverride", {
      width: 1568,
      height: 1003,
      deviceScaleFactor: 1,
      mobile: false,
    });
    await cdp.send("Page.navigate", { url: `${referenceUrl}?page=${encodeURIComponent(pageId)}` });
    await waitForPage(cdp, pageId);
    for (const [viewportName, width, height] of resizeSequence) {
      await cdp.send("Emulation.setDeviceMetricsOverride", {
        width,
        height,
        deviceScaleFactor: 1,
        mobile: false,
      });
      await delay(120);
      await waitForPage(cdp, pageId);
      const state = JSON.parse(await evaluate(cdp, responsiveAuditExpression(width, height)));
      if (!state.ok) {
        failures.push(`${viewportName} ${width}x${height} ${pageId}: ${state.failures.join("; ")}`);
      }
    }
  }

  cdp.close();
  if (failures.length > 0) {
    throw new Error(`Hub web-reference responsive audit failed:\n${failures.join("\n")}`);
  }
  console.log(`validated ${pages.length} Hub web-reference pages across ${viewports.length} responsive viewports`);
  console.log(`validated ${resizePages.length} representative Hub pages through ${resizeSequence.length} live resize steps`);
  console.log("validated local-only runtime assets, viewport-full shell geometry, and no UI screenshot/AI-draft reuse in the browser source");
} finally {
  await cleanup();
}

function validateRuntimeDependencies() {
  const html = readFileSync(resolve(here, "index.html"), "utf8");
  for (const requiredLocalFile of [
    "styles.css",
    "covers.css",
    "responsive.css",
    "fullscreen-preview.css",
    "interaction-states.css",
    "cover-rendering.js",
    "app.js",
    "interaction-enhancements.js",
  ]) {
    if (!html.includes(requiredLocalFile)) {
      throw new Error(`index.html must load local runtime file ${requiredLocalFile}.`);
    }
  }

  const externalRefs = [...html.matchAll(/\b(?:src|href)=["']([^"']+)["']/g)]
    .map((match) => match[1])
    .filter((ref) => /^(?:https?:)?\/\//i.test(ref));
  if (externalRefs.length > 0) {
    throw new Error(`index.html must not load external runtime assets: ${externalRefs.join(", ")}`);
  }

  for (const forbidden of ["react", "vue", "jquery", "bootstrap", "tailwind", "cdn"]) {
    if (html.toLowerCase().includes(forbidden)) {
      throw new Error(`index.html must not depend on ${forbidden}.`);
    }
  }

  const coverRendererSource = readFileSync(resolve(here, "cover-rendering.js"), "utf8");
  for (const requiredCover of [
    "project-elysium.png",
    "project-stellar-outpost.png",
    "project-sands-of-time.png",
    "project-whispering-woods.png",
    "project-neon-streets.png",
  ]) {
    if (!coverRendererSource.includes(requiredCover)) {
      throw new Error(`cover-rendering.js must map real reference project cover ${requiredCover}.`);
    }
  }

  const coverSource = readFileSync(resolve(here, "covers.css"), "utf8");
  for (const snippet of [".project-cover-image", "object-fit: cover", "filter: saturate"]) {
    if (!coverSource.includes(snippet)) {
      throw new Error(`covers.css must style real project-cover image previews: ${snippet}`);
    }
  }
}

function validateLayoutModel() {
  const source = readFileSync(resolve(here, "LAYOUT_MODEL.md"), "utf8");
  for (const snippet of [
    "The final reference PNGs are export artifacts",
    "No CDN, framework bundle, component library, or third-party runtime is used.",
    "The page does not use `hub-ai-drafts/*`, `hub-*.png`, or",
    "Project cover art uses real local reference images",
    "| `.hub-shell` | Root window grid |",
    "| `.workspace` | Scrollable page slot |",
    "| `max-width: 1180px` or `max-height: 860px` | Compact desktop |",
    "`640x640` validation floor",
    "`validate-responsive.mjs` checks all browser-openable pages across `1568x1003`",
    "five-step live resize sequence without reloading",
  ]) {
    if (!source.includes(snippet)) {
      throw new Error(`LAYOUT_MODEL.md is missing required responsive migration text: ${snippet}`);
    }
  }
}

function responsiveAuditExpression(width, height) {
  return `(() => {
    const width = ${width};
    const height = ${height};
    const failures = [];
    const shell = document.querySelector(".hub-shell");
    const workspace = document.querySelector(".workspace");
    const topbar = document.querySelector(".topbar");
    const sidebar = document.querySelector(".sidebar");
    const heading = document.querySelector(".page-heading h2");
    if (!shell || !workspace || !topbar || !sidebar || !heading) {
      return JSON.stringify({ ok: false, failures: ["missing shell, workspace, topbar, sidebar, or page heading"] });
    }

    const shellRect = shell.getBoundingClientRect();
    const workspaceRect = workspace.getBoundingClientRect();
    const topbarRect = topbar.getBoundingClientRect();
    const sidebarRect = sidebar.getBoundingClientRect();
    const pageId = shell.dataset.page;
    const projectCoverPages = new Set([
      "projects-dashboard",
      "hub-projects-browser",
      "hub-projects-browser-filter-menu",
      "hub-projects-browser-sort-menu",
      "hub-projects-detail",
      "hub-projects-detail-delete-confirm"
    ]);
    const bodyScrollWidth = document.scrollingElement.scrollWidth;
    const bodyScrollHeight = document.scrollingElement.scrollHeight;

    if (Math.abs(shellRect.width - width) > 1) failures.push("shell does not fill viewport width");
    if (Math.abs(shellRect.height - height) > 1) failures.push("shell does not fill viewport height");
    if (bodyScrollWidth > width + 1) failures.push("document has horizontal overflow");
    if (bodyScrollHeight > height + 1) failures.push("document scrolls outside the app shell");
    if (workspaceRect.width < 360) failures.push("workspace collapsed below 360px");
    if (workspaceRect.height < 420 && height >= 720) failures.push("workspace collapsed below useful height");
    if (topbarRect.right > width + 1 || topbarRect.left < -1) failures.push("topbar escapes viewport");
    if (sidebarRect.bottom > height + 1) failures.push("sidebar escapes viewport height");
    if (heading.getBoundingClientRect().right > width + 1) failures.push("page heading escapes viewport");
    if (pageId === "projects-dashboard" && width > 1568) {
      const cards = document.querySelector(".project-cards");
      if (cards) {
        const cardsRect = cards.getBoundingClientRect();
        if (workspaceRect.right - cardsRect.right > 42) {
          failures.push("project cards do not expand across the fullscreen preview width");
        }
      }
    }
    if (pageId.startsWith("hub-projects-browser") && width > 980) {
      const browserTable = document.querySelector(".browser-table");
      const browserHead = document.querySelector(".browser-head");
      const browserRows = [...document.querySelectorAll(".browser-row")];
      const selectedRow = document.querySelector(".browser-row.selected");
      if (!browserTable || !browserHead || browserRows.length === 0 || !selectedRow) {
        failures.push("browser table is missing a table, header, selected row, or row");
      } else {
        const tableRect = browserTable.getBoundingClientRect();
        const headRect = browserHead.getBoundingClientRect();
        const selectedRect = selectedRow.getBoundingClientRect();
        if (Math.abs(tableRect.width - headRect.width) > 2) {
          failures.push("browser header does not fill the table width");
        }
        if (Math.abs(headRect.left - selectedRect.left) > 2 || Math.abs(headRect.right - selectedRect.right) > 2) {
          failures.push("browser selected-row highlight does not span the header width");
        }
        const headCells = [...browserHead.children].map((node) => node.getBoundingClientRect().left);
        const headWidths = [...browserHead.children].map((node) => node.getBoundingClientRect().width);
        for (const [rowIndex, browserRow] of browserRows.entries()) {
          const rowRect = browserRow.getBoundingClientRect();
          if (Math.abs(headRect.width - rowRect.width) > 2) {
            failures.push("browser row " + rowIndex + " does not fill the header width");
            break;
          }
          if (Math.abs(headRect.left - rowRect.left) > 2 || Math.abs(headRect.right - rowRect.right) > 2) {
            failures.push("browser row " + rowIndex + " edges are not aligned with the header edges");
            break;
          }
          const rowCells = [...browserRow.children].map((node) => node.getBoundingClientRect().left);
          const rowWidths = [...browserRow.children].map((node) => node.getBoundingClientRect().width);
          for (let index = 1; index < Math.min(headCells.length, rowCells.length); index += 1) {
            if (Math.abs(headCells[index] - rowCells[index]) > 3) {
              failures.push("browser row " + rowIndex + " column " + index + " is not aligned with the header");
              break;
            }
            if (Math.abs(headWidths[index] - rowWidths[index]) > 3) {
              failures.push("browser row " + rowIndex + " column " + index + " width does not match the header");
              break;
            }
          }
          if (failures.some((failure) => failure.startsWith("browser row " + rowIndex + " column "))) {
            break;
          }
        }
      }
    }

    const runtimeImages = [...document.images].map((image) => image.getAttribute("src") || "");
    const forbiddenImages = runtimeImages.filter((src) =>
      src.includes("hub-ai-drafts") ||
      /docs\\/ui-and-layout\\/hub(?:-[^/]+)?\\.png/i.test(src) ||
      src.includes("hub-web-reference-1568x1003.png")
    );
    if (forbiddenImages.length > 0) failures.push("runtime reuses Hub draft/export PNGs: " + forbiddenImages.join(", "));
    if (projectCoverPages.has(pageId) && document.querySelectorAll(".project-cover-image").length === 0) {
      failures.push("project page is missing real reference project cover images");
    }

    const visibleOutliers = [...document.querySelectorAll("body *")].flatMap((node) => {
      const style = getComputedStyle(node);
      if (style.display === "none" || style.visibility === "hidden" || Number(style.opacity) === 0) return [];
      const rect = node.getBoundingClientRect();
      if (rect.width < 2 || rect.height < 2) return [];
      if (rect.left < -3 || rect.right > width + 3) {
        const label = node.className ? "." + String(node.className).trim().replace(/\\s+/g, ".") : node.tagName.toLowerCase();
        return [label + " [" + Math.round(rect.left) + "," + Math.round(rect.right) + "]"];
      }
      return [];
    }).slice(0, 8);
    if (visibleOutliers.length > 0) failures.push("visible horizontal outliers: " + visibleOutliers.join(", "));

    return JSON.stringify({ ok: failures.length === 0, pageId, failures });
  })()`;
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForPage(cdp, pageId, attempts = 80) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    const state = JSON.parse(
      await evaluate(
        cdp,
        "JSON.stringify({ page: document.querySelector('.hub-shell')?.dataset.page, title: document.title, heading: document.querySelector('.page-heading h2')?.innerText })",
      ),
    );
    if (state.page === pageId && state.title?.endsWith(" - Zircon Hub Web Reference") && state.heading) {
      return;
    }
    await delay(100);
  }
  throw new Error(`Timed out waiting for Hub web-reference page ${pageId}.`);
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
  for (let attempt = 0; attempt < 8; attempt += 1) {
    try {
      rmSync(profile, { recursive: true, force: true });
      if (!existsSync(profile)) return;
    } catch (error) {
      if (attempt === 7) {
        console.warn(`Temporary Edge profile cleanup skipped: ${profile}`);
        return;
      }
    }
    await delay(250);
  }
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
  if (!resolvedProfile.startsWith(tmpWithSeparator) || !resolvedProfile.includes("zircon-hub-responsive-cdp-")) {
    throw new Error(`Refusing to remove unexpected temporary profile path: ${profilePath}`);
  }
}
