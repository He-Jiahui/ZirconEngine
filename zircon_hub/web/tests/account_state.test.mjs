import assert from "node:assert/strict";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import { createServer } from "vite";

import { AccountResponseFence, accountErrorCode } from "../src/account/fence.ts";
import { assertSnapshot, parsePage } from "../src/account/protocol.ts";

const cacheDir = path.join("E:/cargo-targets/zircon-engine/hub-web-validation", `account-state-${process.pid}`, "vite-cache");
await mkdir(cacheDir, { recursive: true });
const server = await createServer({ configFile: false, cacheDir, root: fileURLToPath(new URL("../", import.meta.url)), optimizeDeps: { noDiscovery: true, include: [] }, server: { middlewareMode: true }, appType: "custom" });
const { AccountController } = await server.ssrLoadModule("/src/account/controller.ts");
await server.close();

const snapshot = (backendEpoch, generation = "7") => ({
  backendEpoch,
  account: { configured: true, status: "signed-in", issuer: "https://issuer.test", subject: "owner", displayName: "Owner", generation, error: null },
  data: null,
  error: null,
  operations: [],
  operationsRevision: "0",
  operationsError: null,
});

test("account response fence rejects late operations from an older request", () => {
  const fence = new AccountResponseFence("hub-a");
  const first = fence.begin("7");
  const second = fence.begin("7");

  assert.equal(fence.accepts(first, snapshot("hub-a"), "7"), false);
  assert.equal(fence.accepts(second, snapshot("hub-a"), "7"), true);
});

test("account response fence rejects a restarted Hub and a changed account generation", () => {
  const fence = new AccountResponseFence("hub-a");
  const request = fence.begin("7");

  assert.equal(fence.accepts(request, snapshot("hub-b"), "7"), false);
  assert.equal(fence.accepts(request, snapshot("hub-a", "8"), "8"), false);
  fence.reset("hub-b");
  assert.equal(fence.accepts(request, snapshot("hub-a"), "7"), false);
});

test("native account errors never expose arbitrary provider text", () => {
  assert.equal(accountErrorCode("identity_provider_unavailable"), "identity_provider_unavailable");
  assert.equal(accountErrorCode(new Error("https://idp.example/token?access_token=secret")), "account_service_operation_failed");
  assert.equal(accountErrorCode("access_token_secret"), "account_service_operation_failed");
});

test("independent account resource scopes accept both replies and reject mismatched response generation", () => {
  const fence = new AccountResponseFence("hub-a");
  const organizations = fence.begin("7", "organizations");
  const invitations = fence.begin("7", "invitations");
  assert.equal(fence.accepts(organizations, snapshot("hub-a"), "7"), true);
  assert.equal(fence.accepts(invitations, snapshot("hub-a"), "7"), true);
  assert.equal(fence.accepts(organizations, snapshot("hub-a", "8"), "7"), false);
});

const orgA = "00000000-0000-4000-8000-000000000001";
const orgB = "00000000-0000-4000-8000-000000000002";
const operation = "00000000-0000-4000-8000-000000000003";
const operationSummary = (status = "unknown", operationId = operation) => ({ operationId, status, action: "create-organization", organizationId: null, error: status === "unknown" ? "account_service_outcome_unknown" : status === "failed" ? "account_service_operation_failed" : null });
const org = id => ({ id, name: id === orgA ? "First" : "Second", policyRevision: "1" });
const page = items => ({ items, nextCursor: null });
const signed = (generation = "7", data = null, error = null) => ({
  backendEpoch: "hub-a", account: { configured: true, status: "signed-in", issuer: "https://issuer.test", subject: "owner", displayName: "Owner", generation, error: null }, data, error,
  operations: [], operationsRevision: "0", operationsError: null,
});
const signedOut = (generation = "8", error = null) => ({
  backendEpoch: "hub-a", account: { configured: true, status: "signed-out", issuer: null, subject: null, displayName: null, generation, error: null }, data: null, error,
  operations: [], operationsRevision: "0", operationsError: null,
});
function deferred() { let resolve; const promise = new Promise(done => { resolve = done; }); return { promise, resolve }; }
function controller(dispatch, initial = signed()) {
  return new AccountController("hub-a", { load: async () => initial, dispatch, operationId: () => operation });
}

