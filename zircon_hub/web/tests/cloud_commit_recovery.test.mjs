import assert from "node:assert/strict";
import test from "node:test";
import { createRequire } from "node:module";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";
import { accountAdminShell } from "./account_admin_shell.mjs";

const validationRoot = process.env.ZIRCON_HUB_VALIDATION_ROOT ?? "E:/cargo-targets/zircon-engine/hub-web-validation";
const runRoot = path.join(validationRoot, `cloud-commit-recovery-${process.pid}`);
const ssrCacheDir = path.join(runRoot, "ssr-vite-cache");
await mkdir(ssrCacheDir, { recursive: true });
const server = await createServer({ configFile: false, cacheDir: ssrCacheDir, root: fileURLToPath(new URL("../", import.meta.url)), optimizeDeps: { noDiscovery: true, include: [] }, server: { middlewareMode: true }, appType: "custom" });
const { assertSnapshot, parseCloudCommitReconciliation } = await server.ssrLoadModule("/src/account/protocol.ts");
const { AccountController } = await server.ssrLoadModule("/src/account/controller.ts");
const { operationNoticeDetails } = await server.ssrLoadModule("/src/account/OperationNotice.tsx");
const { accountCopy } = await server.ssrLoadModule("/src/account/copy.ts");
await server.close();

const organization = "00000000-0000-4000-8000-000000000001";
const project = "00000000-0000-4000-8000-000000000002";
const operationId = "00000000-0000-4000-8000-000000000003";
const digest = "a".repeat(64);
const summary = (status, error = status === "unknown" ? "account_service_outcome_unknown" : null) => ({
  operationId, status, action: "cloud-commit", organizationId: organization, error,
});
const packageSummary = targetMode => ({
  operationId, status: "unknown", action: "package-install", organizationId: organization,
  error: "account_service_outcome_unknown", ...(targetMode ? { targetMode } : {}),
});
const account = (operations = [summary("unknown")], data = null, error = null, revision = "1") => ({
  backendEpoch: "cloud-recovery", account: { configured: true, status: "signed-in", issuer: "https://issuer.test", subject: "alice", displayName: null, generation: "7", error: null },
  data, error, operations, operationsRevision: revision, operationsError: null,
});
const committed = () => ({ status: "committed", result: { status: "committed", snapshot: {
  organizationId: organization, projectId: project, baseRevision: "0", revision: "1", manifestDigest: digest,
} } });
const conflict = () => ({ status: "conflict", result: {
  status: "conflict", organizationId: organization, projectId: project, baseRevision: "0", currentRevision: "1", manifestDigest: digest,
} });

test("restored cloud journal summary is accepted but malformed scope is rejected", () => {
  assertSnapshot(account());
  assertSnapshot(account([packageSummary("client_runtime")]));
  assertSnapshot(account([packageSummary(undefined)]));
  assert.throws(() => assertSnapshot(account([{ ...summary("unknown"), organizationId: null }])), /account_protocol_invalid/);
  assert.throws(() => assertSnapshot(account([{ ...summary("unknown"), status: "conflict" }])), /account_protocol_invalid/);
  assert.throws(() => assertSnapshot(account([{ ...packageSummary("server_runtime") }])), /account_protocol_invalid/);
  assert.throws(() => assertSnapshot(account([{ ...summary("unknown"), targetMode: "editor_host" }])), /account_protocol_invalid/);
});

