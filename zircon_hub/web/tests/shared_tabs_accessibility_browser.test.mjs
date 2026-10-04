import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { existsSync } from "node:fs";
import test from "node:test";
import path from "node:path";

const require = createRequire(import.meta.url);
const playwrightRoot = process.env.ZIRCON_PLAYWRIGHT_NODE_MODULES;
const { chromium } = playwrightRoot ? require(path.join(playwrightRoot, "playwright")) : { chromium: undefined };
const executablePath = process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH || (chromium ? chromium.executablePath() : undefined);
const browserAvailable = Boolean(chromium && executablePath && existsSync(executablePath));
const launchOptions = executablePath ? { executablePath } : {};
const baseUrl = process.env.ZIRCON_HUB_TEST_URL;

test("shared page tabs expose a stable localized tablist name", { skip: !baseUrl || !browserAvailable }, async () => {
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
            return { ...fallbackShellState, activePage: "builds", backendEpoch: "shared-tabs", stateRevision: "1" };
          }
          if (command === "plugin:event|listen") return 1;
          return undefined;
        },
      };
    });
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.getByTestId("hub-navigation-drawer").waitFor();
    await page.getByRole("tab", { name: "工作流", exact: true }).waitFor();
    const tablist = page.getByRole("tablist", { name: "工作流, 历史, 输出", exact: true });
    assert.equal(await tablist.count(), 1);
    assert.equal(await tablist.getByRole("tab", { name: "工作流", exact: true }).getAttribute("aria-selected"), "true");
  } finally {
    await browser.close();
  }
});
