import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
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

test("source engine rows publish their active selection state", async () => {
  const source = await readFile(new URL("components/data/SourceEngineList.tsx", root), "utf8");

  assert.match(source, /<ButtonBase[\s\S]*?aria-pressed=\{engine\.active\}/);
});

test("source engine rows expose their active selection state in Chrome", { skip: !baseUrl || !browserAvailable }, async () => {
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
            const state = structuredClone(fallbackShellState);
            state.activePage = "settings";
            state.pageTitle = "设置";
            state.backendEpoch = "source-engine-selection-accessibility";
            state.stateRevision = "1";
            state.sourceEngines = [
              { id: "engine-a", name: "Engine A", sourcePath: "E:/engine-a", outputPath: "E:/engine-a/out", status: "可用", active: true, buildHistory: [] },
              { id: "engine-b", name: "Engine B", sourcePath: "E:/engine-b", outputPath: "E:/engine-b/out", status: "未选中", active: false, buildHistory: [] },
            ];
            state.activeSourceEngineId = "engine-a";
            return state;
          }
          if (command === "plugin:event|listen") return 1;
          return undefined;
        },
      };
    });
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.getByRole("heading", { name: "工具链、构建默认值与路径", exact: true }).waitFor();
    const active = page.getByRole("button", { name: /Engine A/ }).last();
    const inactive = page.getByRole("button", { name: /Engine B/ }).last();
    assert.equal(await active.getAttribute("aria-pressed"), "true");
    assert.equal(await inactive.getAttribute("aria-pressed"), "false");
  } finally {
    await browser.close();
  }
});