test("cloud reconciliation is strict about operation identity, tenant, status, revision and receipt shape", () => {
  assert.deepEqual(parseCloudCommitReconciliation({ status: "unknown" }, summary("unknown"), operationId), { status: "unknown" });
  assert.deepEqual(parseCloudCommitReconciliation(committed(), summary("committed"), operationId), committed());
  assert.deepEqual(parseCloudCommitReconciliation(conflict(), summary("failed", "account_cloud_conflict"), operationId), conflict());
  const cases = [
    [committed(), summary("committed"), project],
    [committed(), summary("unknown"), operationId],
    [committed(), summary("failed", "account_cloud_conflict"), operationId],
    [{ ...committed(), result: { ...committed().result, snapshot: { ...committed().result.snapshot, organizationId: project } } }, summary("committed"), operationId],
    [{ ...committed(), result: { ...committed().result, snapshot: { ...committed().result.snapshot, revision: "2" } } }, summary("committed"), operationId],
    [{ ...conflict(), result: { ...conflict().result, currentRevision: "0" } }, summary("failed", "account_cloud_conflict"), operationId],
    [{ ...conflict(), result: { ...conflict().result, extra: "unexpected" } }, summary("failed", "account_cloud_conflict"), operationId],
    [{ status: "unknown", result: null }, summary("unknown"), operationId],
  ];
  for (const [receipt, operation, id] of cases) {
    assert.throws(() => parseCloudCommitReconciliation(receipt, operation, id), /account_protocol_invalid/);
  }
});

test("cloud Unknown is reconcile only; typed conflict remains terminal and does not dirty organization authority", async () => {
  const requests = [];
  let operations = [summary("unknown")];
  let revision = "1";
  const reply = (data = null, error = null) => account(operations, data, error, revision);
  const controller = new AccountController("cloud-recovery", {
    load: async () => reply(), operationId: () => project,
    dispatch: async request => {
      requests.push(request);
      if (request.action === "organizations") return reply({ items: [{ id: organization, name: "Team", policyRevision: "1" }], nextCursor: null });
      if (request.action === "invitations" || request.action === "members") return reply({ items: [], nextCursor: null });
      if (request.action === "projects") return reply({ items: [{ id: project, name: "Demo" }], nextCursor: null });
      if (request.action === "reconcile") {
        operations = [summary("failed", "account_cloud_conflict")]; revision = "2";
        return reply(conflict());
      }
      if (request.action === "acknowledge") { operations = []; revision = "3"; return reply(); }
      throw new Error(`unexpected action ${request.action}`);
    },
  });
  await controller.start();
  await controller.selectOrganization(organization);
  assert.equal(await controller.retryMutation(operationId), false);
  assert.equal(requests.some(request => request.action === "retry"), false);
  assert.equal(await controller.checkMutation(operationId), false);
  assert.deepEqual(requests.at(-1), { action: "reconcile", backendEpoch: "cloud-recovery", generation: "7", operationId });
  assert.equal(controller.getSnapshot().snapshot.operations[0].error, "account_cloud_conflict");
  assert.deepEqual(controller.getSnapshot().organizationsNeedingRefresh, []);
  assert.equal(await controller.retryMutation(operationId), false);
  assert.equal(await controller.acknowledgeOperation(operationId), true);
  assert.deepEqual(controller.getSnapshot().snapshot.operations, []);
});

test("cloud identity survives a reconcile error after its summary disappears and cannot dispatch retry", async () => {
  const requests = [];
  let operations = [summary("unknown")];
  let revision = "1";
  const reply = (data = null, error = null) => account(operations, data, error, revision);
  const controller = new AccountController("cloud-recovery", {
    load: async () => reply(), operationId: () => project,
    dispatch: async request => {
      requests.push(request);
      if (request.action === "organizations") return reply({ items: [{ id: organization, name: "Team", policyRevision: "1" }], nextCursor: null });
      if (request.action === "invitations" || request.action === "members") return reply({ items: [], nextCursor: null });
      if (request.action === "projects") return reply({ items: [{ id: project, name: "Demo" }], nextCursor: null });
      if (request.action === "reconcile") {
        operations = []; revision = "2";
        return reply(null, "account_service_operation_failed");
      }
      throw new Error(`unexpected action ${request.action}`);
    },
  });
  await controller.start();
  await controller.selectOrganization(organization);
  assert.equal(await controller.checkMutation(operationId), false);
  const { mutation, snapshot } = controller.getSnapshot();
  assert.deepEqual(snapshot.operations, []);
  assert.equal(mutation?.status, "unknown");
  assert.equal(mutation?.sourceAction, "cloud-commit");
  assert.equal(mutation?.canRetry, false);
  assert.equal(await controller.retryMutation(operationId), false);
  assert.equal(requests.some(request => request.action === "retry"), false);
});

