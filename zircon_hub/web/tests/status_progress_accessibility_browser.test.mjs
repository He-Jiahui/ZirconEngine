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

test("running task progress exposes the operation as its accessible name", { skip: !baseUrl || !browserAvailable }, async () => {
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
            return {
              ...fallbackShellState,
              activePage: "builds",
              backendEpoch: "status-progress",
              stateRevision: "1",
              taskSummary: {
                ...fallbackShellState.taskSummary,
                label: "构建中",
                detail: "正在构建编辑器运行时",
                operation: "构建任务",
                running: true,
                cancellable: false,
                progressPercent: 42,
              },
            };
          }
          if (command === "plugin:event|listen") return 1;
          return undefined;
        },
      };
    });
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.getByTestId("hub-navigation-drawer").waitFor();
    const main = page.getByRole("main");
    await main.getByRole("tab", { name: "工作流", exact: true }).waitFor();
    await main.getByText("构建中", { exact: true }).waitFor();

    const progress = page.getByRole("progressbar", { name: "构建任务", exact: true });
    assert.equal(await progress.count(), 1);
    assert.equal(await progress.getAttribute("aria-valuenow"), "42");
  } finally {
    await browser.close();
  }
});
