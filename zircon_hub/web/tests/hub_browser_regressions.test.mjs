import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { existsSync } from "node:fs";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const require = createRequire(import.meta.url);
const playwrightRoot = process.env.ZIRCON_PLAYWRIGHT_NODE_MODULES;
const { chromium } = playwrightRoot ? require(path.join(playwrightRoot, "playwright")) : { chromium: undefined };
const executablePath = process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH || (chromium ? chromium.executablePath() : undefined);
const browserAvailable = Boolean(chromium && executablePath && existsSync(executablePath));
const launchOptions = executablePath ? { executablePath } : {};

const baseUrl = process.env.ZIRCON_HUB_TEST_URL;
const screenshotDir = process.env.ZIRCON_HUB_SCREENSHOT_DIR;

test("Tauri bootstrap rejection exposes only recovery and dispatches no business action", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await browser.newContext({ viewport: { width: 768, height: 800 } });
    await context.addInitScript(() => {
      const calls = [];
      let callbackId = 0;
      window.__HUB_TEST_INVOKES__ = calls;
      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
      window.__TAURI_INTERNALS__ = {
        transformCallback(callback) {
          callbackId += 1;
          window[`__TAURI_CALLBACK_${callbackId}`] = callback;
          return callbackId;
        },
        unregisterCallback() {},
        async invoke(command) {
          calls.push(command);
          if (command === "hub_state") {
            throw new Error("backend unavailable");
          }
          if (command === "plugin:event|listen") {
            return 1;
          }
          return undefined;
        },
      };
    });
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.getByText("Hub backend unavailable", { exact: true }).waitFor();
    assert.equal(await page.getByTestId("hub-navigation-drawer").count(), 0);
    assert.deepEqual(
      await page.evaluate(() => window.__HUB_TEST_INVOKES__.filter((command) => command === "hub_action")),
      [],
    );
  } finally {
    await browser.close();
  }
});

test("delayed Hub event cannot overwrite a newer action response", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await context.addInitScript(() => {
      const callbacks = new Map();
      let callbackId = 0;
      let state;
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
            state = { ...fallbackShellState, backendEpoch: "browser-epoch", stateRevision: "1" };
            return state;
          }
          if (command === "plugin:event|listen") {
            window.__HUB_EVENT_HANDLER__ = args.handler;
            return 1;
          }
          if (command === "hub_action") {
            const delayed = { ...state, stateRevision: "2" };
            state = { ...state, activePage: "editor", pageTitle: "编辑器", stateRevision: "3" };
            setTimeout(() => {
              callbacks.get(window.__HUB_EVENT_HANDLER__)?.({
                event: "hub-state-changed",
                id: 1,
                payload: delayed,
              });
              window.__HUB_DELAYED_EVENT_DELIVERED__ = true;
            }, 50);
            return state;
          }
          return undefined;
        },
      };
    });
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.getByTestId("hub-navigation-drawer").waitFor();
    const drawer = page.getByTestId("hub-navigation-drawer");
    await drawer.getByRole("button", { name: "编辑器", exact: true }).click();
    await page.waitForFunction(() => window.__HUB_DELAYED_EVENT_DELIVERED__ === true);

    assert.equal(await drawer.getByRole("button", { name: "编辑器", exact: true }).getAttribute("aria-current"), "page");
    assert.equal(await drawer.getByRole("button", { name: "项目", exact: true }).getAttribute("aria-current"), null);
  } finally {
    await browser.close();
  }
});

test("malformed stale event cannot quarantine a newer ready Hub state", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await context.addInitScript(() => {
      const callbacks = new Map();
      let callbackId = 0;
      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
      window.__TAURI_INTERNALS__ = {
        transformCallback(callback) { callbacks.set(++callbackId, callback); return callbackId; },
        unregisterCallback(id) { callbacks.delete(id); },
        async invoke(command, args) {
          if (command === "hub_state") {
            const { fallbackShellState } = await import("/src/data/hubData.ts");
            return { ...fallbackShellState, backendEpoch: "event-epoch", stateRevision: "5" };
          }
          if (command === "plugin:event|listen") {
            window.__HUB_EMIT_EVENT__ = (payload) => callbacks.get(args.handler)?.({
              event: "hub-state-changed", id: 1, payload,
            });
            return 1;
          }
          return undefined;
        },
      };
    });
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.getByTestId("hub-navigation-drawer").waitFor();
    await page.waitForFunction(() => typeof window.__HUB_EMIT_EVENT__ === "function");
    await page.evaluate(async () => {
      window.__HUB_EMIT_EVENT__({
        backendEpoch: "event-epoch", stateRevision: "4", taskSummary: null,
      });
      await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
    });
    assert.equal(await page.getByTestId("hub-bootstrap-surface").count(), 0);
    assert.equal(await page.getByTestId("hub-navigation-drawer").count(), 1);

    await page.evaluate(() => window.__HUB_EMIT_EVENT__({
      backendEpoch: "unknown-epoch", stateRevision: "1", taskSummary: null,
    }));
    await page.getByText("Hub protocol mismatch", { exact: true }).waitFor();
  } finally {
    await browser.close();
  }
});

