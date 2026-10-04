import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { existsSync } from "node:fs";
import path from "node:path";
import test from "node:test";

const require = createRequire(import.meta.url);
const playwrightRoot = process.env.ZIRCON_PLAYWRIGHT_NODE_MODULES;
const { chromium } = playwrightRoot ? require(path.join(playwrightRoot, "playwright")) : { chromium: undefined };
const executablePath = process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH || (chromium ? chromium.executablePath() : undefined);
const browserAvailable = Boolean(chromium && executablePath && existsSync(executablePath));
const launchOptions = executablePath ? { executablePath } : {};
const baseUrl = process.env.ZIRCON_HUB_TEST_URL;

for (const viewport of [
  { width: 360, height: 760 },
  { width: 1280, height: 800 },
]) {
  test(`project search remains named and bounded at ${viewport.width}px`, { skip: !baseUrl || !browserAvailable }, async () => {
    const browser = await chromium.launch({ headless: true, ...launchOptions });
    try {
      const context = await contextWithState(browser, viewport);
      const page = await context.newPage();
      await page.goto(baseUrl);
      const search = page.getByRole("textbox", { name: "搜索项目...", exact: true });
      await search.waitFor();
      assert.equal(await search.getAttribute("aria-label"), "搜索项目...");
      assert.equal(await search.getAttribute("placeholder"), "搜索项目...");
      const overflow = await page.evaluate(() => ({
        document: document.documentElement.scrollWidth - document.documentElement.clientWidth,
        body: document.body.scrollWidth - document.body.clientWidth,
      }));
      assert.deepEqual(overflow, { document: 0, body: 0 });
    } finally {
      await browser.close();
    }
  });
}

async function contextWithState(browser, viewport) {
  const context = await browser.newContext({ viewport });
  await context.addInitScript(() => {
    const callbacks = new Map();
    let callbackId = 0;
    let state;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback(callback) {
        callbacks.set(++callbackId, callback);
        return callbackId;
      },
      unregisterCallback(id) {
        callbacks.delete(id);
      },
      async invoke(command, args) {
        if (command === "hub_state") {
          const { fallbackShellState } = await import("/src/data/hubData.ts");
          state = structuredClone(fallbackShellState);
          state.backendEpoch = "search-accessibility-browser";
          state.stateRevision = "1";
          return state;
        }
        if (command === "plugin:event|listen") {
          return 1;
        }
        if (command === "hub_action") {
          state = { ...state, stateRevision: String(BigInt(state.stateRevision) + 1n) };
          return state;
        }
        return undefined;
      },
    };
  });
  return context;
}
