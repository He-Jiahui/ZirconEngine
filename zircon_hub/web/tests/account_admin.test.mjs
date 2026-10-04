import assert from "node:assert/strict";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import { createServer } from "vite";
import { accountAdminShell } from "./account_admin_shell.mjs";

const server = await createServer({ configFile: false, root: fileURLToPath(new URL("../", import.meta.url)), optimizeDeps: { noDiscovery: true, include: [] }, server: { middlewareMode: true }, appType: "custom" });
const { AccountController } = await server.ssrLoadModule("/src/account/controller.ts");
const { parsePage } = await server.ssrLoadModule("/src/account/protocol.ts");
await server.close();

const orgA = "00000000-0000-4000-8000-000000000001";
const orgB = "00000000-0000-4000-8000-000000000002";
const inviteA = "00000000-0000-4000-8000-000000000101";
const inviteB = "00000000-0000-4000-8000-000000000102";
const issuer = "https://issuer.test/realms/zircon";
const owner = { issuer, subject: "owner", role: "owner", active: true };
const member = { issuer, subject: "member", role: "member", active: true };
const page = (items, nextCursor = null) => ({ items, nextCursor });
const organization = (id, policyRevision = "1") => ({ id, policyRevision, name: id === orgA ? "First team" : "Second team" });
const signed = (data = null, error = null, generation = "7") => ({
  backendEpoch: "admin-ui", account: { configured: true, status: "signed-in", issuer, subject: "owner", displayName: "Owner", generation, error: null },
  data, error, operations: [], operationsRevision: "0", operationsError: null,
});
const summary = (request, status) => ({ operationId: request.operationId, status, action: "mutate", organizationId: request.organization, error: status === "committed" ? null : status === "unknown" ? "account_service_outcome_unknown" : "account_service_operation_failed" });
function deferred() { let resolve; const promise = new Promise(done => { resolve = done; }); return { promise, resolve }; }

function fixture(handle) {
  const requests = [];
  let ids = 0;
  const client = new AccountController("admin-ui", {
    load: async () => signed(),
    operationId: () => `00000000-0000-4000-8000-${String(++ids).padStart(12, "0")}`,
    dispatch: async request => {
      requests.push(request);
      const custom = await handle?.(request);
      if (custom !== undefined) return custom;
      if (request.action === "organizations") return signed(page([organization(orgA), organization(orgB)]));
      if (request.action === "members") return signed(page([owner, member]));
      return signed(page([]));
    },
  });
  return { client, requests, ids: () => ids };
}

test("typed member, ownership and invitation changes each dispatch one exact native operation", async () => {
  for (const mutation of [
    { action: "set-member", issuer, subject: "member", role: "admin", active: false },
    { action: "transfer-ownership", issuer, subject: "member" },
    { action: "invite", issuer, subject: "invitee", role: "viewer", expires_at: 2000000000 },
    { action: "revoke-invite", invitation_id: inviteA },
  ]) {
    const { client, requests, ids } = fixture(request => request.action === "mutate"
      ? { ...signed(), operations: [summary(request, "committed")], operationsRevision: "2" } : undefined);
    await client.start();
    await client.selectOrganization(orgA);
    assert.equal(await client.mutate(orgA, "1", mutation), true);
    const writes = requests.filter(request => request.action === "mutate");
    assert.equal(writes.length, 1);
    assert.equal(ids(), 1);
    assert.deepEqual(writes[0], { action: "mutate", backendEpoch: "admin-ui", generation: "7", organization: orgA, expectedPolicyRevision: "1", operationId: "00000000-0000-4000-8000-000000000001", mutation });
    assert.equal(client.getSnapshot().selectedOrganization, orgA);
  }
});

