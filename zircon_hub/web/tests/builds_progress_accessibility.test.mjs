import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const root = new URL("../src/", import.meta.url);
const require = createRequire(import.meta.url);
const playwrightRoot = process.env.ZIRCON_PLAYWRIGHT_NODE_MODULES;
const { chromium } = playwrightRoot ? require(path.join(playwrightRoot, "playwright")) : { chromium: undefined };
const executablePath = process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH || (chromium ? chromium.executablePath() : undefined);
const browserAvailable = Boolean(chromium && executablePath && existsSync(executablePath));
const launchOptions = executablePath ? { executablePath } : {};
const baseUrl = process.env.ZIRCON_HUB_TEST_URL;

test("build workflow progress exposes its localized panel name", async () => {
  const source = await readFile(new URL("pages/BuildsPage.tsx", root), "utf8");

  assert.match(
    source,
    /<LinearProgress\s+aria-label=\{text\.buildWorkflow\}\s+variant=\"determinate\"\s+value=\{state\.taskSummary\.progressPercent\}/,
  );
});

test("build workflow progress exposes its localized name in Chrome", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await context.addInitScript(() => {
      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
      window.__TAURI_INTERNALS__ = {
        transformCallback() { return 1; },
        unregisterCallback() {},
        async invoke(command) {
          if (command === "hub_state") {
            const { fallbackShellState } = await import("/src/data/hubData.ts");
            const state = structuredClone(fallbackShellState);
            state.activePage = "builds";
            state.pageTitle = "构建";
            state.backendEpoch = "builds-progress-accessibility";
            state.stateRevision = "1";
            state.taskSummary = {
              ...state.taskSummary,
              running: true,
              label: "构建中",
              operation: "构建项目",
              progressPercent: 42,
            };
            return state;
          }
          if (command === "plugin:event|listen") return 1;
          return undefined;
        },
      };
    });
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.getByRole("heading", { name: "构建", exact: true }).waitFor();
    const progress = page.getByRole("progressbar", { name: "构建工作流", exact: true });
    assert.equal(await progress.count(), 1);
    assert.equal(await progress.getAttribute("aria-valuenow"), "42");
  } finally {
    await browser.close();
  }
});
