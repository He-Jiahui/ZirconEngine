import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import test from "node:test";
import path from "node:path";

const sourceRoot = new URL("../src/components/data/", import.meta.url);
const playwrightRoot = process.env.ZIRCON_PLAYWRIGHT_NODE_MODULES;
const require = createRequire(import.meta.url);
const { chromium } = playwrightRoot ? require(path.join(playwrightRoot, "playwright")) : { chromium: undefined };
const executablePath = process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH || (chromium ? chromium.executablePath() : undefined);
const browserAvailable = Boolean(chromium && executablePath && existsSync(executablePath));
const launchOptions = executablePath ? { executablePath } : {};
const baseUrl = process.env.ZIRCON_HUB_TEST_URL;

test("interactive HubList rows have a listbox owner and stable selection semantics", async () => {
  const source = await readFile(new URL("HubList.tsx", sourceRoot), "utf8");

  assert.match(source, /ariaLabel\?: string/);
  assert.match(source, /role=\{hasSelectHandler \? "listbox" : undefined\}/);
  assert.match(source, /aria-label=\{hasSelectHandler \? \(ariaLabel \?\? "可选项目列表"\) : undefined\}/);
  assert.match(source, /role=\{hasSelectHandler \? "option" : undefined\}/);
  assert.match(source, /aria-selected=\{hasSelectHandler \? Boolean\(item\.selected\) : undefined\}/);
});

test("interactive catalog rows form one listbox with selected options in Chrome", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await contextWithCatalogState(browser);
    const page = await context.newPage();
    await page.goto(baseUrl);

    const listbox = page.getByRole("listbox").first();
    await listbox.waitFor();
    assert.equal(await listbox.getAttribute("aria-label"), "可选项目列表");
    const options = listbox.getByRole("option");
    assert.ok(await options.count() > 0);
    assert.equal(await options.first().getAttribute("aria-selected"), "true");
  } finally {
    await browser.close();
  }
});

async function contextWithCatalogState(browser) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  await context.addInitScript(() => {
    let callbackId = 0;
    let state;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback() {
        return ++callbackId;
      },
      unregisterCallback() {},
      async invoke(command) {
        if (command === "hub_state") {
          const { fallbackShellState } = await import("/src/data/hubData.ts");
          state = structuredClone(fallbackShellState);
          state.backendEpoch = "hub-listbox-accessibility";
          state.stateRevision = "1";
          state.activePage = "assets";
          state.pageTitle = state.ui.shell.navItems.find((item) => item.id === "assets")?.label ?? "assets";
          state.assets = [
            { id: "asset-a", name: "Asset A", detail: "Primary", size: "1 KB", kind: "Texture", source: "Project", sourceKey: "project", path: "E:/assets/a" },
            { id: "asset-b", name: "Asset B", detail: "Secondary", size: "2 KB", kind: "Mesh", source: "Engine", sourceKey: "engine", path: "E:/assets/b" },
          ];
          return state;
        }
        if (command === "plugin:event|listen") {
          return 1;
        }
        return state;
      },
    };
  });
  return context;
}
