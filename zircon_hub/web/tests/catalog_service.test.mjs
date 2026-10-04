import assert from "node:assert/strict";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const server = await createServer({ configFile: false, root: fileURLToPath(new URL("../", import.meta.url)), cacheDir: `E:/cargo-targets/zircon-engine/hub-web-validation/catalog-service-${process.pid}/vite-cache`, optimizeDeps: { noDiscovery: true, include: [] }, server: { middlewareMode: true }, appType: "custom" });
const { AccountController } = await server.ssrLoadModule("/src/account/controller.ts");
const { parseCatalogPage, parseEntitlements, parseInventory, parseInstallReceipt, isInstalled } = await server.ssrLoadModule("/src/catalog/service/protocol.ts");
const { assertSnapshot } = await server.ssrLoadModule("/src/account/protocol.ts");
const { accountErrorCode } = await server.ssrLoadModule("/src/account/fence.ts");
await server.close();

const orgA = "00000000-0000-4000-8000-000000000001";
const orgB = "00000000-0000-4000-8000-000000000002";
const pkgA = "00000000-0000-4000-8000-000000000011";
const pkgB = "00000000-0000-4000-8000-000000000012";
const opA = "00000000-0000-4000-8000-000000000021";
const issuer = "https://issuer.test";
const release = (packageId = pkgA) => ({ iss: "https://catalog.test", aud: "zircon-hub", sub: "publisher", exp: 4102444800, package_id: packageId, revision: "9007199254740993", version: "1.2.3", name: packageId === pkgA ? "Terrain tools" : "Material tools", kind: "plugin", description: "Editor tools", license_id: "MIT", license_text: "Permission is hereby granted.", artifact_digest: "ab".repeat(32), artifact_size: 12 });
const entitlement = (packageId = pkgA) => ({ packageId, revision: release().revision, licenseId: "MIT" });
const installed = (operationId = opA, packageId = pkgA) => ({ operationId, packageId, version: "1.2.3", releaseRevision: release().revision, artifactDigest: release().artifact_digest });
const inventory = (packages = [], revision = "0", targetMode = "editor_host") => ({ schemaVersion: 2, targetMode, revision, packages });
const page = (items, nextCursor = null) => ({ items, nextCursor });
function deferred() { let resolve; const promise = new Promise(done => { resolve = done; }); return { promise, resolve }; }

function fixture(override = () => undefined, initialOperations = []) {
  let generation = "7";
  let signedIn = true;
  let operations = initialOperations;
  let operationsRevision = "0";
  let nextId = 33;
  let operationIds = 0;
  const requests = [];
  const snapshot = (data = null, error = null) => ({ backendEpoch: "catalog-ui", account: { configured: true, status: signedIn ? "signed-in" : "signed-out", issuer: signedIn ? issuer : null, subject: signedIn ? "owner" : null, displayName: signedIn ? "Owner" : null, generation, error: null }, data, error, operations: signedIn ? operations : [], operationsRevision: signedIn ? operationsRevision : "0", operationsError: null });
  const setOperation = (request, status, data = null) => {
    const existing = operations.find(item => item.operationId === request.operationId);
    const targetMode = existing?.targetMode ?? request.targetMode;
    operations = [...operations.filter(item => item.operationId !== request.operationId), { operationId: request.operationId, action: existing?.action ?? request.action, organizationId: existing?.organizationId ?? request.organization, ...(targetMode ? { targetMode } : {}), status, error: status === "unknown" ? "account_service_outcome_unknown" : status === "failed" ? "account_service_operation_failed" : null }];
    operationsRevision = String(BigInt(operationsRevision) + 1n);
    return snapshot(data, operations.at(-1).error);
  };
  const client = new AccountController("catalog-ui", {
    load: async () => snapshot(), operationId: () => { operationIds++; return `00000000-0000-4000-8000-${String(nextId++).padStart(12, "0")}`; },
    dispatch: async request => {
      requests.push(request);
      const overridden = override(request, { snapshot, setOperation });
      if (overridden !== undefined) return overridden;
      if (request.action === "logout") { generation = String(BigInt(generation) + 1n); signedIn = false; return snapshot(); }
      if (request.action === "refresh") { generation = String(BigInt(generation) + 1n); return snapshot(); }
      if (request.action === "organizations") return snapshot(page([{ id: orgA, name: "Team A", policyRevision: "3" }, { id: orgB, name: "Team B", policyRevision: "4" }]));
      if (request.action === "members") return snapshot(page([{ issuer, subject: "owner", active: true, role: "owner" }]));
      if (["projects", "invitations", "issued-invitations"].includes(request.action)) return snapshot(page([]));
      if (request.action === "catalog") return snapshot(page([release()]));
      if (request.action === "catalog-entitlements") return snapshot(page([entitlement()]));
      if (request.action === "package-inventory") return snapshot(inventory([], "0", request.targetMode ?? "editor_host"));
      if (request.action === "acknowledge") { operations = operations.filter(item => item.operationId !== request.operationId); operationsRevision = String(BigInt(operationsRevision) + 1n); return snapshot(); }
      throw new Error(`unexpected test action ${request.action}`);
    },
  });
  return { client, requests, snapshot, setOperation, operationIds: () => operationIds };
}

