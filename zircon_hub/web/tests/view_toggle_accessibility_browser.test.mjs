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

for (const scenario of [
  { projectSubpage: "dashboard", label: "项目" },
  { projectSubpage: "project-browser", label: "项目浏览器" },
]) {
  test(`project view toggle group is named on ${scenario.projectSubpage}`, { skip: !baseUrl || !browserAvailable, timeout: 15_000 }, async () => {
    const browser = await chromium.launch({ headless: true, ...launchOptions });
    try {
      const context = await contextWithState(browser, scenario.projectSubpage);
      const page = await context.newPage();
      await page.goto(baseUrl);

      const group = page.getByRole("group", { name: scenario.label, exact: true });
      await group.waitFor();
      assert.equal(await group.count(), 1);
      assert.equal(await group.getByRole("button", { name: "网格视图", exact: true }).getAttribute("aria-pressed"), "true");
      assert.equal(await group.getByRole("button", { name: "列表视图", exact: true }).getAttribute("aria-pressed"), "false");
    } finally {
      await browser.close();
    }
  });
}

async function contextWithState(browser, projectSubpage) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  await context.addInitScript(({ projectSubpage }) => {
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
      async invoke(command) {
        if (command === "hub_state") {
          const { fallbackShellState } = await import("/src/data/hubData.ts");
          state = structuredClone(fallbackShellState);
          state.backendEpoch = `view-toggle-${projectSubpage}`;
          state.stateRevision = "1";
          state.activePage = "projects";
          state.projectSubpage = projectSubpage;
          state.projectViewMode = "grid";
          return state;
        }
        if (command === "plugin:event|listen") {
          return 1;
        }
        return undefined;
      },
    };
  }, { projectSubpage });
  return context;
}