test("a rejected organization change requires refreshed policy and a new user-confirmed operation", async () => {
  let policy = "1";
  let records = [];
  let revision = 0;
  const response = (data = null, error = null) => ({ ...signed(data, error), operations: records, operationsRevision: String(revision) });
  const { client, requests, ids } = fixture(request => {
    if (request.action === "organizations") return response(request.after ? page([organization(orgB, policy)]) : page([organization(orgA)], orgA));
    if (request.action === "members") return response(page([owner, member]));
    if (request.action === "mutate") {
      const status = request.expectedPolicyRevision === policy ? "committed" : "failed";
      records = [...records, summary(request, status)]; revision++;
      return response(null, status === "failed" ? "account_service_operation_failed" : null);
    }
    return response(page([]));
  });
  await client.start();
  await client.load("organizations", true);
  await client.selectOrganization(orgB);
  policy = "2"; // The service reports policy conflicts through the sanitized failure contract.
  const change = { action: "set-member", issuer, subject: "member", role: "admin", active: true };
  assert.equal(await client.mutate(orgB, "1", change), false);
  assert.deepEqual(client.getSnapshot().organizationsNeedingRefresh, [orgB]);
  assert.equal(await client.retryMutation(records[0].operationId), false);
  assert.equal(await client.mutate(orgB, "2", change), false);
  assert.equal(ids(), 1);
  const refreshed = await client.refreshOrganization();
  assert.equal(refreshed.policyRevision, "2");
  assert.equal(client.getSnapshot().selectedOrganization, orgB);
  assert.deepEqual(client.getSnapshot().organizationsNeedingRefresh, []);
  assert.equal(requests.filter(request => request.action === "mutate").length, 1);
  assert.equal(await client.mutate(orgB, refreshed.policyRevision, change), true);
  assert.equal(ids(), 2);
  const writes = requests.filter(request => request.action === "mutate");
  assert.notEqual(writes[0].operationId, writes[1].operationId);
  assert.equal(writes[0].expectedPolicyRevision, "1");
  assert.equal(writes[1].expectedPolicyRevision, "2");
});

test("an unknown admin mutation blocks new changes and recovery sends only the original operation id", async () => {
  let record;
  let revision = "0";
  const response = (data = null, error = null) => ({ ...signed(data, error), operations: record ? [record] : [], operationsRevision: revision });
  const { client, requests, ids } = fixture(request => {
    if (request.action === "mutate") { record = summary(request, "unknown"); revision = "1"; return response(null, "account_service_outcome_unknown"); }
    if (request.action === "retry") { record = { ...record, status: "committed", error: null }; revision = "2"; return response(); }
    if (request.action === "organizations") return response(page([organization(orgA)]));
    if (request.action === "members") return response(page([owner, member]));
    return response(page([]));
  });
  await client.start(); await client.selectOrganization(orgA);
  assert.equal(await client.mutate(orgA, "1", { action: "transfer-ownership", issuer, subject: "member" }), false);
  assert.equal(await client.mutate(orgA, "1", { action: "invite", issuer, subject: "other", role: "member", expires_at: 2000000000 }), false);
  assert.equal(ids(), 1);
  assert.equal(await client.retryMutation(record.operationId), true);
  assert.deepEqual(requests.find(request => request.action === "retry"), { action: "retry", backendEpoch: "admin-ui", generation: "7", operationId: record.operationId });
});

test("late organization refresh cannot restore an older selection or its member authority", async () => {
  const pending = deferred();
  let refreshing = false;
  const { client } = fixture(request => refreshing && request.action === "organizations" ? pending.promise : undefined);
  await client.start(); await client.selectOrganization(orgA);
  refreshing = true;
  const refresh = client.refreshOrganization();
  await client.selectOrganization(orgB);
  pending.resolve(signed(page([organization(orgA, "2"), organization(orgB, "2")])));
  assert.equal(await refresh, null);
  assert.equal(client.getSnapshot().selectedOrganization, orgB);
  assert.deepEqual(client.getSnapshot().members.items, [owner, member]);
});

test("selection discovers the actor beyond the initial member page", async () => {
  const { client, requests } = fixture(request => request.action === "members"
    ? signed(request.after ? page([owner]) : page([member], "12")) : undefined);
  await client.start(); await client.selectOrganization(orgA);
  assert.equal(client.getSnapshot().members.items.find(item => item.subject === "owner").role, "owner");
  assert.equal(requests.filter(request => request.action === "members")[1].after, "12");
});

const issuedInvitation = (id = inviteA, organizationId = orgA, status = "pending") => ({ id, organizationId, policyRevision: "1", targetIssuer: issuer, targetSubject: "invitee", role: "member", expiresAt: 2000000000, status });

test("issued invitation DTOs preserve qualified recipient identity and reject malformed state", () => {
  for (const status of ["pending", "accepted", "revoked", "expired"]) assert.deepEqual(parsePage("issued-invitations", page([issuedInvitation(inviteA, orgA, status)])).items, [issuedInvitation(inviteA, orgA, status)]);
  for (const row of [{ ...issuedInvitation(), targetIssuer: null }, { ...issuedInvitation(), targetSubject: "" }, { ...issuedInvitation(), policyRevision: "01" }, { ...issuedInvitation(), status: "invented" }, { ...issuedInvitation(), expiresAt: -1 }]) {
    assert.throws(() => parsePage("issued-invitations", page([row])), /account_protocol_invalid/);
  }
});