async function ready(f, targetMode = "editor_host") {
  await f.client.start(); await f.client.selectOrganization(orgA); await f.client.loadCatalog();
  await f.client.loadCatalogEntitlements(); await f.client.loadPackageInventory(targetMode);
  return f.client.getSnapshot().serviceCatalog.items[0];
}

test("service parsers retain exact decimal revisions, empty signed pages and no private native fields", () => {
  const source = release();
  assert.deepEqual(parseCatalogPage(page([source])).items, [source]);
  assert.deepEqual(parseCatalogPage(page([], pkgA)), page([], pkgA));
  assert.deepEqual(parseEntitlements(page([entitlement()], "9007199254740993")), page([entitlement()], "9007199254740993"));
  const packageRow = { ...installed(), requestDigest: "private", slot: "private", files: ["private"], identityDigest: "private" };
  assert.deepEqual(parseInventory(inventory([packageRow], "9007199254740993")), inventory([installed()], "9007199254740993"));
  const receipt = { schemaVersion: 2, targetMode: "editor_host", operationId: opA, inventoryRevision: "1", package: packageRow };
  assert.deepEqual(parseInstallReceipt(receipt, "editor_host"), { schemaVersion: 2, targetMode: "editor_host", operationId: opA, inventoryRevision: "1", package: installed() });
  for (const action of ["catalog-license", "package-install"]) assertSnapshot({ ...fixture().snapshot(), operations: [{ operationId: opA, action, organizationId: orgA, status: "unknown", error: null }] });
});

test("malformed catalog and install data cannot establish installed status", () => {
  for (const mutation of [{ revision: 1 }, { revision: "01" }, { revision: "0" }, { kind: "script" }, { artifact_digest: "bad" }, { artifact_size: 1.5 }, { artifact_size: 1073741825 }, { package_id: "bad" }]) assert.throws(() => parseCatalogPage(page([{ ...release(), ...mutation }])), /account_protocol_invalid/);
  assert.throws(() => parseInventory(inventory([installed(), installed()]), "editor_host"), /account_protocol_invalid/);
  assert.throws(() => parseInventory({ schemaVersion: 1, revision: "0", packages: [] }, "editor_host"), /account_protocol_invalid/);
  assert.throws(() => parseInventory(inventory([], "0", "client_runtime"), "editor_host"), /account_protocol_invalid/);
  assert.throws(() => parseInstallReceipt({ schemaVersion: 2, targetMode: "client_runtime", operationId: opA, inventoryRevision: "1", package: installed() }, "editor_host"), /account_protocol_invalid/);
  assert.throws(() => parseInstallReceipt({ schemaVersion: 2, targetMode: "server_runtime", operationId: opA, inventoryRevision: "1", package: installed() }, "editor_host"), /account_protocol_invalid/);
  assert.equal(isInstalled(release(), inventory(), "editor_host"), false);
  assert.equal(isInstalled(release(), inventory([{ ...installed(), artifactDigest: "cd".repeat(32) }]), "editor_host"), false);
  assert.equal(isInstalled(release(), inventory([installed()]), "editor_host"), true);
  assert.equal(isInstalled(release(), inventory([installed()], "0", "client_runtime"), "editor_host"), false);
});

test("package operation summaries validate optional target mode while preserving legacy entries", () => {
  const source = fixture().snapshot();
  const summary = { operationId: opA, action: "package-install", organizationId: orgA, status: "unknown", error: "account_service_outcome_unknown" };
  assertSnapshot({ ...source, operations: [{ ...summary, targetMode: "editor_host" }] });
  assertSnapshot({ ...source, operations: [summary] });
  assert.throws(() => assertSnapshot({ ...source, operations: [{ ...summary, targetMode: "server_runtime" }] }), /account_protocol_invalid/);
  assert.throws(() => assertSnapshot({ ...source, operations: [{ ...summary, action: "catalog-license", targetMode: "client_runtime" }] }), /account_protocol_invalid/);
});