test("account protocol rejects malformed identity and preserves exact member and invitation DTOs", () => {
  assertSnapshot(signed());
  for (const invalid of [undefined, { ...signed(), account: { ...signed().account, generation: 7 } }, { ...signed(), account: { ...signed().account, subject: null } }, { ...signedOut(), account: signed().account, error: undefined }]) {
    assert.throws(() => assertSnapshot(invalid), /account_protocol_invalid/);
  }
  const member = { issuer: "https://issuer.test", subject: "actual-subject", role: "owner", active: true };
  assert.deepEqual(parsePage("members", page([member])).items, [member]);
  assert.throws(() => parsePage("members", page([{ name: "fake" }])), /account_protocol_invalid/);
});

test("parallel overview loads retain organizations and invitations independently", async () => {
  const organizations = deferred();
  const invitations = deferred();
  const client = controller(request => request.action === "organizations" ? organizations.promise : invitations.promise);
  const loading = client.start();
  await new Promise(resolve => setImmediate(resolve));
  invitations.resolve(signed("7", page([])));
  organizations.resolve(signed("7", page([org(orgA)])));
  await loading;
  assert.deepEqual(client.getSnapshot().organizations.items, [org(orgA)]);
  assert.equal(client.getSnapshot().invitations.loaded, true);
  assert.equal(client.getSnapshot().organizations.loading, false);
});

test("switching organizations clears old data and rejects both late old resource responses", async () => {
  const firstMembers = deferred();
  const firstProjects = deferred();
  const client = controller(request => {
    if (request.action === "organizations") return Promise.resolve(signed("7", page([org(orgA), org(orgB)])));
    if (request.action === "invitations") return Promise.resolve(signed("7", page([])));
    if (request.organization === orgA) return request.action === "members" ? firstMembers.promise : firstProjects.promise;
    return Promise.resolve(signed("7", page(request.action === "members" ? [{ issuer: "https://issuer.test", subject: "second", role: "member", active: true }] : [{ id: orgB, name: "Second project" }])));
  });
  await client.start();
  const first = client.selectOrganization(orgA);
  await client.selectOrganization(orgB);
  firstMembers.resolve(signed("7", page([{ issuer: "https://issuer.test", subject: "first", role: "owner", active: true }])));
  firstProjects.resolve(signed("7", page([{ id: orgA, name: "First project" }])));
  await first;
  assert.equal(client.getSnapshot().selectedOrganization, orgB);
  assert.equal(client.getSnapshot().members.items[0].subject, "second");
  assert.equal(client.getSnapshot().projects.items[0].name, "Second project");
});

test("cancel invalidates late login and publishes signed-out state despite revocation failure", async () => {
  const login = deferred();
  const client = controller(request => request.action === "sign-in" ? login.promise : Promise.resolve(signedOut("10", "identity_provider_unavailable")), signedOut("7"));
  await client.start();
  const pendingLogin = client.authenticate("sign-in");
  await client.authenticate("cancel");
  login.resolve(signed("9"));
  await pendingLogin;
  assert.equal(client.getSnapshot().snapshot.account.status, "signed-out");
  assert.equal(client.getSnapshot().snapshot.account.generation, "10");
  assert.equal(client.getSnapshot().error, "identity_provider_unavailable");
});

test("malformed current response clears loading and exposes protocol failure", async () => {
  const client = controller(async request => request.action === "organizations" ? undefined : signed("7", page([])));
  await client.start();
  assert.equal(client.getSnapshot().organizations.loading, false);
  assert.equal(client.getSnapshot().organizations.error, "account_protocol_invalid");
});