test("window action failure from an older epoch cannot overwrite the current task summary", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await context.addInitScript(() => {
      const callbacks = new Map();
      let callbackId = 0;
      let backendEpoch = "window-epoch-a";
      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
      window.__TAURI_INTERNALS__ = {
        metadata: { currentWindow: { label: "main" } },
        transformCallback(callback) { callbacks.set(++callbackId, callback); return callbackId; },
        unregisterCallback(id) { callbacks.delete(id); },
        async invoke(command, args) {
          if (command === "hub_state") {
            const { fallbackShellState } = await import("/src/data/hubData.ts");
            return { ...fallbackShellState, backendEpoch, stateRevision: backendEpoch.endsWith("a") ? "5" : "1" };
          }
          if (command === "plugin:event|listen") {
            window.__HUB_EMIT_EVENT__ = (payload) => callbacks.get(args.handler)?.({
              event: "hub-state-changed", id: 1, payload,
            });
            return 1;
          }
          if (command === "plugin:window|minimize") {
            return new Promise((_, reject) => { window.__HUB_REJECT_MINIMIZE__ = reject; });
          }
          return undefined;
        },
      };
      window.__HUB_SET_NEW_EPOCH__ = () => { backendEpoch = "window-epoch-b"; };
    });
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.getByTestId("hub-navigation-drawer").waitFor();
    const minimizeLabel = await page.evaluate(async () => (await import("/src/data/hubData.ts")).fallbackShellState.ui.shell.minimize);
    await page.getByRole("button", { name: minimizeLabel, exact: true }).click();
    await page.waitForFunction(() => typeof window.__HUB_REJECT_MINIMIZE__ === "function");
    await page.evaluate(() => {
      window.__HUB_SET_NEW_EPOCH__();
      window.__HUB_EMIT_EVENT__({ invalid: true });
    });
    await page.getByText("Hub protocol mismatch", { exact: true }).waitFor();
    await page.getByRole("button", { name: "Retry" }).click();
    await page.getByTestId("hub-navigation-drawer").waitFor();
    await page.evaluate(async () => {
      window.__HUB_REJECT_MINIMIZE__(new Error("old window action failed"));
      await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
    });
    assert.equal(await page.getByTestId("hub-task-summary-alert").count(), 0);
    assert.equal(await page.getByTestId("hub-navigation-drawer").count(), 1);
  } finally {
    await browser.close();
  }
});

test("event received during initial bootstrap is reconciled before the shell renders", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await context.addInitScript(() => {
      const callbacks = new Map();
      let callbackId = 0;
      let bootstrapReleased = false;
      let bootstrapState;
      let bootstrapEvent;
      const bootstrapResolvers = [];

      const loadBootstrapState = async () => {
        if (!bootstrapState) {
          const { fallbackShellState } = await import("/src/data/hubData.ts");
          bootstrapState = { ...fallbackShellState, backendEpoch: "bootstrap-epoch", stateRevision: "1" };
        }
        return bootstrapState;
      };

      const emitBootstrapEvent = async () => {
        const { fallbackShellState } = await import("/src/data/hubData.ts");
        bootstrapEvent = {
          event: "hub-state-changed",
          id: 1,
          payload: {
            ...fallbackShellState,
            backendEpoch: "bootstrap-epoch",
            stateRevision: "3",
            activePage: "editor",
            pageTitle: "编辑器",
          },
        };
        const handler = window.__HUB_EVENT_HANDLER__;
        if (typeof handler === "number") {
          callbacks.get(handler)?.(bootstrapEvent);
        }
      };

      window.__HUB_RESOLVE_BOOTSTRAP__ = () => {
        bootstrapReleased = true;
        for (const resolve of bootstrapResolvers.splice(0)) {
          resolve();
        }
      };
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
            window.__HUB_BOOTSTRAP_STARTED__ = true;
            const bootstrap = await loadBootstrapState();
            if (bootstrapReleased) {
              return bootstrap;
            }
            return new Promise((resolve) => {
              bootstrapResolvers.push(() => resolve(bootstrap));
            });
          }
          if (command === "plugin:event|listen") {
            window.__HUB_EVENT_HANDLER__ = args.handler;
            window.__HUB_EMIT_DURING_BOOTSTRAP__ = emitBootstrapEvent;
            // React.StrictMode mounts effects twice in the dev fixture. If the
            // event is emitted between those passes, replay it for the newest
            // listener so the second bootstrap cannot lose the staged state.
            if (bootstrapEvent) {
              queueMicrotask(() => callbacks.get(args.handler)?.(bootstrapEvent));
            }
            return 1;
          }
          return undefined;
        },
      };
    });
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.waitForFunction(
      () => window.__HUB_BOOTSTRAP_STARTED__ === true && typeof window.__HUB_EMIT_DURING_BOOTSTRAP__ === "function",
    );
    await page.evaluate(async () => {
      await window.__HUB_EMIT_DURING_BOOTSTRAP__();
      window.__HUB_RESOLVE_BOOTSTRAP__();
    });
    await page.getByTestId("hub-navigation-drawer").waitFor();

    const drawer = page.getByTestId("hub-navigation-drawer");
    assert.equal(await drawer.getByRole("button", { name: "编辑器", exact: true }).getAttribute("aria-current"), "page");
    assert.equal(await drawer.getByRole("button", { name: "项目", exact: true }).getAttribute("aria-current"), null);
  } finally {
    await browser.close();
  }
});