test("asset releases and plugin packages above native limits never dispatch installation", async () => {
  for (const change of [{ kind: "asset" }, { artifact_size: 16 * 1024 * 1024 + 1 }]) {
    const f = fixture((request, { snapshot }) => request.action === "catalog" ? snapshot(page([{ ...release(), ...change }])) : undefined);
    const selected = await ready(f);
    assert.equal(await f.client.installCatalogPackage(selected, "editor_host"), false);
    assert.equal(f.operationIds(), 0);
  }
  for (const code of ["account_package_service_unavailable", "account_package_trust_denied", "account_package_capacity_exceeded", "account_package_inventory_conflict", "account_package_target_migration_required", "account_package_policy_unconfigured", "account_package_target_unconfigured"]) assert.equal(accountErrorCode(code), code);
  assert.equal(accountErrorCode("package trust details: native-private"), "account_service_operation_failed");
});

test("catalog search forwards bounded query, resets cursor and rejects a stale search response", async () => {
  const old = deferred();
  const f = fixture((request, { snapshot }) => request.action === "catalog" ? request.query === "old" ? old.promise : snapshot(page([release(pkgB)])) : undefined);
  await f.client.start();
  const pending = f.client.loadCatalog("old");
  await f.client.loadCatalog("Material");
  old.resolve(f.snapshot(page([release()]))); await pending;
  assert.equal(f.client.getSnapshot().catalogQuery, "Material");
  assert.equal(f.client.getSnapshot().serviceCatalog.items[0].package_id, pkgB);
  assert.equal(f.requests.at(-1).query, "Material");
  assert.equal(f.requests.at(-1).backendEpoch, "catalog-ui");
  const count = f.requests.length;
  await f.client.loadCatalog("x".repeat(257));
  assert.equal(f.requests.length, count);
});

test("empty signed catalog pages retain pagination and cyclic cursor is rejected", async () => {
  const f = fixture((request, { snapshot }) => request.action === "catalog" ? snapshot(request.after ? page([release(pkgB)]) : page([], pkgA)) : undefined);
  await f.client.start(); await f.client.loadCatalog(); await f.client.loadCatalog(undefined, true);
  assert.equal(f.client.getSnapshot().serviceCatalog.items[0].package_id, pkgB);
  assert.equal(f.requests.at(-1).after, pkgA);
  const bad = fixture((request, { snapshot }) => request.action === "catalog" ? snapshot(page([], pkgA)) : undefined);
  await bad.client.start(); await bad.client.loadCatalog(); await bad.client.loadCatalog(undefined, true);
  assert.equal(bad.client.getSnapshot().serviceCatalog.error, "account_protocol_invalid");
});

test("organization changes discard old entitlement and inventory replies", async () => {
  const entitled = deferred(); const device = deferred();
  const f = fixture(request => request.organization === orgA && request.action === "catalog-entitlements" ? entitled.promise : request.organization === orgA && request.action === "package-inventory" ? device.promise : undefined);
  await f.client.start(); await f.client.selectOrganization(orgA);
  const loads = [f.client.loadCatalogEntitlements(), f.client.loadPackageInventory("editor_host")];
  await f.client.selectOrganization(orgB);
  entitled.resolve(f.snapshot(page([entitlement()]))); device.resolve(f.snapshot(inventory([installed()], "1")));
  await Promise.all(loads);
  assert.equal(f.client.getSnapshot().selectedOrganization, orgB);
  assert.deepEqual(f.client.getSnapshot().catalogEntitlements.items, []);
  assert.equal(f.client.getSnapshot().packageInventory.data, null);
});

test("logout fences a late package response and clears catalog cache", async () => {
  const delayed = deferred(); let hold = false;
  const f = fixture(request => hold && request.action === "package-inventory" ? delayed.promise : undefined);
  await ready(f); hold = true;
  const beforeLogout = f.snapshot(inventory([installed()], "1"));
  const request = f.client.loadPackageInventory("editor_host"); await f.client.authenticate("logout");
  delayed.resolve(beforeLogout); await request;
  assert.equal(f.client.getSnapshot().packageInventory.data, null);
  assert.deepEqual(f.client.getSnapshot().serviceCatalog.items, []);
  assert.equal(f.client.getSnapshot().snapshot.account.status, "signed-out");
});

