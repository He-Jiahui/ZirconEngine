import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { readFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import path from "node:path";
import test from "node:test";

const root = new URL("../src/", import.meta.url);
const require = createRequire(import.meta.url);
const playwrightRoot = process.env.ZIRCON_PLAYWRIGHT_NODE_MODULES;
const { chromium } = playwrightRoot ? require(path.join(playwrightRoot, "playwright")) : { chromium: undefined };
const executablePath = process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH || (chromium ? chromium.executablePath() : undefined);
const browserAvailable = Boolean(chromium && executablePath && existsSync(executablePath));
const launchOptions = executablePath ? { executablePath } : {};
const baseUrl = process.env.ZIRCON_HUB_TEST_URL;

test("settings health progress exposes the localized completeness label", async () => {
  const source = await readFile(new URL("components/data/SettingsSection.tsx", root), "utf8");

  assert.match(
    source,
    /<LinearProgress\s+aria-label=\{settingsText\.completenessLabel\}\s+variant=\"determinate\"\s+value=\{draftSettings\.health\.completion\}\s*\/>/,
  );
});

test("settings health progress exposes its localized name in Chrome", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
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
            return {
              ...structuredClone(fallbackShellState),
              activePage: "settings",
              pageTitle: "设置",
              backendEpoch: "settings-progress-accessibility",
              stateRevision: "1",
            };
          }
          if (command === "plugin:event|listen") return 1;
          return undefined;
        },
      };
    });
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.getByRole("heading", { name: "工具链、构建默认值与路径", exact: true }).waitFor();
    const progress = page.getByRole("progressbar", { name: "完整度", exact: true });
    assert.equal(await progress.count(), 1);
    assert.equal(await progress.getAttribute("aria-valuenow"), "100");
  } finally {
    await browser.close();
  }
});
