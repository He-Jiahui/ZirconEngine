import assert from "node:assert/strict";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const server = await createServer({ configFile: false, root: fileURLToPath(new URL("../", import.meta.url)), optimizeDeps: { noDiscovery: true, include: [] }, server: { middlewareMode: true }, appType: "custom" });
const { parseCloudHead } = await server.ssrLoadModule("/src/account/cloud_head.ts");
const { AccountController } = await server.ssrLoadModule("/src/account/controller.ts");
await server.close();

const organization = "00000000-0000-4000-8000-000000000001";
const project = "00000000-0000-4000-8000-000000000002";
const hex = "a".repeat(64);
const cloudHead = () => ({
  organizationId: organization, projectId: project, baseRevision: "0", revision: "1",
  manifestDigest: hex, createdAt: 1, createdBy: hex,
  manifest: { schemaVersion: 1, engine: "Zircon", packageLockDigest: hex, ignorePolicy: "zircon-project-v1", sourceRevision: null,
    files: [{ path: "Content/scene.zui", digest: hex, bytes: 5 }] },
});
const account = (status = "signed-in", generation = "7", data = null) => ({
  backendEpoch: "hub-a", account: { configured: true, status, issuer: status === "signed-in" ? "https://issuer.test" : null,
    subject: status === "signed-in" ? "alice" : null, displayName: null, generation, error: null },
  data, error: null, operations: [], operationsRevision: "0", operationsError: null,
});
const page = items => ({ items, nextCursor: null });
function deferred() { let resolve; const promise = new Promise(done => { resolve = done; }); return { promise, resolve }; }

test("cloud head accepts empty and exact project snapshot but rejects invalid identity, path and budgets", () => {
  assert.equal(parseCloudHead(null, organization, project), null);
  assert.deepEqual(parseCloudHead(cloudHead(), organization, project), cloudHead());
  const baseline = cloudHead();
  for (const bad of [
    { ...baseline, organizationId: project },
    { ...baseline, projectId: organization },
    { ...baseline, revision: "0" },
    { ...baseline, manifestDigest: "A".repeat(64) },
    { ...baseline, createdAt: -1 },
    { ...baseline, unexpected: true },
    { ...baseline, manifest: { ...baseline.manifest, files: [{ path: "../secret", digest: hex, bytes: 5 }] } },
    { ...baseline, manifest: { ...baseline.manifest, files: [{ path: "Content/scene.zui", digest: hex, bytes: 5 }, { path: "content/SCENE.ZUI", digest: hex, bytes: 5 }] } },
    { ...baseline, manifest: { ...baseline.manifest, files: [{ path: "Content/scene.zui", digest: hex, bytes: 16 * 1024 * 1024 + 1 }] } },
  ]) assert.throws(() => parseCloudHead(bad, organization, project), /account_protocol_invalid/);
});

test("cloud head query is bound to selected project and discards a late reply after sign-out", async () => {
  const reply = deferred();
  const actions = [];
  const client = new AccountController("hub-a", { load: async () => account(), operationId: () => project,
    dispatch: request => {
      actions.push(request);
      if (request.action === "organizations") return Promise.resolve(account("signed-in", "7", page([{ id: organization, name: "Team", policyRevision: "1" }])));
      if (request.action === "invitations" || request.action === "members") return Promise.resolve(account("signed-in", "7", page([])));
      if (request.action === "projects") return Promise.resolve(account("signed-in", "7", page([{ id: project, name: "Demo" }])));
      if (request.action === "cloud-head") return reply.promise;
      if (request.action === "logout") return Promise.resolve(account("signed-out", "8"));
      throw new Error(`unexpected ${request.action}`);
    },
  });
  await client.start();
  await client.selectOrganization(organization);
  const pending = client.loadCloudHead(project);
  assert.deepEqual(actions.at(-1), { action: "cloud-head", backendEpoch: "hub-a", generation: "7", organization, project });
  assert.equal(client.getSnapshot().cloudHead?.loading, true);
  await client.authenticate("logout");
  reply.resolve(account("signed-in", "7", cloudHead()));
  await pending;
  assert.equal(client.getSnapshot().cloudHead, null);
});

test("cloud head distinguishes an empty project from a verified head and fails closed on a wrong project", async () => {
  let response = null;
  const client = new AccountController("hub-a", { load: async () => account(), operationId: () => project,
    dispatch: request => {
      if (request.action === "organizations") return Promise.resolve(account("signed-in", "7", page([{ id: organization, name: "Team", policyRevision: "1" }])));
      if (request.action === "invitations" || request.action === "members") return Promise.resolve(account("signed-in", "7", page([])));
      if (request.action === "projects") return Promise.resolve(account("signed-in", "7", page([{ id: project, name: "Demo" }])));
      if (request.action === "cloud-head") return Promise.resolve(account("signed-in", "7", response));
      throw new Error(`unexpected ${request.action}`);
    },
  });
  await client.start();
  await client.selectOrganization(organization);
  await client.loadCloudHead(project);
  assert.deepEqual(client.getSnapshot().cloudHead, { organization, project, snapshot: null, loading: false, loaded: true, error: null });
  response = cloudHead();
  await client.loadCloudHead(project);
  assert.equal(client.getSnapshot().cloudHead?.snapshot?.revision, "1");
  response = { ...cloudHead(), projectId: organization };
  await client.loadCloudHead(project);
  assert.deepEqual(client.getSnapshot().cloudHead, { organization, project, snapshot: null, loading: false, loaded: false, error: "account_protocol_invalid" });
  await client.loadCloudHead(organization);
  assert.equal(client.getSnapshot().cloudHead?.project, project);
});
