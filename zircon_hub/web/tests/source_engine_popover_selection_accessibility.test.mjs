import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import test from "node:test";
import path from "node:path";

const sourceRoot = new URL("../src/components/overlays/", import.meta.url);
const playwrightRoot = process.env.ZIRCON_PLAYWRIGHT_NODE_MODULES;
const require = createRequire(import.meta.url);
const { chromium } = playwrightRoot ? require(path.join(playwrightRoot, "playwright")) : { chromium: undefined };
const executablePath = process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH || (chromium ? chromium.executablePath() : undefined);
const browserAvailable = Boolean(chromium && executablePath && existsSync(executablePath));
const launchOptions = executablePath ? { executablePath } : {};
const baseUrl = process.env.ZIRCON_HUB_TEST_URL;

test("source engine popover rows publish their active selection state", async () => {
  const source = await readFile(new URL("SourceEnginePopover.tsx", sourceRoot), "utf8");

  assert.match(source, /<ButtonBase[\s\S]*?aria-pressed=\{active\}/);
});

test("source engine popover rows expose their active selection state in Chrome", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await contextWithState(browser);
    const page = await context.newPage();
    await page.goto(baseUrl);

    const trigger = page.locator('button[aria-controls="hub-source-engine-popover"]');
    await trigger.click();
    const popover = page.locator("#hub-source-engine-popover");
    await popover.waitFor();
    const active = popover.getByRole("button", { name: /Engine A/ });
    const fallback = popover.getByRole("button", { name: /Engine B/ });
    assert.equal(await active.getAttribute("aria-pressed"), "true");
    assert.equal(await fallback.getAttribute("aria-pressed"), "false");
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
          const state = structuredClone(fallbackShellState);
          state.backendEpoch = "engine-popover-accessibility";
          state.stateRevision = "1";
          state.sourceEngines = [
            { id: "engine-a", name: "Engine A", sourcePath: "E:/engine-a", outputPath: "E:/engine-a/out", status: "可用", active: true, buildHistory: [] },
            { id: "engine-b", name: "Engine B", sourcePath: "E:/engine-b", outputPath: "E:/engine-b/out", status: "备用", active: false, buildHistory: [] },
          ];
          state.activeSourceEngineId = "engine-a";
          return state;
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