test("issued invitations wait for the qualified actor and never query for member or viewer", async () => {
  for (const role of ["owner", "admin", "member", "viewer"]) {
    const actorPage = deferred();
    const { client, requests } = fixture(request => request.action === "members" ? request.after ? actorPage.promise : signed(page([member], "12")) : undefined);
    await client.start();
    const selecting = client.selectOrganization(orgA);
    await new Promise(resolve => setImmediate(resolve));
    await client.load("issued-invitations");
    assert.equal(requests.some(request => request.action === "issued-invitations"), false);
    actorPage.resolve(signed(page([{ ...owner, role }])));
    await selecting;
    assert.equal(requests.filter(request => request.action === "issued-invitations").length, ["owner", "admin"].includes(role) ? 1 : 0);
  }
});

test("issued invitation pages are organization-fenced and reject a different organization DTO", async () => {
  const stale = deferred();
  let hold = false;
  const { client } = fixture(request => request.action === "issued-invitations" ? request.organization === orgA
    ? hold ? stale.promise : signed(request.after ? page([issuedInvitation(inviteB)]) : page([issuedInvitation()], inviteA))
    : signed(page([issuedInvitation(inviteA, orgA)])) : undefined);
  await client.start(); await client.selectOrganization(orgA); await client.load("issued-invitations", true);
  assert.deepEqual(client.getSnapshot().issuedInvitations.items.map(item => item.id), [inviteA, inviteB]);
  hold = true;
  const loading = client.load("issued-invitations");
  await client.selectOrganization(orgB);
  stale.resolve(signed(page([issuedInvitation()])));
  await loading;
  assert.equal(client.getSnapshot().selectedOrganization, orgB);
  assert.deepEqual(client.getSnapshot().issuedInvitations.items, []);
  assert.equal(client.getSnapshot().issuedInvitations.error, "account_protocol_invalid");
});

test("revoke refresh restores its target from a later issued-invitation page without dispatching a mutation", async () => {
  const { client, requests } = fixture(request => request.action === "issued-invitations" ? signed(request.after
    ? page([{ ...issuedInvitation(inviteB), policyRevision: "3" }]) : page([issuedInvitation()], inviteA)) : undefined);
  await client.start(); await client.selectOrganization(orgA);
  assert.equal(client.getSnapshot().issuedInvitations.items.some(item => item.id === inviteB), false);
  await client.refreshOrganization(undefined, inviteB);
  assert.equal(client.getSnapshot().issuedInvitations.items.find(item => item.id === inviteB).policyRevision, "3");
  assert.equal(requests.some(request => request.action === "mutate"), false);
});

test("a recipient can refresh and reconfirm a rejected acceptance without organization membership", async () => {
  let operations = [];
  let revision = "0";
  const response = (data = null, error = null) => ({ ...signed(data, error), operations, operationsRevision: revision });
  const { client, requests } = fixture(request => {
    if (request.action === "mutate") { operations = [summary(request, "failed")]; revision = "1"; return response(null, "account_service_operation_failed"); }
    if (request.action === "invitations") return response(page([{ id: inviteA, organizationId: orgA, organizationName: "First team", policyRevision: revision === "0" ? "1" : "2", role: "member", expiresAt: 2000000000, status: "pending" }]));
    return response(page([]));
  });
  await client.start();
  assert.equal(await client.mutate(orgA, "1", { action: "accept-invite", invitation_id: inviteA }), false);
  assert.deepEqual(client.getSnapshot().organizationsNeedingRefresh, [orgA]);
  await client.load("invitations");
  assert.deepEqual(client.getSnapshot().organizationsNeedingRefresh, []);
  assert.equal(client.getSnapshot().invitations.items[0].policyRevision, "2");
  assert.equal(requests.filter(request => request.action === "mutate").length, 1);
});

const require = createRequire(import.meta.url);
const playwrightRoot = process.env.ZIRCON_PLAYWRIGHT_NODE_MODULES;
const chromium = playwrightRoot ? require(path.join(playwrightRoot, "playwright")).chromium : undefined;
const launchOptions = process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH ? { executablePath: process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH } : {};
const browserUrl = process.env.ZIRCON_HUB_TEST_URL;
const screenshotDir = process.env.ZIRCON_HUB_SCREENSHOT_DIR;
const browserEnabled = Boolean(chromium && browserUrl);