test("targetless legacy package operations stay blocked until target migration", async () => {
  const requests = [];
  let operations = [packageSummary(undefined)];
  let revision = "1";
  const reply = (data = null) => account(operations, data, null, revision);
  const controller = new AccountController("cloud-recovery", {
    load: async () => reply(), operationId: () => project,
    dispatch: async request => {
      requests.push(request);
      if (request.action === "organizations") return reply({ items: [{ id: organization, name: "Team", policyRevision: "1" }], nextCursor: null });
      if (request.action === "invitations" || request.action === "members") return reply({ items: [], nextCursor: null });
      if (request.action === "projects") return reply({ items: [{ id: project, name: "Demo" }], nextCursor: null });
      throw new Error(`unexpected action ${request.action}`);
    },
  });
  await controller.start();
  await controller.selectOrganization(organization);
  assert.equal(await controller.createOrganization("Blocked until reviewed"), false);
  assert.equal(await controller.retryMutation(operationId), false);
  assert.equal(await controller.checkMutation(operationId), false);
  assert.equal(controller.getSnapshot().error, "account_package_target_migration_required");
  assert.equal(requests.some(request => request.action === "retry" || request.action === "reconcile"), false);
  assert.equal(await controller.acknowledgeOperation(operationId), false);
  assert.equal(await controller.createOrganization("Still blocked"), false);
  assert.deepEqual(controller.getSnapshot().snapshot.operations, [packageSummary(undefined)]);
  assert.equal(requests.some(request => request.action === "acknowledge" || request.action === "create-organization"), false);
});

test("committed cloud receipt preserves organization selection while a malformed revision freezes writes", async () => {
  for (const malformed of [false, true]) {
    const requests = [];
    let operations = [summary("unknown")];
    let revision = "1";
    const reply = (data = null) => account(operations, data, null, revision);
    const controller = new AccountController("cloud-recovery", {
      load: async () => reply(), operationId: () => project,
      dispatch: async request => {
        requests.push(request);
        if (request.action === "organizations") return reply({ items: [{ id: organization, name: "Team", policyRevision: "1" }], nextCursor: null });
        if (request.action === "invitations" || request.action === "members") return reply({ items: [], nextCursor: null });
        if (request.action === "projects") return reply({ items: [{ id: project, name: "Demo" }], nextCursor: null });
        if (request.action === "reconcile") {
          operations = [summary("committed")]; revision = "2";
          const outcome = committed();
          if (malformed) outcome.result.snapshot.revision = "2";
          return reply(outcome);
        }
        throw new Error(`unexpected action ${request.action}`);
      },
    });
    await controller.start();
    await controller.selectOrganization(organization);
    assert.equal(await controller.checkMutation(operationId), !malformed);
    assert.equal(controller.getSnapshot().selectedOrganization, organization);
    assert.equal(controller.getSnapshot().snapshot.operationsError, malformed ? "account_protocol_invalid" : null);
    if (malformed) {
      assert.equal(await controller.createOrganization("blocked"), false);
      assert.equal(requests.some(request => request.action === "create-organization"), false);
    }
  }
});

