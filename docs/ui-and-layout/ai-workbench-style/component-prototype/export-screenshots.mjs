import { spawn } from "node:child_process";
import { existsSync, mkdirSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve, sep } from "node:path";
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
const outputDir = resolve(here, "_screenshots");
const referenceUrl = pathToFileURL(resolve(here, "index.html")).href;
const port = Number.parseInt(process.env.ZIRCON_WORKBENCH_CDP_PORT ?? String(11480 + Math.floor(Math.random() * 500)), 10);
const profile = mkdirTempProfile();
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

let nextId = 1;

try {
  mkdirSync(outputDir, { recursive: true });
  const list = await waitForJson(`http://127.0.0.1:${port}/json/list`);
  const target = list.find((item) => item.type === "page") ?? list[0];
  const cdp = await connect(target.webSocketDebuggerUrl);
  await cdp.send("Page.enable");
  await cdp.send("Runtime.enable");

  for (const shot of [
    ["workbench-1672x941-final.png", 1672, 941],
    ["workbench-720x760.png", 720, 760],
  ]) {
    const [fileName, width, height] = shot;
    await cdp.send("Emulation.setDeviceMetricsOverride", {
      width,
      height,
      deviceScaleFactor: 1,
      mobile: false,
    });
    await cdp.send("Page.navigate", { url: referenceUrl });
    await waitForWorkbench(cdp);
    await settleLayout(cdp);
    await delay(250);
    const result = await cdp.send("Page.captureScreenshot", {
      format: "png",
      captureBeyondViewport: false,
      fromSurface: true,
    });
    await writePngWithRetry(resolve(outputDir, fileName), Buffer.from(result.data, "base64"));
    console.log(`captured ${fileName} at ${width}x${height}`);
  }

  cdp.close();
} finally {
  await cleanup();
}

function mkdirTempProfile() {
  const root = resolve(tmpdir(), `zircon-workbench-cdp-${process.pid}-${Date.now()}`);
  mkdirSync(root, { recursive: true });
  return root;
}

async function waitForWorkbench(cdp, attempts = 80) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    const ready = await evaluate(cdp, `Boolean(
      document.querySelector(".zr-window") &&
      document.querySelector(".zr-viewport-reference")?.complete &&
      (!document.fonts || document.fonts.status === "loaded")
    )`);
    if (ready) return;
    await delay(100);
  }
  throw new Error("Timed out waiting for workbench component prototype.");
}

async function settleLayout(cdp) {
  await evaluate(cdp, `new Promise((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(resolve));
  })`);
}

async function writePngWithRetry(filePath, bytes, attempts = 12) {
  const tempPath = `${filePath}.${process.pid}.tmp`;
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      writeFileSync(tempPath, bytes);
      renameSync(tempPath, filePath);
      return;
    } catch (error) {
      rmSync(tempPath, { force: true });
      if (!["EBUSY", "EPERM", "EACCES"].includes(error.code) || attempt === attempts - 1) {
        throw error;
      }
      await delay(125);
    }
  }
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
  if (!resolvedProfile.startsWith(tmpWithSeparator) || !resolvedProfile.includes("zircon-workbench-cdp-")) {
    throw new Error(`Refusing to remove unexpected temporary profile path: ${profilePath}`);
  }
}

function delay(ms) {
  return new Promise((resolveDelay) => setTimeout(resolveDelay, ms));
}