async function browserFixture(browser, width, language, role = "owner") {
  const context = await browser.newContext({ viewport: { width, height: 900 } });
  const shellText = await accountAdminShell(language);
  await context.addInitScript(({ language, role, orgA, orgB, issuer, inviteA, inviteB, shellText }) => {
    let generation = 7;
    let signedIn = true;
    let policyRevision = 1;
    let operationsRevision = 0;
    let operations = [];
    let currentRole = role;
    const issuedStates = {};
    let changedMember = { issuer, subject: "member", role: "member", active: true };
    let shell;
    window.__ADMIN_REQUESTS__ = [];
    window.__ADMIN_OUTCOME__ = "committed";
    const account = (data = null, error = null) => ({ backendEpoch: "admin-ui", account: { configured: true, status: signedIn ? "signed-in" : "signed-out", issuer: signedIn ? issuer : null, subject: signedIn ? "owner" : null, displayName: signedIn ? "Organization Owner" : null, generation: String(generation), error: null }, data, error, operations: signedIn ? operations : [], operationsRevision: signedIn ? String(operationsRevision) : "0", operationsError: null });
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback() { return 1; }, unregisterCallback() {},
      async invoke(command, args) {
        if (command === "plugin:event|listen") return 1;
        if (command === "hub_state") {
          const { fallbackShellState } = await import("/src/data/hubData.ts");
          for (const [section, fields] of Object.entries(shellText.ui)) {
            if (Object.keys(fields).sort().join() !== Object.keys(fallbackShellState.ui[section]).sort().join()) throw new Error(`native ${section} text schema differs from the browser fixture`);
          }
          shell = {
            ...fallbackShellState, backendEpoch: "admin-ui", stateRevision: "1", activePage: "team",
            pageTitle: shellText.pageTitle, pageSubtitle: shellText.pageSubtitle,
            ui: shellText.ui, comingSoon: shellText.comingSoon,
            taskSummary: { ...fallbackShellState.taskSummary, label: shellText.ui.common.ready, detail: shellText.readyDetail },
            settings: { ...fallbackShellState.settings, language },
          };
          return shell;
        }
        if (command === "account_state") return account();
        if (command !== "account_action") return undefined;
        const request = args.request;
        window.__ADMIN_REQUESTS__.push(request);
        if (request.action === "logout") { signedIn = false; generation++; return account(); }
        if (request.action === "refresh") { generation++; return account(); }
        if (request.action === "organizations") return account(request.after ? { items: [{ id: orgB, name: "Second team", policyRevision: String(policyRevision) }], nextCursor: null } : { items: [{ id: orgA, name: "First team with a long organization name for narrow windows", policyRevision: String(policyRevision) }], nextCursor: orgA });
        if (request.action === "members") return account(request.after ? { items: [{ issuer, subject: "owner", role: currentRole, active: true }], nextCursor: null } : { items: [changedMember], nextCursor: "12" });
        if (request.action === "issued-invitations") {
          const invitation = (id, targetSubject, status = "pending") => ({ id, organizationId: request.organization, policyRevision: String(policyRevision), targetIssuer: issuer, targetSubject, role: "member", expiresAt: status === "expired" ? 1 : Math.floor(Date.now() / 1000) + 86400, status: issuedStates[id] ?? status });
          return account(request.after ? { items: [invitation(inviteB, "second-invitee"), invitation("00000000-0000-4000-8000-000000000103", "expired-invitee", "expired"), invitation("00000000-0000-4000-8000-000000000104", "accepted-invitee", "accepted"), invitation("00000000-0000-4000-8000-000000000105", "revoked-invitee", "revoked")], nextCursor: null } : { items: [invitation(inviteA, "first-invitee-with-a-long-qualified-account-name")], nextCursor: inviteA });
        }
        if (request.action === "projects" || request.action === "invitations") return account({ items: [], nextCursor: null });
        if (request.action === "mutate") {
          const status = window.__ADMIN_OUTCOME__;
          const record = { operationId: request.operationId, status, action: "mutate", organizationId: request.organization, error: status === "committed" ? null : status === "unknown" ? "account_service_outcome_unknown" : "account_service_operation_failed" };
          operations = [...operations, record]; operationsRevision++; policyRevision++;
          if (status === "committed" && request.mutation.action === "set-member") changedMember = { ...changedMember, role: request.mutation.role, active: request.mutation.active };
          if (status === "committed" && request.mutation.action === "transfer-ownership") { changedMember = { ...changedMember, role: "owner" }; currentRole = "admin"; }
          if (status === "committed" && request.mutation.action === "revoke-invite") issuedStates[request.mutation.invitation_id] = "revoked";
          return account(null, record.error);
        }
        if (request.action === "reconcile" || request.action === "retry") {
          operations = operations.map(item => item.operationId === request.operationId ? { ...item, status: "committed", error: null } : item); operationsRevision++;
          return account();
        }
        if (request.action === "acknowledge") { operations = operations.filter(item => item.operationId !== request.operationId); operationsRevision++; return account(); }
        throw new Error("unexpected fixture request");
      },
    };
  }, { language, role, orgA, orgB, issuer, inviteA, inviteB, shellText });
  const page = await context.newPage();
  await page.goto(browserUrl);
  const account = page.getByTestId("hub-account");
  await account.waitFor();
  const navigation = page.getByTestId("hub-navigation-drawer");
  for (const item of shellText.ui.shell.navItems) assert.equal(await navigation.getByRole("button", { name: item.label, exact: true }).count(), 1);
  assert.equal(await page.getByText(shellText.pageSubtitle, { exact: true }).count(), 1);
  assert.equal(await page.getByText(shellText.readyDetail, { exact: true }).count(), 1);
  if (language === "English") assert.doesNotMatch(await page.locator("body").innerText(), /\p{Script=Han}/u);
  await account.getByRole("button", { name: "First team with a long organization name for narrow windows", exact: true }).click();
  await account.getByTestId("account-member-owner").waitFor();
  return { context, page, account };
}

