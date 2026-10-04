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

test("HubDialog exposes stable labelledby and describedby relationships", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await contextWithState(browser);
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.getByTestId("hub-page-header").getByRole("button", { name: "新建项目", exact: true }).click();

    const dialog = page.getByRole("dialog");
    await dialog.waitFor();
    const labelledBy = await dialog.getAttribute("aria-labelledby");
    const describedBy = await dialog.getAttribute("aria-describedby");
    assert.ok(labelledBy);
    assert.ok(describedBy);
    assert.equal(await page.locator(`#${labelledBy}`).textContent(), "新建项目");
    assert.ok((await page.locator(`#${describedBy}`).textContent())?.includes("项目名称"));
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
          state.backendEpoch = "dialog-accessibility-browser";
          state.stateRevision = String(revision);
          state.sourceEngines = [{
            id: "engine-a",
            name: "Engine A",
            sourcePath: "E:/engine-a",
            outputPath: "E:/engine-a/out",
            status: "可用",
            active: true,
            buildHistory: [],
          }];
          state.activeSourceEngineId = "engine-a";
          return state;
        }
        if (command === "plugin:event|listen") return 1;
        if (command === "hub_action") {
          state = {
            ...state,
            stateRevision: String(++revision),
            projectSubpage: args?.request?.actionId === "new-project" ? "new-project" : state.projectSubpage,
          };
          return state;
        }
        return undefined;
      },
    };
  });
  return context;
}
