import { spawn } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

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
  await cdp.send("Page.navigate", { url: referenceUrl });
  await waitForWorkbench(cdp);
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
  for (const required of ["tokens.css", "atoms.css", "collections.css", "surfaces.css", "workbench.css", "responsive.css", "app.js"]) {
    if (!html.includes(required)) {
      throw new Error(`index.html must load ${required}.`);
    }
  }
  if (/https?:\/\//i.test(html)) {
    throw new Error("index.html must not load external resources.");
  }
  const sources = ["app.js", "atoms.js", "collections.js", "surfaces.js", "icons.js"].map((file) => readFileSync(resolve(here, file), "utf8")).join("\n");
  if (sources.includes("workbench.png")) {
    throw new Error("Component prototype must not embed the full workbench reference screenshot.");
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
    const showcase = document.querySelector(".zr-showcase");
    const statusbar = document.querySelector(".zr-statusbar");
    if (!app || !windowNode || !topbar || !rail || !viewport || !showcase || !statusbar) {
      return JSON.stringify({ ok: false, failures: ["missing core workbench regions"] });
    }

    const appRect = app.getBoundingClientRect();
    const topbarRect = topbar.getBoundingClientRect();
    const railRect = rail.getBoundingClientRect();
    const viewportRect = viewport.getBoundingClientRect();
    const showcaseRect = showcase.getBoundingClientRect();
    const statusbarRect = statusbar.getBoundingClientRect();
    const scroll = document.scrollingElement;

    if (Math.ceil(appRect.width) > width + 1) failures.push("app wider than viewport");
    if (Math.ceil(topbarRect.width) > width + 1) failures.push("topbar wider than viewport");
    if (topbarRect.top < -1 || topbarRect.bottom > Math.max(height, scroll.clientHeight) + 1) failures.push("topbar escapes visible shell");
    if (railRect.left < -1 || railRect.right > width + 1) failures.push("rail escapes viewport");
    if (viewportRect.width < 220) failures.push("viewport collapsed below 220px");
    if (showcaseRect.width < 220) failures.push("component drawer collapsed below 220px");
    if (statusbarRect.left < -1 || statusbarRect.right > width + 1) failures.push("statusbar escapes viewport width");
    if (scroll.scrollWidth > Math.max(width, 640) + 1) failures.push("document horizontal overflow exceeds responsive floor");

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

async function waitForWorkbench(cdp, attempts = 80) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    const ready = await evaluate(cdp, "Boolean(document.querySelector('.zr-window') && document.querySelector('.zr-viewport-reference')?.complete)");
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
    throw new Error(result.exceptionDetails.text ?? "Runtime.evaluate exception.");
  }
  return result.result.value;
}

async function cleanup() {
  await terminateBrowser();
  assertSafeTemporaryProfile(profile);
  rmSync(profile, { recursive: true, force: true });
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

function delay(ms) {
  return new Promise((resolveDelay) => setTimeout(resolveDelay, ms));
}
