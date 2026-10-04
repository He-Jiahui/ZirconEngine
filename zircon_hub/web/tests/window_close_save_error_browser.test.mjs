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
const baseUrl = process.env.ZIRCON_HUB_TEST_URL;

test("window close save error stays visible without replacing a cancellable build", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, executablePath });
  try {
    const longDetail = Array.from({ length: 600 }, (_, index) => `C:\\Projects\\VeryLongFolder${index}\\hub-config.json`).join(" ");
    for (const viewport of [
      { width: 360, height: 640 },
      { width: 768, height: 768 },
      { width: 960, height: 680 },
      { width: 1280, height: 800 },
      { width: 1920, height: 1080 },
    ]) {
      const context = await browser.newContext({ viewport });
      await context.addInitScript((detail) => {
        window.__hubActionCalls = [];
        window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
        window.__TAURI_INTERNALS__ = {
          transformCallback() { return 1; },
          unregisterCallback() {},
          async invoke(command, args) {
            if (command === "hub_state") {
              const { fallbackShellState } = await import("/src/data/hubData.ts");
              window.__hubShellState = {
                ...fallbackShellState,
                activePage: "builds",
                backendEpoch: "window-close-save-error",
                stateRevision: "1",
                taskSummary: {
                  ...fallbackShellState.taskSummary,
                  label: "Building",
                  detail: "Build in progress",
                  operation: "Build project",
                  running: true,
                  cancellable: true,
                  taskId: 17,
                  progressPercent: 42,
                },
                windowCloseSaveError: {
                  label: "Save Hub state failed",
                  detail,
                  recovery: "Check the Hub config path and retry the action",
                },
              };
              return window.__hubShellState;
            }
            if (command === "hub_action") {
              window.__hubActionCalls.push(args);
              window.__hubShellState = { ...window.__hubShellState, stateRevision: "2" };
              return window.__hubShellState;
            }
            if (command === "plugin:event|listen") return 1;
            return undefined;
          },
        };
      }, longDetail);

      const page = await context.newPage();
      const consoleErrors = [];
      page.on("console", (message) => {
        if (message.type() === "error") consoleErrors.push(message.text());
      });
      await page.goto(baseUrl);
      const saveError = page.getByRole("alert").filter({ hasText: "Save Hub state failed" });
      await saveError.waitFor();
      assert.match(await saveError.innerText(), /Check the Hub config path/);
      assert.equal(await page.getByText("Building", { exact: true }).count(), 1);

      const cancelLabel = await page.evaluate(async () => (await import("/src/data/hubData.ts")).fallbackShellState.ui.common.cancelTask);
      const cancel = page.getByRole("button", { name: cancelLabel, exact: true });
      await cancel.waitFor();
      await cancel.scrollIntoViewIfNeeded();
      const alertBox = await saveError.boundingBox();
      const cancelBox = await cancel.boundingBox();
      assert.ok(alertBox && cancelBox && alertBox.y + alertBox.height <= cancelBox.y + 0.5, `save error overlaps cancellation at ${viewport.width}px`);
      assert.equal(await saveError.getAttribute("tabindex"), "0", `save error keyboard focus at ${viewport.width}px`);
      const alertMetrics = await saveError.evaluate((node) => ({ clientHeight: node.clientHeight, scrollHeight: node.scrollHeight }));
      assert.ok(alertMetrics.scrollHeight > alertMetrics.clientHeight, `long error should scroll at ${viewport.width}px: ${JSON.stringify(alertMetrics)}`);
      await saveError.focus();
      await page.keyboard.press("End");
      assert.ok(await saveError.evaluate((node) => node.scrollTop > 0), `keyboard should scroll the error at ${viewport.width}px`);

      await cancel.click();
      const actions = await page.evaluate(() => window.__hubActionCalls);
      assert.equal(actions.at(-1)?.request?.actionId, "cancel-background-task");
      assert.equal(actions.at(-1)?.request?.targetId, "17");
      assert.deepEqual(consoleErrors, [], `browser console errors at ${viewport.width}px`);
      await context.close();
    }
  } finally {
    await browser.close();
  }
});
