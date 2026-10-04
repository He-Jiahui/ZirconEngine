import assert from "node:assert/strict";
import test from "node:test";
import { createRequire } from "node:module";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import { accountAdminShell } from "./account_admin_shell.mjs";

const require = createRequire(import.meta.url);
const playwrightRoot = process.env.ZIRCON_PLAYWRIGHT_NODE_MODULES;
const chromium = playwrightRoot ? require(path.join(playwrightRoot, "playwright")).chromium : undefined;
const launchOptions = process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH ? { executablePath: process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH } : {};
const browserUrl = process.env.ZIRCON_HUB_TEST_URL;
const screenshotDir = process.env.ZIRCON_HUB_SCREENSHOT_DIR;
const enabled = Boolean(chromium && browserUrl);
const orgA = "00000000-0000-4000-8000-000000000001";
const orgB = "00000000-0000-4000-8000-000000000002";
const pkgA = "00000000-0000-4000-8000-000000000011";
const pkgB = "00000000-0000-4000-8000-000000000012";
const priorOperation = "00000000-0000-4000-8000-000000000099";
const labels = language => language === "Chinese" ? {
  installTarget: "安装目标", editorHostTarget: "编辑器宿主", clientRuntimeTarget: "客户端运行时",
  service: "服务目录", local: "本地库", organization: "组织", search: "搜索包", searchAction: "搜索", review: "查看许可", accept: "接受许可", consent: "我代表所选组织接受此许可。", cancel: "取消", install: "安装", installed: "已安装", notInstalled: "未安装", more: "加载更多", check: "查询结果", retry: "重试", dismiss: "关闭", unknown: "操作结果未知", inventory: "安装清单", refreshOrganization: "刷新组织", signOut: "退出登录", signIn: "登录", signedOut: "尚未登录", refreshCatalog: "刷新列表: 服务目录",
} : {
  installTarget: "Install target", editorHostTarget: "Editor host", clientRuntimeTarget: "Client runtime",
  service: "Service catalog", local: "Local library", organization: "Organization", search: "Search packages", searchAction: "Search", review: "Review license", accept: "Accept license", consent: "I accept this license for the selected organization.", cancel: "Cancel", install: "Install", installed: "Installed", notInstalled: "Not installed", more: "Load more", check: "Check result", retry: "Retry", dismiss: "Dismiss", unknown: "Operation outcome unknown", inventory: "Installation inventory", refreshOrganization: "Refresh organization", signOut: "Sign out", signIn: "Sign in", signedOut: "Signed out", refreshCatalog: "Refresh list: Service catalog",
};