const labels = language => language === "Chinese" ? {
  edit: "编辑成员: member", role: "角色", admin: "管理员", viewer: "访客", active: "启用成员资格", save: "保存成员", invite: "邀请成员", subject: "账号 ID", create: "创建邀请", expiry: "有效期", threeDays: "3 天", transfer: "移交所有权", confirm: "确认移交", cancel: "取消", refresh: "刷新组织", reconfirm: "组织已刷新。请核对变更并再次确认。", more: "加载更多", signOut: "退出登录", refreshSession: "刷新会话", revoke: "撤销邀请", confirmRevoke: "确认撤销", revoked: "已撤销", tabs: "成员与项目", members: "成员", projects: "项目", projectName: "项目名称",
} : {
  edit: "Edit member: member", role: "Role", admin: "Admin", viewer: "Viewer", active: "Active membership", save: "Save member", invite: "Invite member", subject: "Account ID", create: "Create invitation", expiry: "Expires in", threeDays: "3 days", transfer: "Transfer ownership", confirm: "Confirm transfer", cancel: "Cancel", refresh: "Refresh organization", reconfirm: "Organization refreshed. Review this change and confirm again.", more: "Load more", signOut: "Sign out", refreshSession: "Refresh session", revoke: "Revoke invitation", confirmRevoke: "Confirm revocation", revoked: "Revoked", tabs: "Members and projects", members: "Members", projects: "Projects", projectName: "Project name",
};

async function choose(page, dialog, label, option) {
  await dialog.getByRole("combobox", { name: label, exact: true }).click();
  await page.getByRole("option", { name: option, exact: true }).click();
}

async function noOverflow(page, locator) {
  await locator.evaluate(async element => {
    const root = element.closest(".MuiDialog-root");
    if (root) await Promise.all(root.getAnimations({ subtree: true }).map(animation => animation.finished.catch(() => {})));
    await document.fonts.ready;
  });
  const bad = await locator.evaluate(root => [...root.querySelectorAll("button, input, p, h6, label")].filter(node => {
    const rect = node.getBoundingClientRect();
    return rect.width > 0 && (rect.left < -1 || rect.right > innerWidth + 1);
  }).map(node => node.textContent));
  assert.deepEqual(bad, []);
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
}

