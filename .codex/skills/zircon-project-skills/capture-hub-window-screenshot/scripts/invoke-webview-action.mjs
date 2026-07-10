const args = parseArgs(process.argv.slice(2));
const port = Number(args.port ?? 0);
const waitMs = Number(args["wait-ms"] ?? 8000);
const expectedTitle = args.title ?? "Zircon Hub";
const action = args.action;
const text = args.text;

if (!Number.isInteger(port) || port <= 0) {
  throw new Error("Missing or invalid --port.");
}
if (action !== "scroll-text" && action !== "click-text" && action !== "wait-text" && action !== "wait-any-text") {
  throw new Error("Missing or invalid --action. Expected scroll-text, click-text, wait-text, or wait-any-text.");
}
if (!text) {
  throw new Error("Missing --text.");
}

const deadline = Date.now() + waitMs;
const page = await waitForPage(port, expectedTitle, deadline);
const client = await connectDevTools(page.webSocketDebuggerUrl);

try {
  await client.send("Runtime.enable");
  await client.send("Page.bringToFront");

  const value =
    action === "wait-text" || action === "wait-any-text"
      ? await waitForBodyText(client, action === "wait-any-text" ? text.split("|||").filter(Boolean) : [text], deadline)
      : await waitForElementAction(client, action, text, deadline);
  if (!value?.found) {
    throw new Error(`Could not find visible WebView target with text '${text}'.`);
  }

  process.stdout.write(`${JSON.stringify(value)}\n`);
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

async function waitForElementAction(client, actionName, targetText, deadline) {
  let lastValue = undefined;
  while (Date.now() < deadline) {
    const result = await client.send("Runtime.evaluate", {
      expression: actionExpression(actionName, targetText),
      awaitPromise: true,
      returnByValue: true,
    });
    lastValue = result.result.result.value;
    if (lastValue?.found) {
      return lastValue;
    }

    await sleep(200);
  }

  return lastValue ?? { found: false, action: actionName, text: targetText };
}

async function waitForBodyText(client, targetTexts, deadline) {
  let lastValue = undefined;
  while (Date.now() < deadline) {
    const result = await client.send("Runtime.evaluate", {
      expression: `(() => {
        const targetTexts = ${JSON.stringify(targetTexts)};
        const text = document.body.innerText || "";
        const matchedText = targetTexts.find((targetText) => text.includes(targetText)) || "";
        return {
          found: Boolean(matchedText),
          action: targetTexts.length > 1 ? "wait-any-text" : "wait-text",
          text: matchedText || targetTexts[0],
          candidates: targetTexts,
          textLength: text.trim().length,
          sample: text.slice(0, 260)
        };
      })()`,
      returnByValue: true,
    });
    lastValue = result.result.result.value;
    if (lastValue?.found) {
      return lastValue;
    }

    await sleep(200);
  }

  return lastValue ?? { found: false, action: targetTexts.length > 1 ? "wait-any-text" : "wait-text", text: targetTexts[0], candidates: targetTexts };
}

function actionExpression(actionName, targetText) {
  return `new Promise((resolve) => {
    const targetText = ${JSON.stringify(targetText)};
    const normalize = (value) => (value || "").replace(/\\s+/g, " ").trim();
    const candidates = [...document.querySelectorAll("button,[role='button'],[role='tab']")];
    const visibleCandidates = candidates.filter((candidate) => {
      const rect = candidate.getBoundingClientRect();
      const style = getComputedStyle(candidate);
      return rect.width > 0 && rect.height > 0 && style.visibility !== "hidden" && style.display !== "none";
    });
    const target =
      visibleCandidates.find((candidate) => normalize(candidate.innerText || candidate.getAttribute("aria-label")) === targetText) ||
      visibleCandidates.find((candidate) => normalize(candidate.innerText || candidate.getAttribute("aria-label")).includes(targetText));

    if (!target) {
      resolve({ found: false, text: targetText });
      return;
    }

    target.scrollIntoView({ block: "center", inline: "center" });
    requestAnimationFrame(() => {
      if (${JSON.stringify(actionName)} === "click-text") {
        target.click();
      }
      setTimeout(() => {
        const rect = target.getBoundingClientRect();
        resolve({
          found: true,
          action: ${JSON.stringify(actionName)},
          text: normalize(target.innerText || target.getAttribute("aria-label")),
          left: rect.left,
          top: rect.top,
          right: rect.right,
          bottom: rect.bottom,
          width: rect.width,
          height: rect.height,
          cx: rect.left + rect.width / 2,
          cy: rect.top + rect.height / 2,
          scrollX: window.scrollX,
          scrollY: window.scrollY
        });
      }, 500);
    });
  })`;
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

function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}
