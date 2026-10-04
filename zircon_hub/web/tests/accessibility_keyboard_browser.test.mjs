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

test("project table rows are keyboard-selectable and keep native table semantics", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await contextWithState(browser, "projects");
    const page = await context.newPage();
    await page.goto(baseUrl);

    const rows = page.getByRole("table").first().locator("tbody tr");
    await rows.first().waitFor({ state: "visible" });
    assert.equal(await rows.count(), 2);
    assert.equal(await rows.nth(0).getAttribute("aria-selected"), "false");
    assert.equal(await rows.nth(0).getAttribute("tabindex"), "0");
    await rows.nth(0).focus();
    await rows.nth(0).press("Enter");
    await page.waitForFunction(() => window.__HUB_TEST_ACTIONS__?.some((entry) => entry.actionId === "select-project"));
    assert.deepEqual(
      await page.evaluate(() => window.__HUB_TEST_ACTIONS__.at(-1)),
      { actionId: "select-project", targetId: "project-a" },
    );

    await rows.nth(1).focus();
    await rows.nth(1).press("Space");
    await page.waitForFunction(() => window.__HUB_TEST_ACTIONS__?.filter((entry) => entry.actionId === "select-project").length === 2);
    assert.deepEqual(
      await page.evaluate(() => window.__HUB_TEST_ACTIONS__.at(-1)),
      { actionId: "select-project", targetId: "project-b" },
    );
  } finally {
    await browser.close();
  }
});

test("tree exposes levels and supports arrow-key disclosure/navigation", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await contextWithState(browser, "builds");
    const page = await context.newPage();
    await page.goto(baseUrl);

    const tree = page.getByRole("tree").first();
    await page.getByRole("tab", { name: "输出" }).click();
    await tree.waitFor();
    const root = tree.getByRole("treeitem").first();
    assert.equal(await root.getAttribute("aria-level"), "1");
    assert.equal(await root.getAttribute("aria-expanded"), "true");
    await root.focus();
    await root.press("ArrowDown");
    assert.equal(await page.evaluate(() => document.activeElement?.getAttribute("data-hub-tree-node-id")), "profile");
    await page.keyboard.press("ArrowUp");
    assert.equal(await page.evaluate(() => document.activeElement?.getAttribute("data-hub-tree-node-id")), "builds");
    await page.keyboard.press("ArrowLeft");
    assert.equal(await root.getAttribute("aria-expanded"), "false");
    await page.keyboard.press("ArrowRight");
    await page.waitForFunction(() => document.querySelector('[data-hub-tree-node-id="profile"]')?.getAttribute("aria-level") === "2");
    assert.equal(await tree.getByRole("group").count(), 1);
  } finally {
    await browser.close();
  }
});

async function contextWithState(browser, route) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  await context.addInitScript(({ route }) => {
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
          state.backendEpoch = "accessibility-browser";
          state.stateRevision = String(revision);
          state.activePage = route;
          state.pageTitle = state.ui.shell.navItems.find((item) => item.id === route)?.label ?? route;
          if (route === "projects") {
            state.projectViewMode = "list";
            state.browserProjects = [
              { id: "project-a", name: "Project A", engineVersion: "1.0", modified: "today", location: "E:/projects/a", coverId: "empty", pinned: false },
              { id: "project-b", name: "Project B", engineVersion: "1.0", modified: "yesterday", location: "E:/projects/b", coverId: "empty", pinned: false },
            ];
            state.recentProjects = state.browserProjects;
          }
          return state;
        }
        if (command === "plugin:event|listen") {
          return 1;
        }
        if (command === "hub_action") {
          const request = args?.request ?? {};
          window.__HUB_TEST_ACTIONS__.push({ actionId: request.actionId, targetId: request.targetId });
          if (request.actionId === "select-project") {
            state = { ...state, selectedProjectId: request.targetId ?? null, stateRevision: String(++revision) };
          }
          return state;
        }
        return undefined;
      },
    };
  }, { route });
  return context;
}