test("protocol mismatch cancels bootstrap so a late response cannot restore the shell", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await context.addInitScript(() => {
      const callbacks = new Map();
      let callbackId = 0;
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
            window.__HUB_RESOLVE_LATE_BOOTSTRAP__ = () => window.__HUB_BOOTSTRAP_RESOLVE__({
              ...fallbackShellState,
              backendEpoch: "late-epoch",
              stateRevision: "1",
            });
            return new Promise((resolve) => {
              window.__HUB_BOOTSTRAP_RESOLVE__ = resolve;
            });
          }
          if (command === "plugin:event|listen") {
            window.__HUB_EVENT_HANDLER__ = args.handler;
            return 1;
          }
          return undefined;
        },
      };
      window.__HUB_EMIT_PROTOCOL_MISMATCH__ = () => callbacks.get(window.__HUB_EVENT_HANDLER__)?.({
        event: "hub-state-changed",
        id: 1,
        payload: { invalid: true },
      });
    });
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.waitForFunction(() => typeof window.__HUB_EVENT_HANDLER__ === "number" && typeof window.__HUB_RESOLVE_LATE_BOOTSTRAP__ === "function");
    await page.evaluate(() => window.__HUB_EMIT_PROTOCOL_MISMATCH__());
    await page.getByTestId("hub-bootstrap-surface").waitFor();
    await page.evaluate(() => window.__HUB_RESOLVE_LATE_BOOTSTRAP__());
    await page.waitForTimeout(50);
    assert.equal(await page.getByTestId("hub-bootstrap-surface").count(), 1);
    assert.equal(await page.getByTestId("hub-navigation-drawer").count(), 0);
  } finally {
    await browser.close();
  }
});

test("malformed action state preserves the previously accepted App state", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await context.addInitScript(() => {
      let callbackId = 0;
      let initialState;
      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
      window.__TAURI_INTERNALS__ = {
        transformCallback() {
          callbackId += 1;
          return callbackId;
        },
        unregisterCallback() {},
        async invoke(command) {
          if (command === "hub_state") {
            if (!initialState) {
              const { fallbackShellState } = await import("/src/data/hubData.ts");
              initialState = { ...fallbackShellState, backendEpoch: "retry-epoch", stateRevision: "1" };
            }
            return initialState;
          }
          if (command === "plugin:event|listen") {
            return 1;
          }
          if (command === "hub_action") {
            return {
              ...initialState,
              stateRevision: "2",
              ui: {
                ...initialState.ui,
                shell: { ...initialState.ui.shell, navItems: null },
              },
            };
          }
          return undefined;
        },
      };
    });
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.getByTestId("hub-navigation-drawer").waitFor();
    const drawer = page.getByTestId("hub-navigation-drawer");
    await drawer.getByRole("button", { name: "编辑器", exact: true }).click();
    const failureAlert = page.getByTestId("hub-task-summary-alert");
    await failureAlert.waitFor();
    assert.equal(await page.getByTestId("hub-navigation-drawer").count(), 1);
    assert.equal(await page.getByTestId("hub-bootstrap-surface").count(), 0);
    assert.equal(await drawer.getByRole("button", { name: "项目", exact: true }).getAttribute("aria-current"), "page");
    assert.equal(await drawer.getByRole("button", { name: "编辑器", exact: true }).getAttribute("aria-current"), null);
    assert.equal(await failureAlert.getByText("检查操作目标后重试", { exact: true }).count(), 1);
  } finally {
    await browser.close();
  }
});