// Native IPC is replaced only to exercise frontend contracts. These tests do not install a product package.
async function fixture(browser, { width = 1280, language = "English", role = "owner", restored = false, alreadyLicensed = false, mode = "plugins", targetMode = "editor_host" } = {}) {
  const context = await browser.newContext({ viewport: { width, height: 900 } });
  const shellText = await accountAdminShell(language);
  await context.addInitScript(({ language, role, restored, alreadyLicensed, mode, targetMode, orgA, orgB, pkgA, pkgB, priorOperation, shellText }) => {
    let generation = 7;
    let signedIn = true;
    let operationsRevision = 0;
    const initial = { action: "package-install", targetMode, operationId: priorOperation, organizationId: orgA, status: "unknown", error: "account_service_outcome_unknown" };
    let operations = restored ? [initial] : [];
    const requestsByOperation = new Map(restored ? [[priorOperation, { action: "package-install", schemaVersion: 2, targetMode, organization: orgA, packageId: pkgA, revision: "9007199254740993" }]] : []);
    const licensed = new Set(alreadyLicensed || restored ? [orgA] : []);
    const inventories = new Map();
    const inventoryKey = request => `${request.organization}:${request.targetMode}`;
    let shell;
    window.__CATALOG_REQUESTS__ = [];
    window.__CATALOG_OUTCOME__ = "committed";
    window.__CATALOG_RECONCILE__ = "unknown";
    window.__CATALOG_INVENTORY_ERROR__ = false;
    window.__CATALOG_HOLD_INVENTORY__ = false;
    window.__CATALOG_FLUSH_INVENTORY__ = null;
    const release = packageId => ({ iss: "https://catalog.test", aud: "zircon-hub", sub: "Zircon Publisher", exp: 4102444800, package_id: packageId, revision: "9007199254740993", version: "1.2.3", name: packageId === pkgA ? "Terrain and landscape authoring tools" : "Material editing tools", kind: mode === "plugins" ? "plugin" : "asset", description: packageId === pkgA ? "Terrain tools for landscape and scene editing." : "Material authoring tools.", license_id: "MIT", license_text: "Permission is hereby granted, free of charge, to any person obtaining a copy of this software.\n\nTHE SOFTWARE IS PROVIDED AS IS.", artifact_digest: "ab".repeat(32), artifact_size: 4194304 });
    const snapshot = (data = null, error = null) => ({ backendEpoch: "catalog-browser", account: { configured: true, status: signedIn ? "signed-in" : "signed-out", issuer: signedIn ? "https://issuer.test" : null, subject: signedIn ? "owner" : null, displayName: signedIn ? "Organization Owner" : null, generation: String(generation), error: null }, data, error, operations: signedIn ? operations : [], operationsRevision: signedIn ? String(operationsRevision) : "0", operationsError: null });
    const apply = (request, operationId) => {
      if (request.action === "catalog-license") licensed.add(request.organization);
      if (request.action === "package-install") inventories.set(inventoryKey(request), { schemaVersion: 2, targetMode: request.targetMode, revision: "1", packages: [{ operationId, packageId: request.packageId, version: "1.2.3", releaseRevision: request.revision, artifactDigest: release(pkgA).artifact_digest, files: ["native-private"], identityDigest: "native-private" }] });
    };
    const complete = (request, status) => {
      const original = requestsByOperation.get(request.operationId) ?? request;
      requestsByOperation.set(request.operationId, original);
      const error = status === "unknown" ? "account_service_outcome_unknown" : status === "failed" ? "account_service_operation_failed" : null;
      operations = [...operations.filter(item => item.operationId !== request.operationId), { operationId: request.operationId, action: original.action, ...(original.targetMode ? { targetMode: original.targetMode } : {}), organizationId: original.organization, status, error }];
      operationsRevision++;
      if (status === "committed") apply(original, request.operationId);
      const data = status !== "committed" ? null : original.action === "catalog-license" ? { packageId: original.packageId, revision: original.revision } : { schemaVersion: 2, targetMode: original.targetMode, operationId: request.operationId, inventoryRevision: "1", package: inventories.get(inventoryKey(original)).packages[0] };
      return snapshot(data, error);
    };
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback() { return 1; }, unregisterCallback() {},
      async invoke(command, args) {
        if (command === "plugin:event|listen") return 1;
        if (command === "hub_state") {
          const { fallbackShellState } = await import("/src/data/hubData.ts");
          shell = { ...fallbackShellState, backendEpoch: "catalog-browser", stateRevision: "1", activePage: mode, pageTitle: shellText.ui.shell.navItems.find(item => item.id === mode).label, pageSubtitle: "", ui: shellText.ui, comingSoon: [], settings: { ...fallbackShellState.settings, language },
            taskSummary: { ...fallbackShellState.taskSummary, label: shellText.ui.common.ready, detail: shellText.readyDetail },
            plugins: [{ id: "local-plugin", displayName: "Local workspace extension", description: "Local source package", moduleCount: 1, category: "Local", scope: "Project", scopeKey: "project", manifestPath: "E:/workspace/plugin.toml", packageRoot: "E:/workspace", maturity: "Source", maturityTone: "neutral", editorScoped: false, defaultPackaging: [] }] };
          return shell;
        }
        if (command === "account_state") return snapshot();
        if (command !== "account_action") return undefined;
        const request = args.request;
        if (["package-install", "package-inventory"].includes(request.action) && (request.schemaVersion !== 2 || !["editor_host", "client_runtime"].includes(request.targetMode))) throw new Error("invalid v2 package target request");
        window.__CATALOG_REQUESTS__.push(request);
        if (request.action === "logout") { signedIn = false; generation++; return snapshot(); }
        if (request.action === "sign-in") { signedIn = true; generation++; return snapshot(); }
        if (request.action === "refresh") { generation++; return snapshot(); }
        if (request.action === "organizations") return snapshot({ items: [{ id: orgA, name: "First organization with a long display name", policyRevision: "3" }, { id: orgB, name: "Second organization", policyRevision: "4" }], nextCursor: null });
        if (request.action === "members") return snapshot({ items: [{ issuer: "https://issuer.test", subject: "owner", role, active: true }], nextCursor: null });
        if (["projects", "invitations", "issued-invitations"].includes(request.action)) return snapshot({ items: [], nextCursor: null });
        if (request.action === "catalog") return snapshot(request.query ? { items: request.query.toLowerCase().includes("material") ? [release(pkgB)] : [], nextCursor: null } : request.after ? { items: [release(pkgB)], nextCursor: null } : { items: [release(pkgA)], nextCursor: pkgA });
        if (request.action === "catalog-entitlements") return snapshot({ items: licensed.has(request.organization) ? [{ packageId: pkgA, revision: release(pkgA).revision, licenseId: "MIT" }] : [], nextCursor: null });
        if (request.action === "package-inventory") {
          const response = window.__CATALOG_INVENTORY_ERROR__ ? snapshot(null, "account_service_operation_failed") : snapshot(inventories.get(inventoryKey(request)) ?? { schemaVersion: 2, targetMode: request.targetMode, revision: "0", packages: [] });
          if (window.__CATALOG_HOLD_INVENTORY__) return new Promise(resolve => { window.__CATALOG_FLUSH_INVENTORY__ = () => resolve(response); });
          return response;
        }
        if (request.action === "catalog-license" || request.action === "package-install") return complete(request, window.__CATALOG_OUTCOME__);
        if (request.action === "reconcile") { const response = complete(request, window.__CATALOG_RECONCILE__); return { ...response, data: window.__CATALOG_RECONCILE__ === "committed" ? { status: "committed", result: response.data } : { status: "unknown" } }; }
        if (request.action === "retry") return complete(request, "committed");
        if (request.action === "acknowledge") { operations = operations.filter(item => item.operationId !== request.operationId); operationsRevision++; return snapshot(); }
        throw new Error("unexpected catalog fixture request");
      },
    };
  }, { language, role, restored, alreadyLicensed, mode, targetMode, orgA, orgB, pkgA, pkgB, priorOperation, shellText });
  const page = await context.newPage();
  const errors = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.goto(browserUrl);
  const text = labels(language);
  await page.getByRole("tab", { name: text.local, exact: true }).waitFor();
  if (mode === "plugins") assert.ok(await page.getByText("Local workspace extension", { exact: true }).count() > 0);
  await page.getByRole("tab", { name: text.service, exact: true }).click();
  const catalog = page.getByTestId("service-catalog");
  await catalog.getByRole("combobox", { name: text.organization, exact: true }).click();
  await page.getByRole("option", { name: "First organization with a long display name", exact: true }).click();
  if (mode === "plugins" && targetMode === "client_runtime") {
    await catalog.getByRole("combobox", { name: text.installTarget, exact: true }).click();
    await page.getByRole("option", { name: text.clientRuntimeTarget, exact: true }).click();
  }
  if (restored) await catalog.getByTestId(`account-operation-${priorOperation}`).waitFor();
  else if (mode === "plugins") await catalog.getByTestId("service-release").getByText(text.notInstalled, { exact: true }).waitFor();
  else await catalog.getByTestId("service-release").waitFor();
  return { context, page, catalog, text, errors };
}