test("failed restart clears stale signed-in authority and blocks queries", async () => {
  let failLoad = false;
  const requests = [];
  const client = new AccountController("hub-a", {
    load: async () => {
      if (failLoad) throw new Error("identity_provider_unavailable");
      return signed();
    },
    dispatch: async request => {
      requests.push(request);
      return signed("7", page([]));
    },
    operationId: () => operation,
  });
  await client.start();
  requests.length = 0;
  failLoad = true;

  await client.start();

  const state = client.getSnapshot();
  assert.equal(state.ready, true);
  assert.equal(state.snapshot.account.status, "unavailable");
  assert.equal(state.snapshot.account.configured, false);
  assert.equal(state.selectedOrganization, null);
  assert.deepEqual(state.organizations.items, []);
  assert.deepEqual(state.members.items, []);
  assert.equal(state.error, "identity_provider_unavailable");
  assert.equal(await client.createOrganization("stale"), false);
  await client.load("organizations");
  assert.deepEqual(requests, []);
});

test("successful restart with a signed-out account clears every cached tenant resource", async () => {
  let current = signed();
  const client = new AccountController("hub-a", {
    load: async () => current,
    dispatch: async request => {
      if (current.account.status === "signed-out") return current;
      if (request.action === "organizations") return signed("7", page([org(orgA)]));
      if (request.action === "members") return signed("7", page([{ issuer: "https://issuer.test", subject: "owner", role: "owner", active: true }]));
      if (request.action === "catalog") return signed("7", { items: [{ package_id: "00000000-0000-4000-8000-000000000004", revision: "1", version: "1.0.0", name: "Package", kind: "plugin", description: "", iss: "https://issuer.test", aud: "zircon-hub", sub: "publisher", exp: 4102444800, license_id: "MIT", license_text: "ok", artifact_digest: "ab".repeat(32), artifact_size: 1 }], nextCursor: null });
      if (request.action === "catalog-entitlements") return signed("7", page([{ packageId: "00000000-0000-4000-8000-000000000004", revision: "1", licenseId: "MIT" }]));
      if (request.action === "package-inventory") return signed("7", { schemaVersion: 2, targetMode: request.targetMode, revision: "1", packages: [] });
      return signed("7", page([]));
    },
    operationId: () => operation,
  });
  await client.start();
  await client.selectOrganization(orgA);
  await client.loadCatalog();
  await client.loadCatalogEntitlements();
  await client.loadPackageInventory("editor_host");
  assert.equal(client.getSnapshot().organizations.items.length, 1);
  assert.equal(client.getSnapshot().serviceCatalog.items.length, 1);
  assert.equal(client.getSnapshot().packageInventory.targetMode, "editor_host");

  current = signedOut("8");
  await client.start();

  const state = client.getSnapshot();
  assert.equal(state.snapshot.account.status, "signed-out");
  assert.equal(state.selectedOrganization, null);
  assert.deepEqual(state.organizations.items, []);
  assert.equal(state.packageInventory.targetMode, null);
  assert.deepEqual(state.members.items, []);
  assert.deepEqual(state.projects.items, []);
  assert.deepEqual(state.serviceCatalog.items, []);
  assert.deepEqual(state.catalogEntitlements.items, []);
  assert.equal(state.packageInventory.data, null);
});

test("a resource reply that changes authority cannot merge into the previous tenant cache", async () => {
  let switched = false;
  const client = controller(async request => {
    if (request.action === "organizations") return switched ? signedOut("7") : signed("7", page([org(orgA)]));
    if (request.action === "members") return signed("7", page([{ issuer: "https://issuer.test", subject: "owner", role: "owner", active: true }]));
    if (request.action === "projects") return signed("7", page([{ id: orgA, name: "Old tenant project" }]));
    return signed("7", page([]));
  });
  await client.start();
  await client.selectOrganization(orgA);
  assert.equal(client.getSnapshot().projects.items.length, 1);

  switched = true;
  await client.load("organizations");

  const state = client.getSnapshot();
  assert.equal(state.snapshot.account.status, "signed-out");
  assert.equal(state.selectedOrganization, null);
  assert.deepEqual(state.organizations.items, []);
  assert.deepEqual(state.projects.items, []);
  assert.deepEqual(state.members.items, []);
});

