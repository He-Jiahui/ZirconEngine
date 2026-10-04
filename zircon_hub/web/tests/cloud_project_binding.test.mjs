import assert from "node:assert/strict";
import test from "node:test";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";
import { accountAdminShell } from "./account_admin_shell.mjs";

const server = await createServer({ configFile: false, root: fileURLToPath(new URL("../", import.meta.url)), optimizeDeps: { noDiscovery: true, include: [] }, server: { middlewareMode: true }, appType: "custom" });
const { AccountController } = await server.ssrLoadModule("/src/account/controller.ts");
const { parseCloudBinding } = await server.ssrLoadModule("/src/account/protocol.ts");
await server.close();

const organization = "00000000-0000-4000-8000-000000000001";
const otherOrganization = "00000000-0000-4000-8000-000000000002";
const project = "00000000-0000-4000-8000-000000000003";
const localProjectGuid = "00000000-0000-4000-8000-000000000004";
const binding = { organizationId: organization, projectId: project, localProjectGuid };
const account = (data = null, error = null) => ({
  backendEpoch: "binding-test", account: { configured: true, status: "signed-in", issuer: "https://issuer.test", subject: "alice", displayName: null, generation: "7", error: null },
  data, error, operations: [], operationsRevision: "0", operationsError: null,
});

test("binding protocol requires exact UUID fields and preserves an unbound local project", () => {
  assert.equal(parseCloudBinding(null), null);
  assert.deepEqual(parseCloudBinding(binding), binding);
  assert.throws(() => parseCloudBinding({ ...binding, projectId: "wrong" }), /account_protocol_invalid/);
  assert.throws(() => parseCloudBinding({ ...binding, path: "E:/Projects/Game" }), /account_protocol_invalid/);
});

test("selected project attaches to an authorized remote project with no head and reloads after restart", async () => {
  const requests = [];
  let persisted = null;
  const transport = {
    load: async () => account(), operationId: () => localProjectGuid,
    dispatch: async request => {
      requests.push(request);
      if (request.action === "organizations") return account({ items: [{ id: organization, name: "Team", policyRevision: "1" }], nextCursor: null });
      if (request.action === "invitations" || request.action === "members") return account({ items: [], nextCursor: null });
      if (request.action === "projects") return account({ items: [{ id: project, name: "Remote" }], nextCursor: null });
      if (request.action === "cloud-binding") return account(persisted);
      if (request.action === "attach-cloud-project") { persisted = binding; return account(binding); }
      throw new Error(`unexpected action ${request.action}`);
    },
  };
  const first = new AccountController("binding-test", transport);
  await first.start();
  await first.selectOrganization(organization);
  await first.loadCloudBinding();
  assert.equal(first.getSnapshot().cloudBinding, null);
  assert.equal(await first.attachCloudProject(project, "E:/Projects/Local"), true);
  assert.deepEqual(first.getSnapshot().cloudBinding, binding);
  assert.deepEqual(requests.at(-1), { action: "attach-cloud-project", backendEpoch: "binding-test", generation: "7", organization, project, selectedProjectId: "E:/Projects/Local" });
  const reopened = new AccountController("binding-test", transport);
  await reopened.start();
  await reopened.loadCloudBinding();
  assert.deepEqual(reopened.getSnapshot().cloudBinding, binding);
  assert.equal(requests.some(request => request.action === "cloud-commit"), false);
});

test("wrong organization and a changed local selection cannot create a visible binding", async () => {
  const requests = [];
  let localSelection = "first";
  const controller = new AccountController("binding-test", {
    load: async () => account(), operationId: () => localProjectGuid,
    dispatch: async request => {
      requests.push(request);
      if (request.action === "organizations") return account({ items: [{ id: organization, name: "First", policyRevision: "1" }, { id: otherOrganization, name: "Other", policyRevision: "1" }], nextCursor: null });
      if (request.action === "invitations" || request.action === "members") return account({ items: [], nextCursor: null });
      if (request.action === "projects") return account({ items: [{ id: project, name: "Remote" }], nextCursor: null });
      if (request.action === "cloud-binding") return account(localSelection === "first" ? binding : null);
      if (request.action === "attach-cloud-project") return account(null, "account_service_operation_failed");
      throw new Error(`unexpected action ${request.action}`);
    },
  });
  await controller.start();
  await controller.selectOrganization(otherOrganization);
  await controller.loadCloudBinding();
  assert.deepEqual(controller.getSnapshot().cloudBinding, binding);
  assert.equal(await controller.attachCloudProject(project, "E:/Projects/Local"), false);
  assert.deepEqual(controller.getSnapshot().cloudBinding, binding);
  localSelection = "second";
  await controller.loadCloudBinding();
  assert.equal(controller.getSnapshot().cloudBinding, null);
  assert.equal(requests.some(request => request.action === "cloud-commit"), false);
});

