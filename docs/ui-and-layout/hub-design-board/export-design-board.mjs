import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { createServer } from "node:http";
import { existsSync, mkdtempSync, rmSync } from "node:fs";
import { mkdir, readFile, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, extname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { CANVAS_HEIGHT, CANVAS_WIDTH } from "../hub-web-reference/page-registry.mjs";
import {
  DESIGN_BOARD_EXPORT_HASH_INPUTS,
  DESIGN_BOARD_EXPORT_METADATA,
  DESIGN_BOARD_LIST,
  DESIGN_BOARD_MANIFEST,
  DESIGN_BOARD_MANIFEST_SCHEMA,
  DESIGN_BOARD_REVIEW_INDEX,
  DESIGN_BOARD_SOURCE,
  REFERENCE_ALIGNMENT_MATRIX,
  STRUCTURE_SIGNOFF_CHECKLIST,
  STRUCTURE_DECISION_LOG,
  STRUCTURE_COVERAGE_MATRIX,
  STRUCTURE_ACCEPTANCE_RECORD,
  STRUCTURE_GEOMETRY_BASELINE,
  STRUCTURE_GEOMETRY_EVIDENCE,
  STRUCTURE_RESPONSIVE_BASELINE,
  STRUCTURE_REVIEW_ROUTE_BASELINE,
  STRUCTURE_OVERLAY_BASELINE,
  STRUCTURE_REFERENCE_ROUTE_BASELINE,
  STRUCTURE_REVIEW_CHECKLIST,
  STRUCTURE_REVIEW_GUIDE,
  STRUCTURE_REVIEW_PACKET,
  STRUCTURE_REVIEW_PACKET_SCHEMA,
  STRUCTURE_REVIEW_STATUS,
  STRUCTURE_TO_REFERENCE_MAP,
} from "./board-registry.mjs";

const host = "127.0.0.1";
const port = Number.parseInt(process.env.ZIRCON_HUB_DESIGN_BOARD_PORT ?? "5298", 10);
const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "../../..");
const outputDir = resolve(repoRoot, "docs/ui-and-layout");
const basePath = `/${DESIGN_BOARD_SOURCE}`;

const mime = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".png": "image/png",
  ".svg": "image/svg+xml",
};

await mkdir(outputDir, { recursive: true });

const server = createServer(async (req, res) => {
  try {
    const url = new URL(req.url ?? "/", `http://${host}:${port}`);
    const requestPath = url.pathname === "/" ? basePath : url.pathname;
    const filePath = resolve(repoRoot, requestPath.replace(/^\/+/, ""));
    if (!filePath.startsWith(repoRoot)) {
      throw new Error(`path escapes repository root: ${requestPath}`);
    }
    const body = await readFile(filePath);
    res.writeHead(200, {
      "Content-Type": mime[extname(filePath)] ?? "application/octet-stream",
      "Cache-Control": "no-cache",
    });
    res.end(body);
  } catch (error) {
    res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
    res.end(`Not found: ${error}`);
  }
});

try {
  await listen(server);
  await captureBoards();
  await writeIndex();
  await writeExportMetadata();
} finally {
  await close(server);
}

function listen(server) {
  return new Promise((resolveListen, rejectListen) => {
    server.once("error", rejectListen);
    server.listen(port, host, () => resolveListen());
  });
}

function close(server) {
  return new Promise((resolveClose) => server.close(() => resolveClose()));
}

async function captureBoards() {
  const edge = [
    "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
    "C:/Program Files/Microsoft/Edge/Application/msedge.exe",
  ].find((candidate) => existsSync(candidate));
  if (!edge) {
    throw new Error("Microsoft Edge executable not found.");
  }

  const cdpPort = Number.parseInt(
    process.env.ZIRCON_HUB_DESIGN_BOARD_CDP_PORT ?? String(11_600 + Math.floor(Math.random() * 500)),
    10,
  );
  const profile = mkdtempSync(join(tmpdir(), "zircon-hub-design-board-export-"));
  const browser = spawn(
    edge,
    [
      "--headless=new",
      "--disable-gpu",
      "--hide-scrollbars",
      "--allow-file-access-from-files",
      `--remote-debugging-port=${cdpPort}`,
      `--user-data-dir=${profile}`,
      "about:blank",
    ],
    { stdio: "ignore" },
  );

  try {
    const list = await waitForJson(`http://127.0.0.1:${cdpPort}/json/list`);
    const target = list.find((item) => item.type === "page") ?? list[0];
    const cdp = await connect(target.webSocketDebuggerUrl);
    await cdp.send("Page.enable");
    await cdp.send("Runtime.enable");
    await cdp.send("Emulation.setDeviceMetricsOverride", {
      width: CANVAS_WIDTH,
      height: CANVAS_HEIGHT,
      deviceScaleFactor: 1,
      mobile: false,
    });

    for (const { id, output } of DESIGN_BOARD_LIST) {
      await captureBoard(cdp, id, resolve(outputDir, output));
    }

    cdp.close();
  } finally {
    browser.kill();
    await waitForExit(browser);
    await removeProfile(profile);
  }
}

async function captureBoard(cdp, boardId, target) {
  const url = `http://${host}:${port}${basePath}?board=${encodeURIComponent(boardId)}`;
  console.log(`Capturing ${boardId} -> ${target}`);
  await cdp.send("Page.navigate", { url });
  await waitForBoard(cdp, boardId);
  await delay(250);
  const screenshot = await cdp.send("Page.captureScreenshot", {
    format: "png",
    fromSurface: true,
    captureBeyondViewport: false,
  });
  await writeFile(target, Buffer.from(screenshot.data, "base64"));
}

