import { spawn } from "node:child_process";
import { createServer } from "node:http";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, extname, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { CANVAS_HEIGHT, CANVAS_WIDTH, DASHBOARD_CAPTURE_NAME, DASHBOARD_PAGE_ID, EXPORTS_LIST } from "./page-registry.mjs";

const host = "127.0.0.1";
const configuredPort = process.env.ZIRCON_HUB_WEB_REFERENCE_PORT;
const preferredPort = Number.parseInt(configuredPort ?? "5198", 10);
const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "../../..");
const outputDir = resolve(repoRoot, "docs/ui-and-layout");
const basePath = "/docs/ui-and-layout/hub-web-reference/index.html";
const replayPath = basePath.replace(/^\/+/, "");
const captureConcurrency = Number.parseInt(process.env.ZIRCON_HUB_WEB_REFERENCE_CONCURRENCY ?? "4", 10);

const mime = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".png": "image/png",
  ".svg": "image/svg+xml",
};

const selected = selectedExports(process.argv.slice(2));
await mkdir(outputDir, { recursive: true });
let activePort = preferredPort;

const server = createServer(async (req, res) => {
  try {
    const url = new URL(req.url ?? "/", `http://${host}:${activePort}`);
    const requestPath = url.pathname === "/" ? basePath : url.pathname;
    const filePath = resolve(repoRoot, requestPath.replace(/^\/+/, ""));
    if (!isInsideRepo(filePath)) {
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
  activePort = await listen(server);
  await captureAll([
    [DASHBOARD_PAGE_ID, resolve(here, DASHBOARD_CAPTURE_NAME)],
    ...selected.map(([pageId, outputName]) => [pageId, resolve(outputDir, outputName)]),
  ]);
  await writeIndex(EXPORTS_LIST);
} finally {
  await close(server);
}

function selectedExports(args) {
  const only = args
    .filter((arg) => arg.startsWith("--only="))
    .flatMap((arg) => arg.slice("--only=".length).split(","))
    .map((arg) => arg.trim())
    .map((arg) => arg.replace(/\.png$/, ""))
    .filter(Boolean);

  if (only.length === 0) {
    return EXPORTS_LIST;
  }

  const available = new Map(EXPORTS_LIST.map(([pageId, outputName]) => [pageId, [pageId, outputName]]));
  const unknown = only.filter((pageId) => !available.has(pageId));
  if (unknown.length > 0) {
    throw new Error(
      `Unknown Hub web-reference --only page id(s): ${unknown.join(", ")}. Available page ids: ${[...available.keys()].join(", ")}`,
    );
  }

  return [...new Set(only)].map((pageId) => available.get(pageId));
}

async function listen(server) {
  try {
    return await listenOn(server, preferredPort);
  } catch (error) {
    if (configuredPort || error.code !== "EADDRINUSE") {
      throw error;
    }
    const fallbackPort = await listenOn(server, 0);
    console.log(`Default Hub web-reference export port ${preferredPort} is in use; using ${fallbackPort}.`);
    return fallbackPort;
  }
}

function listenOn(server, portToTry) {
  return new Promise((resolveListen, rejectListen) => {
    const onError = (error) => {
      server.off("listening", onListening);
      rejectListen(error);
    };
    const onListening = () => {
      server.off("error", onError);
      const address = server.address();
      resolveListen(typeof address === "object" && address ? address.port : portToTry);
    };
    server.once("error", onError);
    server.once("listening", onListening);
    server.listen(portToTry, host);
  });
}

function close(server) {
  if (!server.listening) {
    return Promise.resolve();
  }
  return new Promise((resolveClose) => server.close(() => resolveClose()));
}

function isInsideRepo(filePath) {
  const rootWithSeparator = repoRoot.endsWith(sep) ? repoRoot : `${repoRoot}${sep}`;
  return filePath === repoRoot || filePath.startsWith(rootWithSeparator);
}

async function capture(pageId, target) {
  const url = `http://${host}:${activePort}${basePath}?page=${encodeURIComponent(pageId)}`;
  const args = [
    "playwright",
    "screenshot",
    "--channel",
    "msedge",
    "--viewport-size",
    `${CANVAS_WIDTH},${CANVAS_HEIGHT}`,
    "--wait-for-selector",
    ".hub-shell",
    "--wait-for-timeout",
    "250",
    "--timeout",
    "120000",
    url,
    target,
  ];

  const command = process.platform === "win32" ? process.execPath : "npx";
  const commandArgs =
    process.platform === "win32"
      ? [resolve(dirname(process.execPath), "node_modules/npm/bin/npx-cli.js"), ...args]
      : args;
  await run(command, commandArgs, pageId);
}

async function captureAll(items) {
  const workers = Array.from({ length: Math.max(1, captureConcurrency) }, async (_, workerIndex) => {
    for (let itemIndex = workerIndex; itemIndex < items.length; itemIndex += Math.max(1, captureConcurrency)) {
      const [pageId, target] = items[itemIndex];
      await capture(pageId, target);
    }
  });
  await Promise.all(workers);
}

async function writeIndex(rows) {
  const lines = rows
    .map(
      ([pageId, outputName]) =>
        `- ${outputName}: ${pageId} (${replayPath}?page=${encodeURIComponent(pageId)})`,
    )
    .join("\n");
  await writeFile(
    resolve(here, "EXPORTS.md"),
    `# Hub Web Reference Exports

Canvas: ${CANVAS_WIDTH}x${CANVAS_HEIGHT}
Source page: docs/ui-and-layout/hub-web-reference/index.html
Reference dashboard capture: docs/ui-and-layout/hub-web-reference/${DASHBOARD_CAPTURE_NAME}

${lines}
`,
    "utf8",
  );
}

function run(command, args, pageId) {
  return new Promise((resolveRun, rejectRun) => {
    const child = spawn(command, args, {
      cwd: repoRoot,
      stdio: "inherit",
    });
    child.once("error", (error) => rejectRun(new Error(`screenshot failed for ${pageId}: ${error.message}`)));
    child.once("exit", (code, signal) => {
      if (code === 0) {
        resolveRun();
      } else {
        rejectRun(new Error(`screenshot failed for ${pageId}: exit ${code ?? signal}`));
      }
    });
  });
}
