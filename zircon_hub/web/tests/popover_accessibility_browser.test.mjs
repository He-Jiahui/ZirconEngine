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

test("topbar popovers expose state and restore focus after Escape", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await contextWithState(browser);
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.getByTestId("hub-navigation-drawer").waitFor();

    assert.equal(await page.getByRole("button", { name: "Zircon Engine 1.8.2", exact: true }).count(), 1);
    const engineTrigger = page.locator('button[aria-controls="hub-source-engine-popover"]');
    const enginePopoverId = await engineTrigger.getAttribute("aria-controls");
    assert.ok(enginePopoverId);
    assert.equal(await engineTrigger.getAttribute("aria-haspopup"), "dialog");
    assert.equal(await engineTrigger.getAttribute("aria-expanded"), "false");
    await engineTrigger.click();
    assert.equal(await engineTrigger.getAttribute("aria-expanded"), "true");
    const enginePopover = page.locator(`#${enginePopoverId}`);
    assert.equal(await enginePopover.getAttribute("role"), "dialog");
    assert.equal(await enginePopover.getAttribute("aria-label"), "当前引擎");
    await page.keyboard.press("Escape");
    await page.waitForFunction(() => document.activeElement === document.querySelector('button[aria-controls="hub-source-engine-popover"]'));
    assert.equal(await engineTrigger.getAttribute("aria-expanded"), "false");

    assert.equal(await page.getByRole("button", { name: "我的账户", exact: true }).count(), 1);
    const userTrigger = page.locator('button[aria-controls="hub-user-menu-popover"]');
    const userPopoverId = await userTrigger.getAttribute("aria-controls");
    assert.ok(userPopoverId);
    await userTrigger.click();
    assert.equal(await userTrigger.getAttribute("aria-expanded"), "true");
    const userPopover = page.locator(`#${userPopoverId}`);
    assert.equal(await userPopover.getAttribute("role"), "dialog");
    assert.equal(await userPopover.getAttribute("aria-label"), "我的账户");
    await page.keyboard.press("Escape");
    await page.waitForFunction(() => document.activeElement === document.querySelector('button[aria-controls="hub-user-menu-popover"]'));
    assert.equal(await userTrigger.getAttribute("aria-expanded"), "false");
  } finally {
    await browser.close();
  }
});

async function contextWithState(browser) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  await context.addInitScript(() => {
    const callbacks = new Map();
    let callbackId = 0;
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
          return { ...fallbackShellState, backendEpoch: "popover-accessibility", stateRevision: "1" };
        }
        if (command === "plugin:event|listen") {
          return 1;
        }
        return undefined;
      },
    };
  });
  return context;
}