test("catalog route state and empty search stay truthful", { skip: !baseUrl || !browserAvailable }, async () => {
  await withPage({ width: 1280, height: 800 }, async (page) => {
    await page.goto(baseUrl);
    await page.getByTestId("hub-navigation-drawer").getByRole("button", { name: "学习", exact: true }).click();
    await page.getByRole("tab", { name: "指南" }).click();
    await page.getByTestId("hub-navigation-drawer").getByRole("button", { name: "资产", exact: true }).click();
    await assert.doesNotReject(() => page.getByRole("tab", { name: "全部" }).waitFor({ state: "visible" }));
    assert.equal(await page.getByRole("tab", { name: "全部" }).getAttribute("aria-selected"), "true");

    const search = page.getByRole("textbox", { name: /搜索/ });
    await search.fill("__no_catalog_match__");
    await assert.doesNotReject(() => page.getByText("未找到条目", { exact: true }).waitFor());
    assert.equal(await page.getByRole("button", { name: "打开资源" }).count(), 0);
  });
});

for (const viewport of [
  { width: 360, height: 760 },
  { width: 768, height: 800 },
  { width: 1280, height: 800 },
  { width: 1920, height: 1080 },
]) {
  test(`fixture layout is bounded at ${viewport.width}px`, { skip: !baseUrl || !browserAvailable }, async () => {
    await withPage(viewport, async (page) => {
      await page.goto(baseUrl);
      const activeNav = page.getByTestId("hub-navigation-drawer").getByRole("button", { name: "项目", exact: true });
      assert.equal(await activeNav.getAttribute("aria-current"), "page");
      await assertPageHeaderLayout(page);
      const overflow = await page.evaluate(() => ({
        document: document.documentElement.scrollWidth - document.documentElement.clientWidth,
        body: document.body.scrollWidth - document.body.clientWidth,
      }));
      assert.deepEqual(overflow, { document: 0, body: 0 });

      if (viewport.width <= 980) {
        const widths = await page.locator('[data-testid="hub-navigation-drawer"]').evaluate((drawer) => ({
          root: drawer.getBoundingClientRect().width,
          paper: drawer.querySelector(".MuiDrawer-paper")?.getBoundingClientRect().width,
        }));
        assert.equal(widths.root, widths.paper);
      }

      if (screenshotDir) {
        await mkdir(screenshotDir, { recursive: true });
        await page.screenshot({ path: path.join(screenshotDir, `hub-fixture-${viewport.width}.png`), fullPage: true });
      }
      await page.getByTestId("hub-navigation-drawer").getByRole("button", { name: "设置", exact: true }).click();
      await assertPageHeaderLayout(page);
      await assertRouteAndDialogHeaders(page, viewport);
    });
  });
}

async function withPage(viewport, run) {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const page = await browser.newPage({ viewport });
    const consoleErrors = [];
    page.on("console", (message) => {
      if (message.type() === "error") consoleErrors.push(message.text());
    });
    await run(page);
    assert.deepEqual(consoleErrors, [], `browser console errors: ${consoleErrors.join("\\n")}`);
  } finally {
    await browser.close();
  }
}

async function assertPageHeaderLayout(page) {
  const header = page.getByTestId("hub-page-header");
  await header.waitFor();
  const metrics = await header.evaluate((element) => {
    const heading = element.querySelector("h4");
    const style = getComputedStyle(heading);
    const rect = heading.getBoundingClientRect();
    const headerRect = element.getBoundingClientRect();
    return {
      lines: rect.height / Number.parseFloat(style.lineHeight),
      characters: heading.textContent.length,
      width: rect.width,
      insideHeader: rect.left >= headerRect.left && rect.right <= headerRect.right + 1,
      documentOverflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    };
  });
  assert.ok(metrics.lines <= (metrics.characters <= 6 ? 1.1 : 2.1), `heading must wrap as a readable phrase: ${JSON.stringify(metrics)}`);
  assert.ok(metrics.width >= 80, `heading must have readable horizontal space: ${JSON.stringify(metrics)}`);
  assert.equal(metrics.insideHeader, true);
  assert.equal(metrics.documentOverflow, 0);
  await assertControlBounds(page.locator("main"));
}