for (const language of ["English", "Chinese"]) {
  test(`member and project tabs expose a localized tablist in ${language}`, { skip: !browserEnabled }, async () => {
    const browser = await chromium.launch({ headless: true, ...launchOptions });
    try {
      const { account } = await browserFixture(browser, language === "Chinese" ? 360 : 1280, language);
      const text = labels(language);
      const tabs = account.getByRole("tablist", { name: text.tabs, exact: true });
      assert.equal(await tabs.count(), 1);
      const members = tabs.getByRole("tab", { name: text.members, exact: true });
      const projects = tabs.getByRole("tab", { name: text.projects, exact: true });
      assert.equal(await members.getAttribute("aria-selected"), "true");
      assert.equal(await projects.getAttribute("aria-selected"), "false");

      await projects.click();
      assert.equal(await projects.getAttribute("aria-selected"), "true");
      assert.equal(await members.getAttribute("aria-selected"), "false");
      await account.getByRole("textbox", { name: text.projectName, exact: true }).waitFor();
    } finally { await browser.close(); }
  });
}

for (const language of ["English", "Chinese"]) {
  test(`member edit, invite and explicit ownership transfer dispatch real UI actions in ${language}`, { skip: !browserEnabled }, async () => {
    const browser = await chromium.launch({ headless: true, ...launchOptions });
    try {
      const { page, account } = await browserFixture(browser, language === "Chinese" ? 360 : 1280, language);
      const text = labels(language);
      await account.getByRole("button", { name: text.edit, exact: true }).click();
      let dialog = page.getByRole("dialog");
      await choose(page, dialog, text.role, text.admin);
      await dialog.getByRole("switch", { name: text.active, exact: true }).uncheck();
      await dialog.getByRole("button", { name: text.save, exact: true }).click();
      await dialog.waitFor({ state: "detached" });
      const edit = await page.evaluate(() => window.__ADMIN_REQUESTS__.find(request => request.action === "mutate"));
      assert.deepEqual(edit.mutation, { action: "set-member", issuer, subject: "member", role: "admin", active: false });

      await account.getByRole("button", { name: text.invite, exact: true }).click();
      dialog = page.getByRole("dialog");
      await dialog.getByLabel(text.subject, { exact: true }).fill("new-member");
      await choose(page, dialog, text.role, text.viewer);
      await choose(page, dialog, text.expiry, text.threeDays);
      const before = Math.floor(Date.now() / 1000);
      await dialog.getByRole("button", { name: text.create, exact: true }).click();
      await dialog.waitFor({ state: "detached" });
      const invite = await page.evaluate(() => window.__ADMIN_REQUESTS__.filter(request => request.action === "mutate")[1]);
      assert.deepEqual({ ...invite.mutation, expires_at: 0 }, { action: "invite", issuer, subject: "new-member", role: "viewer", expires_at: 0 });
      assert.ok(invite.mutation.expires_at >= before + 3 * 86400 && invite.mutation.expires_at <= Math.floor(Date.now() / 1000) + 3 * 86400);

      await account.getByRole("button", { name: text.edit, exact: true }).click();
      dialog = page.getByRole("dialog");
      await dialog.getByRole("switch", { name: text.active, exact: true }).check();
      await dialog.getByRole("button", { name: text.save, exact: true }).click();
      await dialog.waitFor({ state: "detached" });
      await account.getByRole("button", { name: text.transfer, exact: true }).click();
      dialog = page.getByRole("dialog");
      assert.equal(await page.evaluate(() => window.__ADMIN_REQUESTS__.filter(request => request.action === "mutate").length), 3);
      await dialog.getByRole("button", { name: text.cancel, exact: true }).click();
      await dialog.waitFor({ state: "detached" });
      assert.equal(await page.evaluate(() => window.__ADMIN_REQUESTS__.filter(request => request.action === "mutate").length), 3);
      await account.getByRole("button", { name: text.transfer, exact: true }).click();
      await page.getByRole("dialog").getByRole("button", { name: text.confirm, exact: true }).click();
      await page.getByRole("dialog").waitFor({ state: "detached" });
      const writes = await page.evaluate(() => window.__ADMIN_REQUESTS__.filter(request => request.action === "mutate"));
      assert.equal(writes.length, 4);
      assert.equal(new Set(writes.map(request => request.operationId)).size, 4);
      assert.deepEqual(writes[3].mutation, { action: "transfer-ownership", issuer, subject: "member" });
      assert.equal(await account.getByRole("button", { name: text.edit, exact: true }).count(), 0);
      assert.equal(await account.getByRole("button", { name: text.invite, exact: true }).isEnabled(), true);
    } finally { await browser.close(); }
  });
}

