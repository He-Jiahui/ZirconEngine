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

test("comboboxes expose localized names in the create-project dialog", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await contextWithState(browser);
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.getByTestId("hub-page-header").getByRole("button", { name: "新建项目", exact: true }).click();

    const dialog = page.getByRole("dialog");
    await dialog.waitFor();
    const engine = dialog.getByRole("combobox", { name: "源码引擎", exact: true });
    const template = dialog.getByRole("combobox", { name: "模板", exact: true });
    assert.equal(await engine.count(), 1);
    assert.equal(await template.count(), 1);
    assert.equal(await engine.getAttribute("placeholder"), "源码引擎");
    assert.equal(await template.getAttribute("placeholder"), "模板");
  } finally {
    await browser.close();
  }
});

test("settings comboboxes expose their localized field names", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await contextWithState(browser, "settings");
    const page = await context.newPage();
    await page.goto(baseUrl);

    const settings = page.getByTestId("hub-navigation-drawer");
    await settings.getByRole("button", { name: "设置", exact: true }).waitFor();
    const buildProfile = page.getByRole("combobox", { name: "构建配置", exact: true });
    const language = page.getByRole("combobox", { name: "语言", exact: true });
    await buildProfile.waitFor();
    await language.waitFor();
    assert.equal(await buildProfile.count(), 1);
    assert.equal(await language.count(), 1);
  } finally {
    await browser.close();
  }
});

async function contextWithState(browser, route = "projects") {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  await context.addInitScript(({ route }) => {
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
          state.backendEpoch = "combobox-accessibility-browser";
          state.stateRevision = String(revision);
          state.activePage = route;
          state.pageTitle = state.ui.shell.navItems.find((item) => item.id === route)?.label ?? route;
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
        if (command === "plugin:event|listen") {
          callbacks.set(args.handler, () => {});
          return args.handler;
        }
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
  }, { route });
  return context;
}