async function assertControlBounds(container) {
  const overflow = await container.evaluate((element) => {
    const bounds = element.getBoundingClientRect();
    return [...element.querySelectorAll('input, [role="combobox"], button, .MuiTextField-root, .MuiInputBase-root')]
      .filter((control) => control.getClientRects().length > 0)
      .map((control) => {
        const rect = control.getBoundingClientRect();
        return { label: control.getAttribute("placeholder") || control.textContent, left: rect.left, right: rect.right, containerLeft: bounds.left, containerRight: bounds.right };
      })
      .filter((rect) => rect.left < bounds.left - 1 || rect.right > bounds.right + 1);
  });
  assert.deepEqual(overflow, [], `controls must fit their visible container: ${JSON.stringify(overflow)}`);
}

async function settleDialog(dialog) {
  await dialog.evaluate(async (element) => {
    const root = element.closest(".MuiDialog-root");
    if (root) await Promise.all(root.getAnimations({ subtree: true }).map((animation) => animation.finished.catch(() => {})));
    await document.fonts.ready;
  });
}

async function assertRouteAndDialogHeaders(page, viewport) {
  await page.addInitScript(() => {
    let state;
    let revision = 1;
    const callbacks = new Map();
    let callbackId = 0;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback(callback) { callbacks.set(++callbackId, callback); return callbackId; },
      unregisterCallback(id) { callbacks.delete(id); },
      async invoke(command, args) {
        if (command === "hub_state") {
          const { fallbackShellState } = await import("/src/data/hubData.ts");
          state ??= { ...fallbackShellState, backendEpoch: "layout-matrix", stateRevision: "1" };
          return state;
        }
        if (command === "plugin:event|listen") {
          window.__HUB_LAYOUT_STATE__ = async (route, language) => {
            const { fallbackShellState } = await import("/src/data/hubData.ts");
            state = structuredClone(fallbackShellState);
            const labels = { projects: "Projects", editor: "Editor", assets: "Assets", builds: "Builds", plugins: "Plugins", cloud: "Local Delivery", team: "Team", learn: "Learning", settings: "Settings" };
            state.backendEpoch = "layout-matrix";
            state.stateRevision = String(++revision);
            state.activePage = route.startsWith("project-") || route === "new-project" ? "projects" : route;
            state.projectSubpage = route.startsWith("project-") || route === "new-project" ? route : "dashboard";
            state.pageTitle = state.ui.shell.navItems.find((item) => item.id === state.activePage)?.label ?? route;
            if (language === "en") {
              state.pageTitle = labels[state.activePage];
              state.ui.projects.title = "Projects";
              state.ui.projects.browserTitle = "Project Browser";
              state.ui.projects.newProjectDialog = "Create Project";
              state.settings.text.heading = "Toolchain, Build Defaults and Paths";
              if (state.settingsDraft) state.settingsDraft.text.heading = state.settings.text.heading;
              Object.assign(state.ui.actions, { importProject: "Import Project", newProject: "New Project", dashboard: "Dashboard", browser: "Project Browser", openEditor: "Open Editor", close: "Close", createProject: "Create Project" });
            }
            callbacks.get(args.handler)?.({ event: "hub-state-changed", id: 1, payload: state });
            await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
          };
          return 1;
        }
        return undefined;
      },
    };
  });
  await page.reload();
  await page.waitForFunction(() => typeof window.__HUB_LAYOUT_STATE__ === "function");
  for (const language of ["zh", "en"]) {
    for (const route of ["projects", "editor", "assets", "builds", "plugins", "cloud", "team", "learn", "settings", "project-browser", "project-detail"]) {
      await page.evaluate(([route, language]) => window.__HUB_LAYOUT_STATE__(route, language), [route, language]);
      await page.waitForFunction(() => !document.querySelector('[data-testid="hub-bootstrap-surface"]'));
      await assertPageHeaderLayout(page);
    }
    await page.evaluate((language) => window.__HUB_LAYOUT_STATE__("new-project", language), language);
    const dialog = page.getByRole("dialog");
    await dialog.waitFor();
    await settleDialog(dialog);
    const bounds = await dialog.evaluate((element) => {
      const rect = element.getBoundingClientRect();
      return { left: rect.left, right: rect.right, overflow: element.scrollWidth - element.clientWidth };
    });
    assert.ok(bounds.left >= 0 && bounds.right <= viewport.width);
    assert.equal(bounds.overflow, 0);
    await assertControlBounds(dialog);
    if (screenshotDir) {
      await page.screenshot({ path: path.join(screenshotDir, `hub-dialog-${language}-${viewport.width}.png`), fullPage: true });
    }
    await page.evaluate((language) => window.__HUB_LAYOUT_STATE__("projects", language), language);
    await dialog.waitFor({ state: "hidden" });
  }
}