test("admin UI refreshes a later-page organization after rejection and requires a separate confirm", { skip: !browserEnabled }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const { page, account } = await browserFixture(browser, 768, "English");
    await account.getByTestId("account-organizations").getByRole("button", { name: "Load more", exact: true }).click();
    await account.getByRole("button", { name: "Second team", exact: true }).click();
    await account.getByRole("button", { name: "Edit member: member", exact: true }).click();
    const dialog = page.getByRole("dialog");
    await choose(page, dialog, "Role", "Admin");
    await page.evaluate(() => { window.__ADMIN_OUTCOME__ = "failed"; });
    await dialog.getByRole("button", { name: "Save member", exact: true }).click();
    await dialog.getByRole("button", { name: "Refresh organization", exact: true }).click();
    await dialog.getByText("Organization refreshed. Review this change and confirm again.", { exact: true }).waitFor();
    assert.equal(await page.evaluate(() => window.__ADMIN_REQUESTS__.filter(request => request.action === "mutate").length), 1);
    await page.evaluate(() => { window.__ADMIN_OUTCOME__ = "committed"; });
    await dialog.getByRole("button", { name: "Save member", exact: true }).click();
    await dialog.waitFor({ state: "detached" });
    const writes = await page.evaluate(() => window.__ADMIN_REQUESTS__.filter(request => request.action === "mutate"));
    assert.equal(writes.length, 2);
    assert.equal(writes[0].organization, orgB);
    assert.equal(writes[1].organization, orgB);
    assert.equal(writes[0].expectedPolicyRevision, "1");
    assert.equal(writes[1].expectedPolicyRevision, "2");
    assert.notEqual(writes[0].operationId, writes[1].operationId);
  } finally { await browser.close(); }
});

test("admin dialogs discard drafts on organization and account changes and unknown outcomes block new writes", { skip: !browserEnabled }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const { page, account } = await browserFixture(browser, 768, "English");
    await account.getByTestId("account-organizations").getByRole("button", { name: "Load more", exact: true }).click();
    await account.getByRole("button", { name: "Invite member", exact: true }).click();
    await page.getByRole("dialog").getByLabel("Account ID", { exact: true }).fill("discard-me");
    await page.evaluate(() => [...document.querySelectorAll('[data-testid="hub-account"] [role="button"]')].find(button => button.textContent === "Second team").click());
    await page.getByRole("dialog").waitFor({ state: "detached" });
    await account.getByRole("button", { name: "Invite member", exact: true }).click();
    assert.equal(await page.getByRole("dialog").getByLabel("Account ID", { exact: true }).inputValue(), "");
    await page.evaluate(() => document.querySelector('button[aria-label="Refresh session"]').click());
    await page.getByRole("dialog").waitFor({ state: "detached" });
    await account.getByRole("button", { name: "First team with a long organization name for narrow windows", exact: true }).click();
    await account.getByRole("button", { name: "Invite member", exact: true }).click();
    const dialog = page.getByRole("dialog");
    assert.equal(await dialog.getByLabel("Account ID", { exact: true }).inputValue(), "");
    await dialog.getByLabel("Account ID", { exact: true }).fill("unknown-member");
    await page.evaluate(() => { window.__ADMIN_OUTCOME__ = "unknown"; });
    await dialog.getByRole("button", { name: "Create invitation", exact: true }).click();
    await dialog.waitFor({ state: "detached" });
    assert.equal(await account.getByRole("button", { name: "Invite member", exact: true }).isDisabled(), true);
    assert.equal(await account.getByRole("button", { name: "Edit member: member", exact: true }).isDisabled(), true);
    const original = await page.evaluate(() => window.__ADMIN_REQUESTS__.find(request => request.action === "mutate"));
    await account.getByTestId(`account-operation-${original.operationId}`).getByRole("button", { name: "Retry", exact: true }).click();
    const recovery = await page.evaluate(() => window.__ADMIN_REQUESTS__.find(request => request.action === "retry"));
    assert.deepEqual(recovery, { action: "retry", backendEpoch: "admin-ui", generation: "8", operationId: original.operationId });
  } finally { await browser.close(); }
});