test("entitlement pagination is required before missing access can become installable", async () => {
  const f = fixture((request, { snapshot }) => request.action === "catalog-entitlements" ? snapshot(request.after ? page([entitlement()]) : page([entitlement(pkgB)], "12")) : undefined);
  const selected = await ready(f);
  assert.equal(await f.client.installCatalogPackage(selected, "editor_host"), false);
  await f.client.loadCatalogEntitlements(true);
  assert.equal(f.requests.at(-1).after, "12");
  assert.equal(f.client.getSnapshot().catalogEntitlements.items.length, 2);
  assert.equal(f.operationIds(), 0);
});

test("only a qualified current owner or admin can confirm a release license", async () => {
  for (const role of ["owner", "admin", "member", "viewer"]) {
    const f = fixture((request, { snapshot, setOperation }) => request.action === "members" ? snapshot(page([{ issuer, subject: "owner", active: true, role }, { issuer: "https://other.test", subject: "owner", active: true, role: "owner" }])) : request.action === "catalog-license" ? setOperation(request, "committed", { packageId: pkgA, revision: release().revision }) : undefined);
    const selected = await ready(f);
    assert.equal(await f.client.acceptCatalogLicense(selected), ["owner", "admin"].includes(role));
    const action = f.requests.find(request => request.action === "catalog-license");
    if (action) assert.deepEqual({ organization: action.organization, expectedPolicyRevision: action.expectedPolicyRevision, revision: action.revision, licenseId: action.licenseId }, { organization: orgA, expectedPolicyRevision: "3", revision: "9007199254740993", licenseId: "MIT" });
  }
});

test("install forwards inventory revision and only inventory establishes installed state", async () => {
  let committed = false;
  let actual = null;
  const f = fixture((request, { snapshot, setOperation }) => {
    if (request.action === "package-install") { committed = true; actual = installed(request.operationId); return setOperation(request, "committed", { schemaVersion: 2, targetMode: request.targetMode, operationId: request.operationId, inventoryRevision: "4", package: actual }); }
    if (request.action === "package-inventory") return snapshot(inventory(committed ? [actual] : [], committed ? "4" : "3"));
  });
  const selected = await ready(f);
  assert.equal(await f.client.installCatalogPackage(selected, "editor_host"), true);
  const action = f.requests.find(request => request.action === "package-install");
  assert.equal(action.schemaVersion, 2);
  assert.equal(action.targetMode, "editor_host");
  assert.equal(action.expectedInventoryRevision, "3");
  assert.equal(action.revision, "9007199254740993");
  assert.equal(isInstalled(selected, f.client.getSnapshot().packageInventory.data, "editor_host"), true);
  assert.equal(await f.client.installCatalogPackage(selected, "editor_host"), false);
  assert.equal(f.operationIds(), 1);
});

test("package inventory and installation stay in the selected runtime target namespace", async () => {
  let committedMode = null;
  const f = fixture((request, { snapshot, setOperation }) => {
    if (request.action === "package-inventory") {
      const packages = request.targetMode === committedMode ? [installed()] : [];
      return snapshot(inventory(packages, packages.length ? "2" : "0", request.targetMode));
    }
    if (request.action === "package-install") {
      committedMode = request.targetMode;
      return setOperation(request, "committed", { schemaVersion: 2, targetMode: request.targetMode, operationId: request.operationId, inventoryRevision: "2", package: installed(request.operationId) });
    }
  });
  const selected = await ready(f);
  assert.equal(f.client.getSnapshot().packageInventory.data.targetMode, "editor_host");
  assert.equal(await f.client.installCatalogPackage(selected, "editor_host"), true);
  await f.client.loadPackageInventory("client_runtime");
  assert.equal(f.client.getSnapshot().packageInventory.targetMode, "client_runtime");
  assert.equal(f.client.getSnapshot().packageInventory.data.targetMode, "client_runtime");
  assert.equal(isInstalled(selected, f.client.getSnapshot().packageInventory.data, "client_runtime"), false);
  assert.equal(await f.client.installCatalogPackage(selected, "client_runtime"), true);
  const installs = f.requests.filter(request => request.action === "package-install");
  assert.deepEqual(installs.map(request => [request.schemaVersion, request.targetMode]), [[2, "editor_host"], [2, "client_runtime"]]);
  assert.equal(f.client.getSnapshot().packageInventory.data.targetMode, "client_runtime");
  assert.equal(isInstalled(selected, f.client.getSnapshot().packageInventory.data, "client_runtime"), true);
});