async function noOverflow(page, root) {
  await page.evaluate(async () => { await document.fonts.ready; await Promise.all(document.getAnimations().map(animation => animation.finished.catch(() => {}))); });
  const bad = await root.evaluate(element => [...element.querySelectorAll("button,input,p,h6,label,dt,dd,[role=combobox]")].filter(node => { const rect = node.getBoundingClientRect(); const style = getComputedStyle(node); return rect.width > 0 && style.opacity !== "0" && style.visibility !== "hidden" && (rect.left < -1 || rect.right > innerWidth + 1 || node.scrollWidth > node.clientWidth + 2); }).map(node => ({ tag: node.tagName, text: node.textContent, className: node.className, clientWidth: node.clientWidth, scrollWidth: node.scrollWidth, rect: node.getBoundingClientRect().toJSON() })));
  if (bad.length && screenshotDir) { await mkdir(screenshotDir, { recursive: true }); await page.screenshot({ path: path.join(screenshotDir, `catalog-overflow-${page.viewportSize().width}.png`) }); }
  assert.deepEqual(bad, []);
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
}

for (const language of ["English", "Chinese"]) test(`license confirmation and receipt-backed installation UI in ${language}`, { skip: !enabled }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const { page, catalog, text, errors } = await fixture(browser, { language, width: language === "Chinese" ? 360 : 1280 });
    assert.equal(await page.getByText("Local workspace extension", { exact: true }).count(), 0);
    assert.equal(await catalog.getByRole("button", { name: text.install, exact: true }).isDisabled(), true);
    await catalog.getByRole("button", { name: text.review, exact: true }).click();
    let dialog = page.getByRole("dialog");
    assert.equal(await dialog.getByRole("button", { name: text.accept, exact: true }).isDisabled(), true);
    await dialog.getByRole("button", { name: text.cancel, exact: true }).click();
    assert.equal(await page.evaluate(() => window.__CATALOG_REQUESTS__.some(item => item.action === "catalog-license")), false);
    await catalog.getByRole("button", { name: text.review, exact: true }).click();
    dialog = page.getByRole("dialog");
    await dialog.getByRole("checkbox", { name: text.consent, exact: true }).check();
    await noOverflow(page, dialog);
    await dialog.getByRole("button", { name: text.accept, exact: true }).click();
    await dialog.waitFor({ state: "detached" });
    await catalog.getByRole("button", { name: text.install, exact: true }).click();
    await catalog.getByTestId("service-release").getByText(text.installed, { exact: true }).waitFor();
    const actions = await page.evaluate(() => window.__CATALOG_REQUESTS__.filter(item => item.action === "catalog-license" || item.action === "package-install"));
    assert.equal(actions.length, 2);
    assert.equal(actions[0].expectedPolicyRevision, "3");
    assert.equal(actions[1].expectedInventoryRevision, "0");
    assert.equal(actions[1].revision, "9007199254740993");
    assert.equal(actions[1].backendEpoch, "catalog-browser");
    assert.equal(actions[1].schemaVersion, 2);
    assert.equal(actions[1].targetMode, "editor_host");
    assert.notEqual(actions[0].operationId, actions[1].operationId);
    assert.equal(await catalog.getByRole("button", { name: text.install, exact: true }).isDisabled(), true);
    assert.doesNotMatch(await catalog.innerText(), /native-private|access_token|package bytes/);
    await noOverflow(page, catalog);
    await catalog.getByRole("combobox", { name: text.organization, exact: true }).click();
    await page.getByRole("option", { name: "Second organization", exact: true }).click();
    await catalog.getByTestId("service-release").getByText(text.notInstalled, { exact: true }).waitFor();
    assert.equal(await catalog.getByRole("button", { name: text.install, exact: true }).isDisabled(), true);
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});