test("cloud notice exposes distinct Unknown, CAS conflict, ID reuse and committed text without a retry action", () => {
  for (const language of ["English", "Chinese"]) {
    const copy = accountCopy(language);
    const unknown = operationNoticeDetails(summary("unknown"), copy);
    assert.equal(unknown.message, copy.cloudUnknown);
    assert.equal(unknown.canCheck, true);
    assert.equal(unknown.canRetry, false);
    const cas = operationNoticeDetails(summary("failed", "account_cloud_conflict"), copy);
    assert.equal(cas.message, copy.cloudConflict);
    assert.equal(cas.canRetry, false);
    assert.equal(cas.canAcknowledge, true);
    const reused = operationNoticeDetails(summary("failed", "account_operation_id_conflict"), copy);
    assert.equal(reused.message, copy.cloudIdConflict);
    assert.equal(operationNoticeDetails(summary("committed"), copy).message, copy.cloudCommitted);
    const targetless = operationNoticeDetails(packageSummary(undefined), copy);
    assert.equal(targetless.message, copy.error("account_package_target_migration_required"));
    assert.equal(targetless.canCheck, false);
    assert.equal(targetless.canRetry, false);
    assert.equal(targetless.canAcknowledge, false);
  }
});

const require = createRequire(import.meta.url);
const playwrightRoot = process.env.ZIRCON_PLAYWRIGHT_NODE_MODULES;
const chromium = playwrightRoot ? require(path.join(playwrightRoot, "playwright")).chromium : undefined;
const executablePath = process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH;