test("a stale target inventory response cannot replace the newer target and target errors wait for explicit retry", async () => {
  const older = deferred();
  let failClient = true;
  const f = fixture((request, { snapshot }) => {
    if (request.action !== "package-inventory") return undefined;
    if (request.targetMode === "editor_host") return older.promise;
    if (failClient) return snapshot(null, "account_package_service_unavailable");
    return snapshot(inventory([], "0", "client_runtime"));
  });
  await f.client.start(); await f.client.selectOrganization(orgA);
  const oldRequest = f.client.loadPackageInventory("editor_host");
  const newRequest = f.client.loadPackageInventory("client_runtime");
  await newRequest;
  assert.equal(f.client.getSnapshot().packageInventory.targetMode, "client_runtime");
  assert.equal(f.client.getSnapshot().packageInventory.error, "account_package_service_unavailable");
  failClient = false;
  await f.client.loadPackageInventory("client_runtime");
  assert.equal(f.client.getSnapshot().packageInventory.data.targetMode, "client_runtime");
  older.resolve(f.snapshot(inventory([installed()], "1", "editor_host")));
  await oldRequest;
  assert.equal(f.client.getSnapshot().packageInventory.data.targetMode, "client_runtime");
});

test("committed journal without inventory proof never fabricates an installation", async () => {
  const f = fixture((request, { setOperation }) => request.action === "package-install" ? setOperation(request, "committed") : undefined);
  const selected = await ready(f);
  assert.equal(await f.client.installCatalogPackage(selected, "editor_host"), true);
  assert.equal(isInstalled(selected, f.client.getSnapshot().packageInventory.data, "editor_host"), false);
});

test("a lost IPC install reply retries the exact request and operation ID", async () => {
  let attempts = 0;
  const f = fixture((request, { setOperation }) => {
    if (request.action === "package-install") { if (++attempts === 1) throw new Error("lost native reply"); return setOperation(request, "unknown"); }
  });
  const selected = await ready(f);
  assert.equal(await f.client.installCatalogPackage(selected, "editor_host"), false);
  const pending = f.client.getSnapshot().mutation;
  assert.equal(pending.status, "unknown");
  assert.equal(await f.client.installCatalogPackage(selected, "editor_host"), false);
  await f.client.retryMutation(pending.operationId);
  assert.equal(f.operationIds(), 1);
  const attemptsSent = f.requests.filter(request => request.action === "package-install");
  assert.deepEqual(attemptsSent[0], attemptsSent[1]);
  assert.equal(f.client.getSnapshot().snapshot.operations[0].status, "unknown");
});

test("retry rejection without a receipt cannot turn a lost install outcome into a definite failure", async () => {
  let attempts = 0;
  const f = fixture((request, { snapshot }) => {
    if (request.action === "package-install") {
      if (++attempts === 1) throw new Error("lost native reply");
      return snapshot(null, "account_package_service_unavailable");
    }
  });
  const selected = await ready(f);
  await f.client.installCatalogPackage(selected, "editor_host");
  await f.client.retryMutation(f.client.getSnapshot().mutation.operationId);
  assert.equal(f.client.getSnapshot().mutation.status, "unknown");
  assert.equal(await f.client.installCatalogPackage(selected, "editor_host"), false);
  assert.equal(f.operationIds(), 1);
  assert.deepEqual(f.requests.filter(request => request.action === "package-install")[0], f.requests.filter(request => request.action === "package-install")[1]);
});

for (const targetMode of ["editor_host", "client_runtime"]) test(`restart-restored unknown ${targetMode} install reconciles, retries and acknowledges the original journal entry`, async () => {
  let phase = "unknown";
  const initial = [{ operationId: opA, action: "package-install", targetMode, organizationId: orgA, status: "unknown", error: "account_service_outcome_unknown" }];
  const f = fixture((request, { snapshot, setOperation }) => {
    if (request.action === "reconcile" || request.action === "retry") { phase = request.action === "retry" ? "committed" : "unknown"; return setOperation(request, phase); }
    if (request.action === "package-inventory") return snapshot(inventory(phase === "committed" ? [installed()] : [], phase === "committed" ? "1" : "0", request.targetMode));
  }, initial);
  const selected = await ready(f, targetMode);
  assert.equal(await f.client.installCatalogPackage(selected, targetMode), false);
  assert.equal(await f.client.checkMutation(opA), false);
  assert.equal(f.client.getSnapshot().snapshot.operations[0].targetMode, targetMode);
  assert.equal(await f.client.retryMutation(opA), true);
  assert.equal(isInstalled(selected, f.client.getSnapshot().packageInventory.data, targetMode), true);
  assert.ok(f.requests.filter(request => request.action === "package-inventory").every(request => request.targetMode === targetMode));
  assert.deepEqual(f.requests.filter(request => request.action === "reconcile" || request.action === "retry").map(request => request.operationId), [opA, opA]);
  assert.equal(await f.client.acknowledgeOperation(opA), true);
  assert.deepEqual(f.client.getSnapshot().snapshot.operations, []);
  assert.equal(f.operationIds(), 0);
});

