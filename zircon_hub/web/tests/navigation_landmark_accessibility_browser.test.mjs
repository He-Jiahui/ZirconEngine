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

test("navigation drawer exposes one navigation landmark around the active route", { skip: !baseUrl || !browserAvailable }, async () => {
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
            return { ...fallbackShellState, backendEpoch: "navigation-landmark", stateRevision: "1" };
          }
          if (command === "plugin:event|listen") return 1;
          return undefined;
        },
      };
    });
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.getByTestId("hub-navigation-drawer").waitFor();

    const navigation = page.getByRole("navigation");
    assert.equal(await navigation.count(), 1);
    const projects = navigation.getByRole("button", { name: "项目", exact: true });
    assert.equal(await projects.getAttribute("aria-current"), "page");

    const collapse = page.getByRole("button", { name: "收起", exact: true });
    assert.equal(await collapse.getAttribute("aria-expanded"), "true");
    assert.equal(await collapse.getAttribute("aria-controls"), "hub-navigation-items");
    await collapse.click();
    const expand = page.getByRole("button", { name: "展开", exact: true });
    assert.equal(await expand.getAttribute("aria-expanded"), "false");
    assert.equal(await page.locator("#hub-navigation-items").count(), 1);
  } finally {
    await browser.close();
  }
});