test("browser keeps cloud recovery safe and blocks targetless legacy install recovery", { skip: !chromium || !executablePath }, async () => {
  const browserRoot = path.join(runRoot, "browser");
  const viteCacheDir = path.join(browserRoot, "vite-cache");
  const profileRoot = path.join(browserRoot, "profiles");
  const tempRoot = path.join(browserRoot, "temp");
  await Promise.all([viteCacheDir, profileRoot, tempRoot].map(directory => mkdir(directory, { recursive: true })));
  const vite = await createServer({ configFile: false, cacheDir: viteCacheDir, root: fileURLToPath(new URL("../", import.meta.url)), server: { host: "127.0.0.1", port: 0 } });
  const previousTemp = process.env.TEMP;
  const previousTmp = process.env.TMP;
  process.env.TEMP = tempRoot;
  process.env.TMP = tempRoot;
  let activeContext = null;
  try {
    await vite.listen();
    for (const mode of ["conflict", "dropped", "legacy-package"]) {
      const context = await chromium.launchPersistentContext(path.join(profileRoot, mode), { headless: true, executablePath, viewport: { width: 768, height: 900 } });
      activeContext = context;
      const shellText = await accountAdminShell("English");
      const initialOperation = mode === "legacy-package"
        ? { operationId, status: "unknown", action: "package-install", organizationId: organization, error: "account_service_outcome_unknown" }
        : { operationId, status: "unknown", action: "cloud-commit", organizationId: organization, error: "account_service_outcome_unknown" };
      await context.addInitScript(({ shellText, organization, operationId, mode, initialOperation }) => {
        let operations = [initialOperation];
        let revision = "1";
        window.__CLOUD_REQUESTS__ = [];
        const snapshot = (data = null) => ({ backendEpoch: "cloud-browser", account: { configured: true, status: "signed-in", issuer: "https://issuer.test", subject: "alice", displayName: "Alice", generation: "7", error: null }, data, error: null, operations, operationsRevision: revision, operationsError: null });
        window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
        window.__TAURI_INTERNALS__ = {
          transformCallback() { return 1; }, unregisterCallback() {},
          async invoke(command, args) {
            if (command === "plugin:event|listen") return 1;
            if (command === "hub_state") {
              const { fallbackShellState } = await import("/src/data/hubData.ts");
              return { ...fallbackShellState, backendEpoch: "cloud-browser", stateRevision: "1", activePage: "team", pageTitle: shellText.ui.shell.navItems.find(item => item.id === "team").label, pageSubtitle: "", ui: shellText.ui, comingSoon: [], settings: { ...fallbackShellState.settings, language: "English" } };
            }
            if (command === "account_state") return snapshot();
            if (command !== "account_action") return undefined;
            const request = args.request;
            window.__CLOUD_REQUESTS__.push(request);
            if (request.action === "organizations") return snapshot({ items: [{ id: organization, name: "Team", policyRevision: "1" }], nextCursor: null });
            if (["invitations", "members", "projects"].includes(request.action)) return snapshot({ items: [], nextCursor: null });
            if (request.action === "reconcile") {
              if (mode === "dropped") {
                operations = []; revision = "2";
                return { ...snapshot(), error: "account_service_operation_failed" };
              }
              operations = [{ operationId, status: "failed", action: "cloud-commit", organizationId: organization, error: "account_cloud_conflict" }];
              revision = "2";
              return snapshot({ status: "conflict", result: { status: "conflict", organizationId: organization, projectId: "00000000-0000-4000-8000-000000000002", baseRevision: "0", currentRevision: "1", manifestDigest: "a".repeat(64) } });
            }
            if (request.action === "acknowledge") { operations = []; revision = "3"; return snapshot(); }
            throw new Error(`unexpected action ${request.action}`);
          },
        };
      }, { shellText, organization, operationId, mode, initialOperation });
      const page = await context.newPage();
      const errors = [];
      page.on("pageerror", error => errors.push(error.message));
      await page.goto(vite.resolvedUrls.local[0]);
      const row = page.getByTestId(`account-operation-${operationId}`);
      if (mode === "legacy-package") {
        const copy = accountCopy("English");
        await row.getByText(copy.error("account_package_target_migration_required"), { exact: true }).waitFor();
        assert.equal(await row.getByRole("button", { name: "Retry" }).count(), 0);
        assert.equal(await row.getByRole("button", { name: "Check result" }).count(), 0);
        assert.equal(await row.getByRole("button", { name: "Dismiss" }).count(), 0);
        assert.deepEqual(await page.evaluate(() => window.__CLOUD_REQUESTS__.filter(request => ["reconcile", "retry", "acknowledge"].includes(request.action)).map(request => request.action)), []);
      } else {
        await row.getByText("Cloud outcome unknown. Check the service receipt before another write.").waitFor();
        assert.equal(await row.getByRole("button", { name: "Retry" }).count(), 0);
        await row.getByRole("button", { name: "Check result" }).click();
        if (mode === "dropped") {
          await row.waitFor({ state: "detached" });
          const local = page.getByTestId(`account-local-operation-${operationId}`);
          await local.getByText("Cloud outcome unknown. Check the service receipt before another write.").waitFor();
          assert.equal(await local.getByRole("button", { name: "Retry" }).count(), 0);
          await local.getByRole("button", { name: "Check result" }).click();
          await page.waitForFunction(() => window.__CLOUD_REQUESTS__.filter(request => request.action === "reconcile").length === 2);
          assert.equal(await local.getByRole("button", { name: "Retry" }).count(), 0);
          assert.deepEqual(await page.evaluate(() => window.__CLOUD_REQUESTS__.filter(request => ["cloud-commit", "reconcile", "retry", "acknowledge"].includes(request.action)).map(request => request.action)), ["reconcile", "reconcile"]);
        } else {
          await row.getByText("Remote head changed. This operation did not publish a snapshot.").waitFor();
          assert.equal(await row.getByRole("button", { name: "Retry" }).count(), 0);
          assert.equal(await row.getByRole("button", { name: "Check result" }).count(), 0);
          await row.getByRole("button", { name: "Dismiss" }).click();
          await row.waitFor({ state: "detached" });
          assert.deepEqual(await page.evaluate(() => window.__CLOUD_REQUESTS__.filter(request => ["cloud-commit", "reconcile", "retry", "acknowledge"].includes(request.action)).map(request => request.action)), ["reconcile", "acknowledge"]);
        }
      }
      assert.deepEqual(errors, []);
      await context.close();
      activeContext = null;
    }
  } finally {
    try {
      if (activeContext) await activeContext.close();
    } finally {
      try {
        await vite.close();
      } finally {
        if (previousTemp === undefined) delete process.env.TEMP; else process.env.TEMP = previousTemp;
        if (previousTmp === undefined) delete process.env.TMP; else process.env.TMP = previousTmp;
      }
    }
  }
});