test("definite install failure requires refresh and a new user command", async () => {
  const f = fixture((request, { setOperation }) => request.action === "package-install" ? setOperation(request, "failed") : undefined);
  const selected = await ready(f);
  await f.client.installCatalogPackage(selected, "editor_host");
  const failed = f.client.getSnapshot().snapshot.operations[0];
  assert.equal(await f.client.retryMutation(failed.operationId), false);
  assert.equal(await f.client.installCatalogPackage(selected, "editor_host"), false);
  assert.equal(f.operationIds(), 1);
  await f.client.refreshOrganization(); await f.client.loadCatalogEntitlements(); await f.client.loadPackageInventory("editor_host");
  await f.client.installCatalogPackage(selected, "editor_host");
  assert.equal(f.operationIds(), 2);
});

for (const targetMode of ["editor_host", "client_runtime"]) test(`reconciliation parses the ${targetMode} service status/result envelope before refreshing inventory`, async () => {
  const initial = [{ operationId: opA, action: "package-install", targetMode, organizationId: orgA, status: "unknown", error: "account_service_outcome_unknown" }];
  let committed = false;
  const receipt = { schemaVersion: 2, targetMode, operationId: opA, inventoryRevision: "1", package: installed() };
  const f = fixture((request, { snapshot, setOperation }) => {
    if (request.action === "reconcile") { committed = true; return setOperation(request, "committed", { status: "committed", result: receipt }); }
    if (request.action === "package-inventory") return snapshot(inventory(committed ? [installed()] : [], committed ? "1" : "0", request.targetMode));
  }, initial);
  const selected = await ready(f, targetMode);
  assert.equal(await f.client.checkMutation(opA), true);
  assert.equal(isInstalled(selected, f.client.getSnapshot().packageInventory.data, targetMode), true);
  assert.equal(f.operationIds(), 0);
});

test("inventory rollback and malformed install receipt remain visible errors", async () => {
  let revision = "3";
  const f = fixture((request, { snapshot }) => request.action === "package-inventory" ? snapshot(inventory([], revision)) : undefined);
  await ready(f); revision = "2"; await f.client.loadPackageInventory("editor_host");
  assert.equal(f.client.getSnapshot().packageInventory.data.revision, "3");
  assert.equal(f.client.getSnapshot().packageInventory.error, "account_protocol_invalid");
  const malformed = fixture((request, { setOperation }) => request.action === "package-install" ? setOperation(request, "committed", { schemaVersion: 2, targetMode: "editor_host", operationId: request.operationId, inventoryRevision: "1", package: installed(request.operationId, pkgB) }) : undefined);
  const selected = await ready(malformed);
  assert.equal(await malformed.client.installCatalogPackage(selected, "editor_host"), false);
  assert.equal(isInstalled(selected, malformed.client.getSnapshot().packageInventory.data, "editor_host"), false);
  assert.equal(malformed.client.getSnapshot().error, "account_protocol_invalid");
  assert.equal(malformed.client.getSnapshot().snapshot.operations[0].status, "committed");
});

test("catalog and inventory data from a different native epoch cannot replace current state", async () => {
  const f = fixture((request, { snapshot }) => request.action === "catalog" ? { ...snapshot(page([release()])), backendEpoch: "other-native" } : request.action === "package-inventory" ? { ...snapshot(inventory([installed()], "1")), backendEpoch: "other-native" } : undefined);
  await f.client.start(); await f.client.selectOrganization(orgA); await f.client.loadCatalog(); await f.client.loadPackageInventory("editor_host");
  assert.deepEqual(f.client.getSnapshot().serviceCatalog.items, []);
  assert.equal(f.client.getSnapshot().serviceCatalog.error, "hub_state_epoch_stale");
  assert.equal(f.client.getSnapshot().packageInventory.data, null);
  assert.equal(f.client.getSnapshot().packageInventory.error, "hub_state_epoch_stale");
});