test("organization refresh rejects a backwards page cursor before selecting a stale tenant", async () => {
  let refreshing = false;
  let refreshPage = 0;
  const client = controller(async request => {
    if (request.action === "organizations") {
      if (!refreshing) return signed("7", page([org(orgA)]));
      refreshPage += 1;
      return refreshPage === 1
        ? signed("7", { items: [org(orgB)], nextCursor: orgB })
        : signed("7", { items: [org(orgA)], nextCursor: orgA });
    }
    if (request.action === "members") return signed("7", page([{ issuer: "https://issuer.test", subject: "owner", role: "owner", active: true }]));
    if (request.action === "projects") return signed("7", page([]));
    return signed("7", page([]));
  });
  await client.start();
  await client.selectOrganization(orgA);
  refreshing = true;
  await client.refreshOrganization();

  const state = client.getSnapshot();
  assert.equal(state.selectedOrganization, null);
  assert.deepEqual(state.organizations.items, []);
  assert.equal(state.organizations.error, "account_protocol_invalid");
});

test("operation journal reload failure blocks new mutations until the journal recovers", async () => {
  let failLoad = false;
  let creates = 0;
  const client = new AccountController("hub-a", {
    load: async () => {
      if (failLoad) throw new Error("account_operation_store_unavailable");
      return signed();
    },
    dispatch: async request => {
      if (request.action === "create-organization") creates += 1;
      return signed("7", page([]));
    },
    operationId: () => operation,
  });
  await client.start();
  failLoad = true;

  await client.reloadOperations();

  assert.equal(client.getSnapshot().snapshot.operationsError, "account_operation_store_unavailable");
  await client.load("organizations");
  assert.equal(client.getSnapshot().snapshot.operationsError, "account_operation_store_unavailable");
  assert.equal(await client.createOrganization("blocked"), false);
  assert.equal(creates, 0);
});

test("failed organization refresh removes cached membership and project authority", async () => {
  let failRefresh = false;
  const client = controller(async request => {
    if (request.action === "organizations") return failRefresh ? signed("7", null, "account_service_operation_failed") : signed("7", page([org(orgA)]));
    if (request.action === "members") return signed("7", page([{ issuer: "https://issuer.test", subject: "owner", role: "owner", active: true }]));
    if (request.action === "projects") return signed("7", page([{ id: orgB, name: "Project" }]));
    return signed("7", page([]));
  });
  await client.start();
  await client.selectOrganization(orgA);
  assert.equal(client.getSnapshot().members.items.length, 1);
  failRefresh = true;
  await client.load("organizations");
  const state = client.getSnapshot();
  assert.equal(state.selectedOrganization, null);
  assert.deepEqual(state.organizations.items, []);
  assert.deepEqual(state.members.items, []);
  assert.deepEqual(state.projects.items, []);
  assert.equal(state.organizations.error, "account_service_operation_failed");
});

test("unknown create outcome retains operation id across receipt lookup and retry", async () => {
  const requests = [];
  let operations = [];
  let revision = "0";
  const response = (data = null, error = null) => ({ ...signed("7", data, error), operations, operationsRevision: revision });
  const client = controller(async request => {
    requests.push(request);
    if (request.action === "create-organization") { operations = [operationSummary()]; revision = "1"; return response(null, "account_service_outcome_unknown"); }
    if (request.action === "reconcile") return response({ status: "unknown" });
    if (request.action === "retry") { operations = [operationSummary("committed")]; revision = "2"; return response(org(orgA)); }
    return response(page([]));
  });
  await client.start();
  assert.equal(await client.createOrganization("Name"), false);
  assert.equal(client.getSnapshot().snapshot.operations[0].status, "unknown");
  assert.equal(await client.createOrganization("Second"), false);
  await client.checkMutation(operation);
  assert.equal(client.getSnapshot().snapshot.operations[0].status, "unknown");
  assert.equal(await client.retryMutation(operation), true);
  assert.equal(requests.filter(request => request.action === "create-organization").length, 1);
  assert.deepEqual(requests.find(request => request.action === "retry"), { action: "retry", backendEpoch: "hub-a", generation: "7", operationId: operation });
  assert.equal(client.getSnapshot().mutation, null);
});

