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

test("project row menu exposes trigger state, menu ownership, and Escape focus return", { skip: !baseUrl || !browserAvailable, timeout: 20_000 }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await contextWithState(browser);
    const page = await context.newPage();
    await page.goto(baseUrl);

    await page.waitForFunction(() => document.querySelectorAll('button[aria-label="项目管理: Project A"]').length === 2);
    const triggers = page.getByRole("button", { name: "项目管理: Project A", exact: true });
    const trigger = triggers.first();
    await trigger.waitFor();
    assert.equal(await triggers.count(), 2);
    assert.equal(await trigger.getAttribute("aria-haspopup"), "menu");
    assert.equal(await trigger.getAttribute("aria-expanded"), "false");
    assert.equal(await trigger.getAttribute("aria-controls"), "hub-project-row-menu");

    await trigger.click();
    const menu = page.locator("#hub-project-row-menu");
    await menu.waitFor();
    assert.equal(await menu.getAttribute("role"), "menu");
    assert.equal(await menu.getAttribute("aria-label"), "项目管理: Project A");
    await page.waitForFunction(() => {
      const triggers = [...document.querySelectorAll('button[aria-label="项目管理: Project A"]')];
      return triggers.length === 2 && triggers[0].getAttribute("aria-expanded") === "true" && triggers[1].getAttribute("aria-expanded") === "false";
    });
    assert.deepEqual(
      await page.evaluate(() => [...document.querySelectorAll('button[aria-label="项目管理: Project A"]')].map((trigger) => trigger.getAttribute("aria-expanded"))),
      ["true", "false"],
    );
    assert.equal(await menu.getByRole("menuitem").count(), 3);

    await page.keyboard.press("Escape");
    await page.waitForFunction(() => {
      const triggers = [...document.querySelectorAll('button[aria-label="项目管理: Project A"]')];
      return document.activeElement?.getAttribute("aria-label") === "项目管理: Project A" && triggers.every((trigger) => trigger.getAttribute("aria-expanded") === "false");
    });
    assert.deepEqual(
      await page.evaluate(() => [...document.querySelectorAll('button[aria-label="项目管理: Project A"]')].map((trigger) => trigger.getAttribute("aria-expanded"))),
      ["false", "false"],
    );
    assert.equal(await page.evaluate(() => document.activeElement?.getAttribute("aria-label")), "项目管理: Project A");
  } finally {
    await browser.close();
  }
});

async function contextWithState(browser) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  await context.addInitScript(() => {
    const callbacks = new Map();
    let callbackId = 0;
    let revision = 1;
    let state;
    window.__HUB_TEST_ACTIONS__ = [];
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
          state.backendEpoch = "project-menu-accessibility-browser";
          state.stateRevision = String(revision);
          state.activePage = "projects";
          state.pageTitle = state.ui.shell.navItems.find((item) => item.id === "projects")?.label ?? "projects";
          state.projectViewMode = "list";
          state.browserProjects = [
            { id: "project-a", name: "Project A", engineVersion: "1.0", modified: "today", location: "E:/projects/a", coverId: "empty", pinned: false },
          ];
          state.recentProjects = state.browserProjects;
          state.projects = [];
          state.selectedProjectId = null;
          return state;
        }
        if (command === "plugin:event|listen") {
          return 1;
        }
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