for (const targetMode of ["editor_host", "client_runtime"]) test(`restored unknown ${targetMode} installation keeps one operation through check, retry and acknowledge`, { skip: !enabled }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const { page, catalog, text } = await fixture(browser, { restored: true, targetMode });
    const operation = catalog.getByTestId(`account-operation-${priorOperation}`);
    await operation.getByText(text.unknown, { exact: true }).waitFor();
    assert.equal(await catalog.getByRole("button", { name: text.install, exact: true }).isDisabled(), true);
    await operation.getByRole("button", { name: text.check, exact: true }).click();
    await operation.getByRole("button", { name: text.retry, exact: true }).click();
    await catalog.getByTestId("service-release").getByText(text.installed, { exact: true }).waitFor();
    await operation.getByRole("button", { name: text.dismiss, exact: true }).click();
    await operation.waitFor({ state: "detached" });
    const requests = await page.evaluate(() => window.__CATALOG_REQUESTS__.filter(item => ["package-install", "reconcile", "retry", "acknowledge"].includes(item.action)));
    assert.deepEqual(requests.map(item => item.action), ["reconcile", "retry", "acknowledge"]);
    assert.ok(requests.every(item => item.operationId === priorOperation));
    assert.ok(await page.evaluate(target => window.__CATALOG_REQUESTS__.filter(item => item.action === "package-inventory").at(-1)?.targetMode === target, targetMode));
  } finally { await browser.close(); }
});

