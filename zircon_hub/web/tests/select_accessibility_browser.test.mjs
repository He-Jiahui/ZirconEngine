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

test("project filter and sort selects keep stable localized names", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await contextWithState(browser);
    const page = await context.newPage();
    await page.goto(baseUrl);
    const filter = page.getByRole("combobox", { name: "筛选项目", exact: true });
    const sort = page.getByRole("combobox", { name: "排序项目", exact: true });
    await filter.waitFor();
    assert.equal(await filter.count(), 1);
    assert.equal(await sort.count(), 1);
    await filter.click();
    await page.getByRole("option", { name: "缺失", exact: true }).click();
    assert.equal(await filter.textContent(), "缺失");
  } finally {
    await browser.close();
  }
});

async function contextWithState(browser) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  await context.addInitScript(() => {
    let revision = 1;
    let state;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback() { return 1; },
      unregisterCallback() {},
      async invoke(command, args) {
        if (command === "hub_state") {
          const { fallbackShellState } = await import("/src/data/hubData.ts");
          state = structuredClone(fallbackShellState);
          state.backendEpoch = "select-accessibility-browser";
          state.stateRevision = String(revision);
          return state;
        }
        if (command === "plugin:event|listen") return 1;
        if (command === "hub_action") {
          state = { ...state, stateRevision: String(++revision) };
          return state;
        }
        return undefined;
      },
    };
  });
  return context;
}
