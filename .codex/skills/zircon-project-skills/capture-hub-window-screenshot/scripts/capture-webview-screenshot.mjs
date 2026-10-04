import { writeFile } from "node:fs/promises";

const args = parseArgs(process.argv.slice(2));
const port = Number(args.port ?? 0);
const output = args.output;
const waitMs = Number(args["wait-ms"] ?? 8000);
const expectedTitle = args.title ?? "Zircon Hub";
const expectedViewportWidth = Number(args["viewport-width"] ?? 0);
const expectedViewportHeight = Number(args["viewport-height"] ?? 0);

if (!Number.isInteger(port) || port <= 0) {
  throw new Error("Missing or invalid --port.");
}
if (!output) {
  throw new Error("Missing --output.");
}
if (
  !Number.isSafeInteger(expectedViewportWidth) ||
  !Number.isSafeInteger(expectedViewportHeight) ||
  expectedViewportWidth < 0 ||
  expectedViewportHeight < 0 ||
  (expectedViewportWidth === 0) !== (expectedViewportHeight === 0)
) {
  throw new Error("--viewport-width and --viewport-height must be positive integers provided together.");
}

const deadline = Date.now() + waitMs;
const page = await waitForPage(port, expectedTitle, deadline);
const client = await connectDevTools(page.webSocketDebuggerUrl);

try {
  await client.send("Page.enable");
  await client.send("Runtime.enable");
  await client.send("Page.bringToFront");

  const state = await waitForRenderedHub(client, deadline);
  // Tauri sizes the inner window in logical pixels, so check the CSS viewport, not PNG device pixels.
  if (
    expectedViewportWidth > 0 &&
    (!Number.isFinite(state.viewportWidth) ||
      !Number.isFinite(state.viewportHeight) ||
      Math.abs(state.viewportWidth - expectedViewportWidth) > 2 ||
      Math.abs(state.viewportHeight - expectedViewportHeight) > 2)
  ) {
    throw new Error(
      `Requested Hub viewport ${expectedViewportWidth}x${expectedViewportHeight}, actual WebView viewport ${state.viewportWidth}x${state.viewportHeight}.`,
    );
  }
  const screenshot = await client.send("Page.captureScreenshot", {
    format: "png",
    captureBeyondViewport: false,
    fromSurface: true,
  });

  if (!screenshot.result?.data) {
    throw new Error("WebView did not return screenshot data.");
  }

  await writeFile(output, Buffer.from(screenshot.result.data, "base64"));
  process.stdout.write(
    `${JSON.stringify({
      path: output,
      title: state.title,
      url: state.url,
      textLength: state.textLength,
      rootChildren: state.rootChildren,
      viewportWidth: state.viewportWidth,
      viewportHeight: state.viewportHeight,
    })}\n`,
  );
} finally {
  client.close();
}

function parseArgs(argv) {
  const parsed = {};
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (!token.startsWith("--")) {
      continue;
    }

    const key = token.slice(2);
    const next = argv[index + 1];
    if (next === undefined || next.startsWith("--")) {
      parsed[key] = "true";
    } else {
      parsed[key] = next;
      index += 1;
    }
  }
  return parsed;
}

async function waitForPage(port, title, deadline) {
  let lastError = "";
  while (Date.now() < deadline) {
    try {
      const pages = await fetch(`http://127.0.0.1:${port}/json`).then((response) => response.json());
      const page =
        pages.find((candidate) => candidate.type === "page" && candidate.title === title) ??
        pages.find((candidate) => candidate.type === "page");
      if (page?.webSocketDebuggerUrl) {
        return page;
      }
      lastError = `No page target found on port ${port}.`;
    } catch (error) {
      lastError = error instanceof Error ? error.message : String(error);
    }

    await sleep(200);
  }

  throw new Error(`Timed out waiting for WebView debugger page on port ${port}: ${lastError}`);
}

function connectDevTools(webSocketUrl) {
  const socket = new WebSocket(webSocketUrl);
  let nextId = 1;
  const pending = new Map();

  socket.addEventListener("message", (event) => {
    const message = JSON.parse(event.data);
    if (message.id && pending.has(message.id)) {
      pending.get(message.id)(message);
      pending.delete(message.id);
    }
  });

  const opened = new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });

  return opened.then(() => ({
    send(method, params = {}) {
      const id = nextId;
      nextId += 1;
      socket.send(JSON.stringify({ id, method, params }));
      return new Promise((resolve, reject) => {
        const timeout = setTimeout(() => {
          pending.delete(id);
          reject(new Error(`Timed out waiting for DevTools method ${method}.`));
        }, 5000);
        pending.set(id, (message) => {
          clearTimeout(timeout);
          if (message.error) {
            reject(new Error(`${method} failed: ${JSON.stringify(message.error)}`));
            return;
          }
          resolve(message);
        });
      });
    },
    close() {
      socket.close();
    },
  }));
}

async function waitForRenderedHub(client, deadline) {
  let lastState = undefined;
  while (Date.now() < deadline) {
    lastState = await readPageState(client);
    if (
      lastState.readyState === "complete" &&
      lastState.title === expectedTitle &&
      lastState.rootChildren > 0 &&
      lastState.textLength > 0
    ) {
      return lastState;
    }

    await sleep(200);
  }

  throw new Error(`Timed out waiting for rendered Hub DOM: ${JSON.stringify(lastState)}`);
}

async function readPageState(client) {
  const expression = `(() => {
    const root = document.getElementById("root");
    const bodyText = document.body.innerText || "";
    return {
      title: document.title,
      url: location.href,
      readyState: document.readyState,
      rootChildren: root ? root.children.length : -1,
      textLength: bodyText.trim().length,
      viewportWidth: window.innerWidth,
      viewportHeight: window.innerHeight,
      bodyText: bodyText.slice(0, 200)
    };
  })()`;

  const result = await client.send("Runtime.evaluate", {
    expression,
    returnByValue: true,
  });

  return result.result.result.value;
}

function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}