test("changing installation target reloads its independent inventory before another install", { skip: !enabled }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const { page, catalog, text, errors } = await fixture(browser, { alreadyLicensed: true });
    await catalog.getByRole("button", { name: text.install, exact: true }).click();
    await catalog.getByTestId("service-release").getByText(text.installed, { exact: true }).waitFor();
    await catalog.getByRole("combobox", { name: text.installTarget, exact: true }).click();
    await page.getByRole("option", { name: text.clientRuntimeTarget, exact: true }).click();
    await catalog.getByTestId("service-release").getByText(text.notInstalled, { exact: true }).waitFor();
    await catalog.getByRole("button", { name: text.install, exact: true }).click();
    await catalog.getByTestId("service-release").getByText(text.installed, { exact: true }).waitFor();
    const installs = await page.evaluate(() => window.__CATALOG_REQUESTS__.filter(item => item.action === "package-install"));
    assert.deepEqual(installs.map(request => [request.schemaVersion, request.targetMode, request.expectedInventoryRevision]), [[2, "editor_host", "0"], [2, "client_runtime", "0"]]);
    assert.notEqual(installs[0].operationId, installs[1].operationId);
    await catalog.getByRole("combobox", { name: text.installTarget, exact: true }).click();
    await page.getByRole("option", { name: text.editorHostTarget, exact: true }).click();
    await catalog.getByTestId("service-release").getByText(text.installed, { exact: true }).waitFor();
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});

test("inventory failure after committed install cannot show Installed and can be refreshed", { skip: !enabled }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const { page, catalog, text } = await fixture(browser, { alreadyLicensed: true });
    await page.evaluate(() => { window.__CATALOG_INVENTORY_ERROR__ = true; });
    await catalog.getByRole("button", { name: text.install, exact: true }).click();
    await catalog.getByTestId("service-release").getByRole("button", { name: text.retry, exact: true }).waitFor();
    assert.equal(await catalog.getByTestId("service-release").getByText(text.installed, { exact: true }).count(), 0);
    await page.evaluate(() => { window.__CATALOG_INVENTORY_ERROR__ = false; });
    await catalog.getByTestId("service-release").getByRole("button", { name: text.retry, exact: true }).click();
    await catalog.getByTestId("service-release").getByText(text.installed, { exact: true }).waitFor();
  } finally { await browser.close(); }
});

test("asset catalog exposes license review without an unsupported installation command", { skip: !enabled }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const { catalog, text } = await fixture(browser, { mode: "assets" });
    assert.equal(await catalog.getByRole("button", { name: text.install, exact: true }).count(), 0);
    assert.equal(await catalog.getByRole("button", { name: text.review, exact: true }).isEnabled(), true);
  } finally { await browser.close(); }
});

test("committed reconciliation envelope refreshes inventory without retrying installation", { skip: !enabled }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const { page, catalog, text } = await fixture(browser, { restored: true });
    await page.evaluate(() => { window.__CATALOG_RECONCILE__ = "committed"; });
    await catalog.getByTestId(`account-operation-${priorOperation}`).getByRole("button", { name: text.check, exact: true }).click();
    await catalog.getByTestId("service-release").getByText(text.installed, { exact: true }).waitFor();
    assert.equal(await page.evaluate(() => window.__CATALOG_REQUESTS__.some(item => item.action === "retry" || item.action === "package-install")), false);
  } finally { await browser.close(); }
});