test("switching organizations during attach releases the binding UI and ignores the stale response", async () => {
  let finishFirstAttach;
  const requests = [];
  const controller = new AccountController("binding-test", {
    load: async () => account(), operationId: () => localProjectGuid,
    dispatch: async request => {
      requests.push(request);
      if (request.action === "organizations") return account({ items: [
        { id: organization, name: "First", policyRevision: "1" },
        { id: otherOrganization, name: "Second", policyRevision: "1" },
      ], nextCursor: null });
      if (request.action === "invitations" || request.action === "members") return account({ items: [], nextCursor: null });
      if (request.action === "projects") return account({ items: [{ id: project, name: "Remote" }], nextCursor: null });
      if (request.action === "cloud-binding") return account(binding);
      if (request.action === "attach-cloud-project" && request.organization === organization) return new Promise(resolve => { finishFirstAttach = resolve; });
      if (request.action === "attach-cloud-project" && request.organization === otherOrganization) return account({ ...binding, organizationId: otherOrganization });
      throw new Error(`unexpected action ${request.action}`);
    },
  });
  await controller.start();
  await controller.selectOrganization(organization);
  const pending = controller.attachCloudProject(project, "E:/Projects/Local");
  assert.equal(controller.getSnapshot().cloudBindingLoading, true);
  await controller.selectOrganization(otherOrganization);
  assert.equal(controller.getSnapshot().cloudBindingLoading, true);
  assert.equal(await controller.attachCloudProject(project, "E:/Projects/Local"), false);
  assert.equal(requests.filter(request => request.action === "attach-cloud-project").length, 1);
  finishFirstAttach(account(binding));
  assert.equal(await pending, false);
  assert.equal(controller.getSnapshot().selectedOrganization, otherOrganization);
  assert.equal(controller.getSnapshot().cloudBindingLoading, false);
  assert.deepEqual(controller.getSnapshot().cloudBinding, binding);
  assert.equal(await controller.attachCloudProject(project, "E:/Projects/Local"), true);
  assert.equal(controller.getSnapshot().cloudBinding?.organizationId, otherOrganization);
  assert.equal(requests.some(request => request.action === "cloud-commit"), false);
});

test("refreshing organizations during attach clears the native result gate after it settles", async () => {
  let finishAttach;
  const requests = [];
  const controller = new AccountController("binding-test", {
    load: async () => account(), operationId: () => localProjectGuid,
    dispatch: async request => {
      requests.push(request);
      if (request.action === "organizations") return account({ items: [{ id: organization, name: "Team", policyRevision: "1" }], nextCursor: null });
      if (request.action === "invitations" || request.action === "members") return account({ items: [], nextCursor: null });
      if (request.action === "projects") return account({ items: [{ id: project, name: "Remote" }], nextCursor: null });
      if (request.action === "cloud-binding") return account(binding);
      if (request.action === "attach-cloud-project") return new Promise(resolve => { finishAttach = resolve; });
      throw new Error(`unexpected action ${request.action}`);
    },
  });
  await controller.start();
  await controller.selectOrganization(organization);
  const pending = controller.attachCloudProject(project, "E:/Projects/Local");
  await controller.load("organizations");
  assert.equal(controller.getSnapshot().selectedOrganization, null);
  assert.equal(controller.getSnapshot().cloudBindingLoading, true);
  finishAttach(account(binding));
  assert.equal(await pending, false);
  assert.equal(controller.getSnapshot().cloudBindingLoading, false);
  assert.deepEqual(controller.getSnapshot().cloudBinding, binding);
  assert.equal(requests.filter(request => request.action === "cloud-binding").length, 1);
});

