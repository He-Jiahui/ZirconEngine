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

test("initial projects route does not eagerly request unrelated page modules", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await context.addInitScript(() => {
      let state;
      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
      window.__TAURI_INTERNALS__ = {
        transformCallback() { return 1; },
        unregisterCallback() {},
        async invoke(command, args) {
          if (command === "hub_state") {
            const { fallbackShellState } = await import("/src/data/hubData.ts");
            state = { ...fallbackShellState, activePage: "projects", backendEpoch: "route-code-split", stateRevision: "1" };
            return state;
          }
          if (command === "plugin:event|listen") return 1;
          if (command === "hub_action") {
            const request = args?.request ?? {};
            if (request.actionId === "show-page" && request.targetId) {
              state = {
                ...state,
                activePage: request.targetId,
                pageTitle: state.ui.shell.navItems.find((item) => item.id === request.targetId)?.label ?? request.targetId,
                stateRevision: "2",
              };
            }
            return state;
          }
          return undefined;
        },
      };
    });
    const page = await context.newPage();
    const requests = [];
    page.on("request", (request) => requests.push(request.url()));
    await page.goto(baseUrl);
    await page.getByTestId("hub-navigation-drawer").waitFor();
    await page.getByTestId("hub-page-header").waitFor();

    assert.equal(requests.some((url) => /\/src\/pages\/SettingsPage\.tsx(?:\?|$)/.test(url)), false);
    await page.getByTestId("hub-navigation-drawer").getByRole("button", { name: "设置", exact: true }).click();
    await page.getByRole("heading", { name: "工具链、构建默认值与路径", exact: true }).waitFor();
    assert.equal(requests.some((url) => /\/src\/pages\/SettingsPage\.tsx(?:\?|$)/.test(url)), true);
  } finally {
    await browser.close();
  }
});