for (const role of ["member", "viewer"]) test(`${role} can install licensed packages but cannot accept a license`, { skip: !enabled }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const { page, catalog, text } = await fixture(browser, { role, alreadyLicensed: true });
    await catalog.getByRole("button", { name: text.review, exact: true }).click();
    assert.equal(await page.getByRole("dialog").getByRole("button", { name: text.accept, exact: true }).isDisabled(), true);
    await page.getByRole("dialog").getByRole("button", { name: text.cancel, exact: true }).click();
    assert.equal(await catalog.getByRole("button", { name: text.install, exact: true }).isEnabled(), true);
    await catalog.getByRole("combobox", { name: text.organization, exact: true }).click();
    await page.getByRole("option", { name: "Second organization", exact: true }).click();
    await catalog.getByTestId("service-release").getByText("License required", { exact: true }).waitFor();
    await catalog.getByRole("button", { name: text.review, exact: true }).click();
    assert.equal(await page.getByRole("dialog").getByRole("button", { name: text.accept, exact: true }).isDisabled(), true);
    await page.getByRole("dialog").getByRole("button", { name: text.cancel, exact: true }).click();
    assert.equal(await catalog.getByRole("button", { name: text.install, exact: true }).isDisabled(), true);
  } finally { await browser.close(); }
});

test("catalog pagination and search do not use local workspace entries", { skip: !enabled }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const { page, catalog, text } = await fixture(browser);
    await catalog.getByRole("button", { name: text.more, exact: true }).click();
    await catalog.getByRole("button", { name: /Material editing tools/ }).waitFor();
    assert.equal(await page.evaluate(() => window.__CATALOG_REQUESTS__.find(item => item.action === "catalog" && item.after)?.after), pkgA);
    await catalog.getByRole("textbox", { name: text.search, exact: true }).fill("Material");
    await catalog.getByRole("button", { name: text.searchAction, exact: true }).click();
    await catalog.getByTestId("service-release").getByRole("heading", { name: "Material editing tools", exact: true }).waitFor();
    assert.equal(await catalog.getByText("Terrain and landscape authoring tools", { exact: true }).count(), 0);
    const query = await page.evaluate(() => window.__CATALOG_REQUESTS__.filter(item => item.action === "catalog").at(-1));
    assert.equal(query.query, "Material");
    assert.equal(query.after, undefined);
    await page.getByRole("tab", { name: text.local, exact: true }).click();
    assert.ok(await page.getByText("Local workspace extension", { exact: true }).count() > 0);
  } finally { await browser.close(); }
});

test("catalog search draft resets when the account authority signs out and back in", { skip: !enabled }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const { catalog, text } = await fixture(browser);
    const search = catalog.getByRole("textbox", { name: text.search, exact: true });
    await search.fill("Material");
    await catalog.getByRole("button", { name: text.searchAction, exact: true }).click();
    await catalog.getByTestId("service-release").getByRole("heading", { name: "Material editing tools", exact: true }).waitFor();

    await catalog.getByRole("button", { name: text.signOut, exact: true }).click();
    await catalog.getByText(text.signedOut, { exact: true }).waitFor();
    await catalog.getByRole("button", { name: text.signIn, exact: true }).click();
    await catalog.getByRole("combobox", { name: text.organization, exact: true }).waitFor();
    await catalog.getByRole("textbox", { name: text.search, exact: true }).waitFor();
    await catalog.getByRole("textbox", { name: text.search, exact: true }).evaluate(element => {
      if (element.value !== "") throw new Error(`catalog query draft retained: ${element.value}`);
    });
  } finally { await browser.close(); }
});

for (const width of [360, 768, 1280, 1920]) for (const language of ["English", "Chinese"]) test(`service catalog and license dialog fit ${width}px in ${language}`, { skip: !enabled }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const { page, catalog, text, errors } = await fixture(browser, { width, language });
    await noOverflow(page, catalog);
    if (screenshotDir) { await mkdir(screenshotDir, { recursive: true }); await page.screenshot({ path: path.join(screenshotDir, `catalog-${width}-${language}.png`), fullPage: true }); }
    await catalog.getByRole("button", { name: text.review, exact: true }).click();
    await noOverflow(page, page.getByRole("dialog"));
    if (screenshotDir) await page.screenshot({ path: path.join(screenshotDir, `catalog-license-${width}-${language}.png`), fullPage: true });
    await page.getByRole("dialog").getByRole("button", { name: text.cancel, exact: true }).click();
    await catalog.getByRole("button", { name: text.signOut, exact: true }).click();
    await catalog.getByText(text.signedOut, { exact: true }).waitFor();
    assert.equal(await catalog.getByTestId("service-release").count(), 0);
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});