test("changing the selected local project during attach hides the old link until native refresh", async () => {
  let finishAttach;
  let localSelection = "first";
  const controller = new AccountController("binding-test", {
    load: async () => account(), operationId: () => localProjectGuid,
    dispatch: async request => {
      if (request.action === "organizations") return account({ items: [{ id: organization, name: "Team", policyRevision: "1" }], nextCursor: null });
      if (request.action === "invitations" || request.action === "members") return account({ items: [], nextCursor: null });
      if (request.action === "projects") return account({ items: [{ id: project, name: "Remote" }], nextCursor: null });
      if (request.action === "cloud-binding") return account(localSelection === "first" ? binding : null);
      if (request.action === "attach-cloud-project") return new Promise(resolve => { finishAttach = resolve; });
      throw new Error(`unexpected action ${request.action}`);
    },
  });
  await controller.start();
  await controller.selectOrganization(organization);
  await controller.loadCloudBinding();
  assert.deepEqual(controller.getSnapshot().cloudBinding, binding);
  const pending = controller.attachCloudProject(project, "E:/Projects/Local");
  localSelection = "second";
  await controller.loadCloudBinding();
  assert.equal(controller.getSnapshot().cloudBinding, null);
  assert.equal(controller.getSnapshot().cloudBindingLoading, true);
  finishAttach(account(null, "hub_cloud_binding_project_changed"));
  assert.equal(await pending, false);
  assert.equal(controller.getSnapshot().cloudBinding, null);
  assert.equal(controller.getSnapshot().cloudBindingLoading, false);
  assert.equal(controller.getSnapshot().cloudBindingError, null);
});

const require = createRequire(import.meta.url);
const playwrightRoot = process.env.ZIRCON_PLAYWRIGHT_NODE_MODULES;
const chromium = playwrightRoot ? require(path.join(playwrightRoot, "playwright")).chromium : undefined;
const executablePath = process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH;

test("browser links the selected local project to an existing remote project and reopens read only", { skip: !chromium || !executablePath }, async () => {
  const vite = await createServer({ configFile: false, root: fileURLToPath(new URL("../", import.meta.url)), server: { host: "127.0.0.1", port: 0 } });
  await vite.listen();
  const browser = await chromium.launch({ headless: true, executablePath });
  try {
    const context = await browser.newContext({ viewport: { width: 900, height: 850 } });
    const shellText = await accountAdminShell("English");
    await context.addInitScript(({ shellText, organization, project, binding }) => {
      window.__BINDING_REQUESTS__ = [];
      const snapshot = (data = null, error = null) => ({
        backendEpoch: "binding-browser",
        account: { configured: true, status: "signed-in", issuer: "https://issuer.test", subject: "alice", displayName: "Alice", generation: "7", error: null },
        data, error, operations: [], operationsRevision: "0", operationsError: null,
      });
      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
      window.__TAURI_INTERNALS__ = {
        transformCallback() { return 1; }, unregisterCallback() {},
        async invoke(command, args) {
          if (command === "plugin:event|listen") return 1;
          if (command === "hub_state") {
            const { fallbackShellState } = await import("/src/data/hubData.ts");
            return { ...fallbackShellState, backendEpoch: "binding-browser", stateRevision: "1", activePage: "team", selectedProjectId: "E:/Projects/Local", pageTitle: "Team", pageSubtitle: "", ui: shellText.ui, comingSoon: [], settings: { ...fallbackShellState.settings, language: "English" } };
          }
          if (command === "account_state") return snapshot();
          if (command !== "account_action") return undefined;
          const request = args.request;
          window.__BINDING_REQUESTS__.push(request);
          if (request.action === "organizations") return snapshot({ items: [{ id: organization, name: "Team", policyRevision: "1" }], nextCursor: null });
          if (request.action === "invitations" || request.action === "members") return snapshot({ items: [], nextCursor: null });
          if (request.action === "projects") return snapshot({ items: [{ id: project, name: "Remote" }], nextCursor: null });
          if (request.action === "cloud-binding") return snapshot(localStorage.getItem("linked") === "yes" ? binding : null);
          if (request.action === "attach-cloud-project") {
            if (request.selectedProjectId !== "E:/Projects/Local" || request.organization !== organization || request.project !== project) throw new Error("wrong attach scope");
            localStorage.setItem("linked", "yes");
            return snapshot(binding);
          }
          throw new Error(`unexpected action ${request.action}`);
        },
      };
    }, { shellText, organization, project, binding });
    const page = await context.newPage();
    const errors = [];
    page.on("pageerror", error => errors.push(error.message));
    await page.goto(vite.resolvedUrls.local[0]);
    await page.getByTestId("account-organizations").getByText("Team").click();
    await page.getByRole("tab", { name: "Projects" }).click();
    await page.getByText("Remote").waitFor();
    await page.getByRole("button", { name: "Link selected local project" }).click();
    await page.getByText("Linked to selected local project").waitFor();
    await page.reload();
    await page.getByTestId("account-organizations").getByText("Team").click();
    await page.getByRole("tab", { name: "Projects" }).click();
    await page.getByText("Linked to selected local project").waitFor();
    assert.deepEqual(await page.evaluate(() => window.__BINDING_REQUESTS__.filter(request => request.action === "cloud-commit")), []);
    assert.deepEqual(errors, []);
    await context.close();
  } finally {
    await browser.close();
    await vite.close();
  }
});