test("pagination keeps previous rows and advances the server cursor without dropping data", async () => {
  const client = controller(async request => request.action === "organizations"
    ? signed("7", request.after ? page([org(orgB)]) : { items: [org(orgA)], nextCursor: orgA })
    : signed("7", page([])));
  await client.start();
  await client.load("organizations", true);
  assert.deepEqual(client.getSnapshot().organizations.items.map(item => item.id), [orgA, orgB]);
  assert.equal(client.getSnapshot().organizations.nextCursor, null);
});

test("resource pagination rejects a backwards cursor and rows from the prior page", async () => {
  const client = controller(async request => {
    if (request.action !== "organizations") return signed("7", page([]));
    return request.after
      ? signed("7", { items: [org(orgA)], nextCursor: orgA })
      : signed("7", { items: [org(orgA)], nextCursor: orgB });
  });
  await client.start();
  await client.load("organizations", true);

  const state = client.getSnapshot().organizations;
  assert.equal(state.error, "account_protocol_invalid");
  assert.deepEqual(state.items, [org(orgA)]);
  assert.equal(state.nextCursor, orgB);
});

test("member pagination rejects a numeric cursor that moves backwards", async () => {
  const member = { issuer: "https://issuer.test", subject: "owner", role: "owner", active: true };
  const client = controller(async request => {
    if (request.action === "organizations") return signed("7", page([org(orgA)]));
    if (request.action === "members") return signed("7", request.after ? { items: [member], nextCursor: "1" } : { items: [member], nextCursor: "2" });
    return signed("7", page([]));
  });
  await client.start();
  await client.selectOrganization(orgA);
  await client.load("members", true);

  const state = client.getSnapshot().members;
  assert.equal(state.error, "account_protocol_invalid");
  assert.deepEqual(state.items, [member]);
  assert.equal(state.nextCursor, "2");
});

test("entitlement pagination rejects duplicates that cross page boundaries", async () => {
  const entitlement = { packageId: "00000000-0000-4000-8000-000000000004", revision: "1", licenseId: "license-a" };
  const client = controller(async request => {
    if (request.action === "organizations") return signed("7", page([org(orgA)]));
    if (request.action === "members") return signed("7", page([{ issuer: "https://issuer.test", subject: "owner", role: "owner", active: true }]));
    if (request.action === "catalog-entitlements") return signed("7", request.after ? { items: [entitlement], nextCursor: null } : { items: [entitlement], nextCursor: "1" });
    return signed("7", page([]));
  });
  await client.start();
  await client.selectOrganization(orgA);
  await client.loadCatalogEntitlements();
  await client.loadCatalogEntitlements(true);

  const state = client.getSnapshot().catalogEntitlements;
  assert.equal(state.error, "account_protocol_invalid");
  assert.deepEqual(state.items, [entitlement]);
  assert.equal(state.loading, false);
});

test("cold restart restores an unknown journal operation and reconciles it without a new payload", async () => {
  const restored = operationSummary();
  const requests = [];
  const client = controller(async request => {
    requests.push(request);
    if (request.action === "organizations" || request.action === "invitations") return signed("7", page([]));
    if (request.action === "reconcile") return { ...signed("7"), operations: [{ ...restored, status: "committed", error: null }], operationsRevision: "2" };
    throw new Error("unexpected account request");
  }, { ...signed(), operations: [restored], operationsRevision: "1" });
  await client.start();
  assert.equal(client.getSnapshot().snapshot.operations[0].status, "unknown");
  assert.equal(await client.createOrganization("blocked"), false);
  await client.checkMutation(operation);
  assert.equal(client.getSnapshot().snapshot.operations[0].status, "committed");
  assert.deepEqual(requests.filter(request => request.action === "reconcile")[0], { action: "reconcile", backendEpoch: "hub-a", generation: "7", operationId: operation });
});

