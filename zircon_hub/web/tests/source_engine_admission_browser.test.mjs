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

test("new-project creation stays disabled when the configured engine target is stale", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await context.addInitScript(() => {
      let callbackId = 0;
      const callbacks = new Map();
      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
      window.__TAURI_INTERNALS__ = {
        transformCallback(callback) {
          callbackId += 1;
          callbacks.set(callbackId, callback);
          return callbackId;
        },
        unregisterCallback(id) {
          callbacks.delete(id);
        },
        async invoke(command, args) {
          if (command === "hub_state") {
            const { fallbackShellState } = await import("/src/data/hubData.ts");
            const state = structuredClone(fallbackShellState);
            state.backendEpoch = "engine-admission-browser";
            state.stateRevision = "1";
            state.sourceEngines = [{
              id: "engine-a",
              name: "Engine A",
              sourcePath: "E:/engine-a",
              outputPath: "E:/engine-a/out",
              status: "可用",
              active: false,
              buildHistory: [],
            }];
            state.activeSourceEngineId = "missing-engine";
            window.__HUB_ENGINE_STATE__ = state;
            return state;
          }
          if (command === "plugin:event|listen") {
            window.__HUB_EVENT_HANDLER__ = args.handler;
            return 1;
          }
          if (command === "hub_action") {
            const next = structuredClone(window.__HUB_ENGINE_STATE__);
            next.stateRevision = String(Number(next.stateRevision) + 1);
            if (args.request?.actionId === "new-project") {
              next.projectSubpage = "new-project";
              next.activePage = "projects";
              next.pageTitle = next.ui.projects.title;
            }
            window.__HUB_ENGINE_STATE__ = next;
            return next;
          }
          return undefined;
        },
      };
      window.__HUB_PUBLISH_ENGINE_STATE__ = (state) => {
        callbacks.get(window.__HUB_EVENT_HANDLER__)?.({
          event: "hub-state-changed",
          id: 1,
          payload: state,
        });
      };
    });
    const page = await context.newPage();
    await page.goto(baseUrl);
    const header = page.getByTestId("hub-page-header");
    await header.getByRole("button", { name: "新建项目", exact: true }).click();
    const dialog = page.getByRole("dialog");
    await dialog.waitFor();
    await dialog.getByRole("textbox", { name: "项目名称" }).fill("Stale engine project");
    await dialog.getByRole("textbox", { name: "位置" }).fill("E:/Projects");
    const create = dialog.getByRole("button", { name: "创建项目", exact: true });
    assert.equal(await create.isDisabled(), true);

    await page.evaluate(() => {
      const next = structuredClone(window.__HUB_ENGINE_STATE__);
      next.stateRevision = String(Number(next.stateRevision) + 1);
      next.activeSourceEngineId = "engine-a";
      window.__HUB_PUBLISH_ENGINE_STATE__(next);
    });
    await assert.doesNotReject(() => create.waitFor({ state: "visible" }));
    await page.waitForFunction(() => !document.querySelector('[role="dialog"] button[disabled]') || false);
    assert.equal(await create.isDisabled(), false);
  } finally {
    await browser.close();
  }
});