for (const role of ["admin", "member", "viewer"]) {
  test(`admin controls match the loaded ${role} membership`, { skip: !browserEnabled }, async () => {
    const browser = await chromium.launch({ headless: true, ...launchOptions });
    try {
      const { account } = await browserFixture(browser, 768, "English", role);
      assert.equal(await account.getByRole("button", { name: "Edit member: member", exact: true }).count(), 0);
      assert.equal(await account.getByRole("button", { name: "Transfer ownership", exact: true }).count(), 0);
      assert.equal(await account.getByRole("button", { name: "Invite member", exact: true }).count(), role === "admin" ? 1 : 0);
      assert.equal(await account.getByTestId("account-issued-invitations").count(), role === "admin" ? 1 : 0);
    } finally { await browser.close(); }
  });
}

for (const language of ["English", "Chinese"]) {
  test(`issued invitation pagination and revoke confirmation use the native operation contract in ${language}`, { skip: !browserEnabled }, async () => {
    const browser = await chromium.launch({ headless: true, ...launchOptions });
    try {
      const { page, account } = await browserFixture(browser, language === "Chinese" ? 360 : 1280, language);
      const text = labels(language);
      const list = account.getByTestId("account-issued-invitations");
      await list.getByRole("button", { name: text.more, exact: true }).click();
      assert.equal(await list.getByRole("button", { name: text.revoke, exact: true }).count(), 2);
      const target = list.getByTestId(`account-issued-${inviteB}`);
      await target.getByRole("button", { name: text.revoke, exact: true }).click();
      let dialog = page.getByRole("dialog");
      await dialog.getByRole("button", { name: text.cancel, exact: true }).click();
      await dialog.waitFor({ state: "detached" });
      assert.equal(await page.evaluate(() => window.__ADMIN_REQUESTS__.some(request => request.action === "mutate")), false);
      await target.getByRole("button", { name: text.revoke, exact: true }).click();
      dialog = page.getByRole("dialog");
      if (language === "English") {
        await page.evaluate(() => { window.__ADMIN_OUTCOME__ = "failed"; });
        await dialog.getByRole("button", { name: text.confirmRevoke, exact: true }).click();
        await dialog.getByRole("button", { name: text.refresh, exact: true }).click();
        await dialog.getByText(text.reconfirm, { exact: true }).waitFor();
        assert.equal(await page.evaluate(() => window.__ADMIN_REQUESTS__.filter(request => request.action === "mutate").length), 1);
        await page.evaluate(() => { window.__ADMIN_OUTCOME__ = "committed"; });
      }
      await dialog.getByRole("button", { name: text.confirmRevoke, exact: true }).click();
      await dialog.waitFor({ state: "detached" });
      const requests = await page.evaluate(() => window.__ADMIN_REQUESTS__);
      const writes = requests.filter(request => request.action === "mutate");
      assert.equal(writes.length, language === "English" ? 2 : 1);
      assert.ok(writes.every(request => request.organization === orgA && request.generation === "7" && request.mutation.action === "revoke-invite" && request.mutation.invitation_id === inviteB));
      assert.equal(new Set(writes.map(request => request.operationId)).size, writes.length);
      assert.equal(writes.at(-1).expectedPolicyRevision, language === "English" ? "2" : "1");
      assert.ok(requests.some(request => request.action === "issued-invitations" && request.after === inviteA));
      await list.getByRole("button", { name: text.more, exact: true }).click();
      assert.equal(await target.getByRole("button", { name: text.revoke, exact: true }).count(), 0);
      await target.getByText(new RegExp(text.revoked)).waitFor();
    } finally { await browser.close(); }
  });
}

for (const width of [360, 768, 1280, 1920]) {
  for (const language of ["English", "Chinese"]) {
    test(`admin dialogs fit ${width}px in ${language}`, { skip: !browserEnabled }, async () => {
      const browser = await chromium.launch({ headless: true, ...launchOptions });
      try {
        const { page, account } = await browserFixture(browser, width, language);
        const text = labels(language);
        await noOverflow(page, account);
        for (const [name, action] of [["edit", text.edit], ["transfer", text.transfer], ["invite", text.invite], ["revoke", text.revoke]]) {
          await account.getByRole("button", { name: action, exact: true }).click();
          const dialog = page.getByRole("dialog");
          await dialog.waitFor();
          await noOverflow(page, dialog);
          if (screenshotDir) {
            await mkdir(screenshotDir, { recursive: true });
            await page.screenshot({ path: path.join(screenshotDir, `account-admin-${name}-${width}-${language}.png`), fullPage: true, animations: "disabled" });
          }
          await dialog.getByRole("button", { name: text.cancel, exact: true }).click();
          await dialog.waitFor({ state: "detached" });
        }
      } finally { await browser.close(); }
    });
  }
}