test("operation store failure preserves the prior journal and blocks new mutations", async () => {
  const restored = operationSummary();
  const client = controller(async request => {
    if (request.action === "organizations" || request.action === "invitations") return { ...signed("7", page([])), operations: [], operationsRevision: "0", operationsError: "account_operation_store_unavailable" };
    throw new Error("unexpected account request");
  }, { ...signed(), operations: [restored], operationsRevision: "2" });
  await client.start();
  assert.deepEqual(client.getSnapshot().snapshot.operations, [restored]);
  assert.equal(client.getSnapshot().snapshot.operationsError, "account_operation_store_unavailable");
  assert.equal(await client.createOrganization("blocked"), false);
});

test("late resource replies cannot replace a newer operation revision or resurrect acknowledged records", async () => {
  const oldOrganizations = deferred();
  let requests = 0;
  const client = controller(async request => {
    if (request.action === "organizations" && ++requests === 2) return oldOrganizations.promise;
    if (request.action === "acknowledge") return { ...signed(), operationsRevision: "9" };
    return { ...signed("7", page([])), operations: [operationSummary("committed")], operationsRevision: "8" };
  }, { ...signed(), operations: [operationSummary("committed")], operationsRevision: "8" });
  await client.start();
  const pending = client.load("organizations");
  assert.equal(await client.acknowledgeOperation(operation), true);
  oldOrganizations.resolve({ ...signed("7", page([org(orgA)])), operations: [operationSummary()], operationsRevision: "7" });
  await pending;
  assert.equal(client.getSnapshot().snapshot.operationsRevision, "9");
  assert.deepEqual(client.getSnapshot().snapshot.operations, []);
  assert.deepEqual(client.getSnapshot().organizations.items, [org(orgA)]);
});

test("operation store recovery reloads native state and unknown records cannot be acknowledged", async () => {
  let recovering = false;
  let creates = 0;
  const native = () => recovering ? { ...signed(), operations: [operationSummary()], operationsRevision: "4" }
    : { ...signed(), operationsError: "account_operation_store_unavailable" };
  const client = new AccountController("hub-a", {
    load: async () => native(), operationId: () => operation,
    dispatch: async request => { if (request.action === "create-organization") creates++; return { ...native(), data: page([]) }; },
  });
  await client.start();
  assert.equal(await client.createOrganization("blocked"), false);
  recovering = true;
  await client.reloadOperations();
  assert.equal(client.getSnapshot().snapshot.operationsError, null);
  assert.equal(await client.acknowledgeOperation(operation), false);
  assert.equal(await client.createOrganization("still blocked"), false);
  assert.equal(creates, 0);
});

test("a lost IPC reply retries the same original request when no journal summary reached the UI", async () => {
  const requests = [];
  const client = controller(async request => {
    if (request.action !== "create-organization") return signed("7", page([]));
    requests.push(request);
    if (requests.length === 1) throw new Error("IPC disconnected");
    return { ...signed("7", org(orgA)), operations: [operationSummary("committed")], operationsRevision: "2" };
  });
  await client.start();
  assert.equal(await client.createOrganization("Original"), false);
  assert.equal(client.getSnapshot().mutation.status, "unknown");
  client.dismissFailedMutation();
  assert.equal(await client.createOrganization("Duplicate"), false);
  assert.equal(await client.retryMutation(operation), true);
  assert.deepEqual(requests[0], requests[1]);
});

test("account switches clear the visible journal and reject late recovery from the old identity", async () => {
  const recovery = deferred();
  const old = { ...signed(), operations: [operationSummary()], operationsRevision: "3" };
  const different = { ...signed("9"), account: { ...signed("9").account, subject: "other" } };
  let current = old;
  const client = controller(async request => {
    if (request.action === "reconcile") return recovery.promise;
    if (request.action === "logout") { current = signedOut("8"); return current; }
    if (request.action === "sign-in") { current = different; return current; }
    return { ...current, data: page([]) };
  }, old);
  await client.start();
  const pending = client.checkMutation(operation);
  await client.authenticate("logout");
  await client.authenticate("sign-in");
  recovery.resolve({ ...old, operations: [operationSummary("committed")], operationsRevision: "4" });
  await pending;
  assert.equal(client.getSnapshot().snapshot.account.subject, "other");
  assert.deepEqual(client.getSnapshot().snapshot.operations, []);
});