async function writeIndex() {
  const lines = DESIGN_BOARD_LIST.map(({ id, output }) => `- ${output}: ${id}`).join("\n");
  await writeFile(
    resolve(here, "EXPORTS.md"),
    `# Hub Design Board Exports

Canvas: ${CANVAS_WIDTH}x${CANVAS_HEIGHT}
Source page: ${DESIGN_BOARD_SOURCE}

${lines}

Review order:
1. hub-design-structure-layout.png
2. hub-design-structure-supplement.png
3. hub-design-functional-details.png

Review support:
- ${DESIGN_BOARD_REVIEW_INDEX}
- ${DESIGN_BOARD_MANIFEST}
- ${DESIGN_BOARD_MANIFEST_SCHEMA}
- ${STRUCTURE_REVIEW_CHECKLIST}
- ${STRUCTURE_COVERAGE_MATRIX}
- ${STRUCTURE_GEOMETRY_EVIDENCE}
- ${STRUCTURE_GEOMETRY_BASELINE}
- ${STRUCTURE_RESPONSIVE_BASELINE}
- ${STRUCTURE_REVIEW_ROUTE_BASELINE}
- ${STRUCTURE_OVERLAY_BASELINE}
- ${STRUCTURE_REFERENCE_ROUTE_BASELINE}
- ${STRUCTURE_REVIEW_GUIDE}
- ${STRUCTURE_TO_REFERENCE_MAP}
- ${REFERENCE_ALIGNMENT_MATRIX}
- ${STRUCTURE_SIGNOFF_CHECKLIST}
- ${STRUCTURE_DECISION_LOG}
- ${STRUCTURE_REVIEW_STATUS}
- ${STRUCTURE_ACCEPTANCE_RECORD}
- ${STRUCTURE_REVIEW_PACKET}
- ${STRUCTURE_REVIEW_PACKET_SCHEMA}

Export metadata: ${DESIGN_BOARD_EXPORT_METADATA}
`,
    "utf8",
  );
}

async function writeExportMetadata() {
  const sourceInputs = [];
  for (const sourceInput of DESIGN_BOARD_EXPORT_HASH_INPUTS) {
    sourceInputs.push({
      path: sourceInput,
      sha256: await sha256File(resolve(repoRoot, sourceInput)),
    });
  }

  const artifacts = [];
  for (const { id, output, category } of DESIGN_BOARD_LIST) {
    const artifactPath = resolve(outputDir, output);
    const info = await stat(artifactPath);
    artifacts.push({
      id,
      output,
      category,
      sha256: await sha256File(artifactPath),
      bytes: info.size,
    });
  }

  await writeFile(
    resolve(repoRoot, DESIGN_BOARD_EXPORT_METADATA),
    `${JSON.stringify(
      {
        source: DESIGN_BOARD_SOURCE,
        canvas: {
          width: CANVAS_WIDTH,
          height: CANVAS_HEIGHT,
        },
        hash_algorithm: "sha256",
        source_inputs: sourceInputs,
        artifacts,
      },
      null,
      2,
    )}\n`,
    "utf8",
  );
}

async function sha256File(filePath) {
  return createHash("sha256").update(await readFile(filePath)).digest("hex");
}

async function waitForBoard(cdp, boardId, attempts = 80) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      const state = JSON.parse(
        await evaluate(
          cdp,
          `JSON.stringify({
            activeBoard: document.documentElement.dataset.board,
            hasShell: !!document.querySelector(".design-shell"),
            boxWidth: document.querySelector(".design-shell")?.getBoundingClientRect().width ?? 0,
            boxHeight: document.querySelector(".design-shell")?.getBoundingClientRect().height ?? 0
          })`,
        ),
      );
      if (state.activeBoard === boardId && state.hasShell && state.boxWidth === CANVAS_WIDTH && state.boxHeight === CANVAS_HEIGHT) {
        return;
      }
    } catch (_) {
      // Navigation swaps documents; wait until the new board is measurable.
    }
    await delay(100);
  }
  throw new Error(`Timed out waiting for design board ${boardId}.`);
}

function delay(ms) {
  return new Promise((resolveDelay) => setTimeout(resolveDelay, ms));
}

async function waitForJson(url, attempts = 80) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return response.json();
      }
    } catch (_) {
      // The debug endpoint takes a moment to open.
    }
    await delay(100);
  }
  throw new Error(`Timed out waiting for ${url}.`);
}

function connect(wsUrl) {
  return new Promise((resolveConnect, rejectConnect) => {
    const ws = new WebSocket(wsUrl);
    const pending = new Map();
    let nextId = 1;

    ws.addEventListener("open", () => {
      resolveConnect({
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
      if (!message.id || !pending.has(message.id)) {
        return;
      }
      const request = pending.get(message.id);
      pending.delete(message.id);
      if (message.error) {
        request.rejectSend(new Error(`${request.method}: ${message.error.message}`));
      } else {
        request.resolveSend(message.result);
      }
    });

    ws.addEventListener("error", rejectConnect);
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

async function waitForExit(child, timeoutMs = 3000) {
  if (child.exitCode !== null || child.signalCode !== null) {
    return;
  }
  await new Promise((resolveExit) => {
    const timer = setTimeout(resolveExit, timeoutMs);
    child.once("exit", () => {
      clearTimeout(timer);
      resolveExit();
    });
  });
}

async function removeProfile(profile) {
  for (let attempt = 0; attempt < 8; attempt += 1) {
    try {
      rmSync(profile, { recursive: true, force: true });
      return;
    } catch (_) {
      await delay(250);
    }
  }
  console.warn(`Temporary Edge profile cleanup skipped: ${profile}`);
}