test("the native operation summary rejects duplicate ids, invalid revisions and identity leakage", () => {
  for (const invalid of [
    { ...signed(), operations: [operationSummary(), operationSummary()] },
    { ...signed(), operationsRevision: "18446744073709551616" },
    { ...signed(), operationsRevision: "01" },
    { ...signed(), operations: [{ ...operationSummary(), status: "invented" }] },
    { ...signed(), operations: [{ ...operationSummary(), action: "mutate", organizationId: null }] },
    { ...signedOut(), operations: [operationSummary()] },
  ]) assert.throws(() => assertSnapshot(invalid), /account_protocol_invalid/);
});

const require = createRequire(import.meta.url);
const playwrightRoot = process.env.ZIRCON_PLAYWRIGHT_NODE_MODULES;
const chromium = playwrightRoot ? require(path.join(playwrightRoot, "playwright")).chromium : undefined;
const launchOptions = process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH ? { executablePath: process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH } : {};
const browserUrl = process.env.ZIRCON_HUB_TEST_URL;
const screenshotDir = process.env.ZIRCON_HUB_SCREENSHOT_DIR;
const browserEnabled = Boolean(chromium && browserUrl);

async function accountBrowserFixture(browser, width, language, signedIn, recovered = false) {
  const context = await browser.newContext({ viewport: { width, height: 900 } });
  await context.addInitScript(({ language, signedIn, orgA, orgB, operation, recovered }) => {
    let generation = 7;
    let signed = signedIn;
    let operations = recovered ? [{ operationId: operation, status: "unknown", action: "create-organization", organizationId: null, error: "account_service_outcome_unknown" }] : [];
    let operationsRevision = recovered ? 1 : 0;
    let shell;
    window.__ACCOUNT_REQUESTS__ = [];
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    const account = (data = null, error = null) => ({ backendEpoch: "account-ui", account: { configured: true, status: signed ? "signed-in" : "signed-out", issuer: signed ? "https://identity.example.test/realms/zircon" : null, subject: signed ? "owner" : null, displayName: signed ? "Workspace Owner With A Long Display Name" : null, generation: String(generation), error: null }, data, error, operations: signed ? operations : [], operationsRevision: signed ? String(operationsRevision) : "0", operationsError: null });
    window.__TAURI_INTERNALS__ = {
      transformCallback() { return 1; }, unregisterCallback() {},
      async invoke(command, args) {
        if (command === "plugin:event|listen") return 1;
        if (command === "hub_state") {
          const { fallbackShellState } = await import("/src/data/hubData.ts");
          shell = { ...fallbackShellState, backendEpoch: "account-ui", stateRevision: "1", activePage: "team", pageTitle: language === "Chinese" ? "团队" : "Team", settings: { ...fallbackShellState.settings, language } };
          return shell;
        }
        if (command === "account_state") return account();
        if (command === "account_action") {
          const request = args.request;
          window.__ACCOUNT_REQUESTS__.push(request);
          if (request.action === "sign-in") return new Promise(resolve => { window.__FINISH_LOGIN__ = () => { signed = true; generation += 1; resolve(account()); }; });
          if (request.action === "cancel" || request.action === "logout") { signed = false; generation += 1; return account(null, "identity_provider_unavailable"); }
          if (request.action === "organizations") return account({ items: [{ id: orgA, name: "A very long organization name that remains readable on narrow windows", policyRevision: "1" }, { id: orgB, name: "Second organization", policyRevision: "1" }], nextCursor: null });
          if (request.action === "members") return account({ items: [{ issuer: "https://identity.example.test/realms/zircon", subject: "owner", role: "owner", active: true }], nextCursor: null });
          if (request.action === "projects") return account({ items: [{ id: orgA, name: "Shared project" }], nextCursor: null });
          if (request.action === "invitations") return account({ items: [], nextCursor: null });
          if (request.action === "create-organization") {
            operations = [{ operationId: request.operationId, status: "unknown", action: "create-organization", organizationId: null, error: "account_service_outcome_unknown" }];
            operationsRevision += 1;
            return account(null, "account_service_outcome_unknown");
          }
          if (request.action === "reconcile") return account({ status: "unknown" });
          if (request.action === "retry") {
            operations = operations.map(item => item.operationId === request.operationId ? { ...item, status: "committed", error: null } : item);
            operationsRevision += 1;
            return account({ id: orgA, name: "Restored organization", policyRevision: "1" });
          }
          if (request.action === "acknowledge") {
            operations = operations.filter(item => item.operationId !== request.operationId);
            operationsRevision += 1;
            return account();
          }
          throw new Error("unexpected account request");
        }
        return undefined;
      },
    };
  }, { language, signedIn, orgA, orgB, operation, recovered });
  const page = await context.newPage();
  await page.goto(browserUrl);
  await page.getByTestId("hub-account").waitFor();
  return { context, page };
}

test("account UI cancels pending login and retains authoritative logout failure", { skip: !browserEnabled }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const { page } = await accountBrowserFixture(browser, 768, "English", false);
    const account = page.getByTestId("hub-account");
    await account.getByRole("button", { name: "Sign in", exact: true }).click();
    await account.getByRole("button", { name: "Cancel sign-in", exact: true }).click();
    await account.getByText("Identity service is unavailable", { exact: true }).waitFor();
    await page.evaluate(() => window.__FINISH_LOGIN__());
    assert.equal(await account.getByTestId("account-identity").count(), 0);
    await account.getByRole("button", { name: "Sign in", exact: true }).waitFor();
  } finally { await browser.close(); }
});

test("account UI restores unknown operations, retries their ids and acknowledges committed results", { skip: !browserEnabled }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const { page } = await accountBrowserFixture(browser, 360, "English", true, true);
    const account = page.getByTestId("hub-account");
    const notice = account.getByTestId(`account-operation-${operation}`);
    await notice.getByText("Operation outcome unknown", { exact: true }).waitFor();
    assert.equal(await account.getByLabel("Organization name", { exact: true }).isDisabled(), true);
    assert.equal(await notice.getByRole("button", { name: "Dismiss", exact: true }).count(), 0);
    await notice.getByRole("button", { name: "Check result", exact: true }).click();
    await notice.getByText("Operation outcome unknown", { exact: true }).waitFor();
    await notice.getByRole("button", { name: "Retry", exact: true }).click();
    await notice.getByText("Operation committed", { exact: true }).waitFor();
    await notice.getByRole("button", { name: "Dismiss", exact: true }).click();
    await notice.waitFor({ state: "detached" });
    assert.equal(await account.getByLabel("Organization name", { exact: true }).isEnabled(), true);
    const requests = await page.evaluate(() => window.__ACCOUNT_REQUESTS__.filter(request => ["retry", "reconcile", "acknowledge"].includes(request.action)));
    assert.deepEqual(requests.map(request => request.action), ["reconcile", "retry", "acknowledge"]);
    assert.ok(requests.every(request => request.operationId === operation && Object.keys(request).length === 4));
  } finally { await browser.close(); }
});

for (const width of [360, 768, 1280, 1920]) {
  for (const language of ["English", "Chinese"]) {
    test(`account browser layout at ${width}px in ${language}`, { skip: !browserEnabled }, async () => {
      const browser = await chromium.launch({ headless: true, ...launchOptions });
      try {
        const { page } = await accountBrowserFixture(browser, width, language, true, true);
        const account = page.getByTestId("hub-account");
        await account.getByRole("button", { name: "A very long organization name that remains readable on narrow windows", exact: true }).click();
        await account.getByText("owner", { exact: true }).waitFor();
        const overflows = await account.evaluate(element => [...element.querySelectorAll("button, input, p, h6")].filter(node => {
          const rect = node.getBoundingClientRect();
          return rect.width > 0 && (rect.left < -1 || rect.right > innerWidth + 1);
        }).map(node => node.textContent));
        assert.deepEqual(overflows, []);
        assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
        if (screenshotDir) {
          await mkdir(screenshotDir, { recursive: true });
          await page.screenshot({ path: path.join(screenshotDir, `account-${width}-${language}.png`), fullPage: true });
        }
      } finally { await browser.close(); }
    });
  }
}
