import assert from "node:assert/strict";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const server = await createServer({ configFile: false, root: fileURLToPath(new URL("../", import.meta.url)), cacheDir: `E:/cargo-targets/zircon-engine/hub-web-validation/cloud-sync-controller-${process.pid}/vite-cache`, optimizeDeps: { noDiscovery: true, include: [] }, server: { middlewareMode: true }, appType: "custom" });
const { AccountController } = await server.ssrLoadModule("/src/account/controller.ts");
await server.close();

const organization = "00000000-0000-4000-8000-000000000001";
const project = "00000000-0000-4000-8000-000000000002";
const operationId = "00000000-0000-4000-8000-000000000003";
const digest = "a".repeat(64);
const summary = (status, error = status === "unknown" ? "account_service_outcome_unknown" : null) => ({
  operationId,
  status,
  action: "cloud-commit",
  organizationId: organization,
  error,
});
const account = (data = null, operations = [], error = null, revision = "0") => ({
  backendEpoch: "cloud-sync-controller",
  account: { configured: true, status: "signed-in", issuer: "https://issuer.test", subject: "alice", displayName: null, generation: "7", error: null },
  data,
  error,
  operations,
  operationsRevision: revision,
  operationsError: null,
});
const page = items => ({ items, nextCursor: null });
const head = {
  organizationId: organization,
  projectId: project,
  baseRevision: "3",
  revision: "4",
  manifestDigest: digest,
  createdAt: 1,
  createdBy: digest,
  manifest: { schemaVersion: 1, engine: "Zircon", packageLockDigest: digest, ignorePolicy: "zircon-project-v1", sourceRevision: null, files: [] },
};

test("cloud push keeps the native operation identity for reconciliation and fences another write", async () => {
  const requests = [];
  let operations = [];
  let operationsRevision = "0";
  const reply = (data = null, error = null) => account(data, operations, error, operationsRevision);
  const controller = new AccountController("cloud-sync-controller", {
    load: async () => reply(),
    operationId: () => operationId,
    dispatch: async request => {
      requests.push(request);
      if (request.action === "organizations") return reply(page([{ id: organization, name: "Team", policyRevision: "1" }]));
      if (request.action === "invitations" || request.action === "members") return reply(page([]));
      if (request.action === "projects") return reply(page([{ id: project, name: "Demo" }]));
      if (request.action === "cloud-binding") return reply({ localProjectGuid: project, organizationId: organization, projectId: project });
      if (request.action === "cloud-head") return reply(head);
      if (request.action === "cloud-push") {
        operations = [summary("unknown")];
        operationsRevision = "1";
        return reply(null, "account_service_outcome_unknown");
      }
      if (request.action === "reconcile") {
        operations = [summary("committed")];
        operationsRevision = "2";
        return reply({ status: "committed", result: { status: "committed", snapshot: { organizationId: organization, projectId: project, baseRevision: "4", revision: "5", manifestDigest: digest } } });
      }
      throw new Error(`unexpected action ${request.action}`);
    },
  });

  await controller.start();
  await controller.selectOrganization(organization);
  await controller.loadCloudBinding();
  await controller.loadCloudHead(project);
  assert.equal(await controller.pushCloudSnapshot(), false);
  assert.deepEqual(requests.at(-1), {
    action: "cloud-push",
    backendEpoch: "cloud-sync-controller",
    generation: "7",
    operationId,
    baseRevision: "4",
  });
  assert.equal(controller.getSnapshot().snapshot.operations[0].operationId, operationId);
  assert.equal(controller.getSnapshot().snapshot.operations[0].status, "unknown");
  assert.equal(controller.getSnapshot().cloudSync?.status, "unknown");
  assert.equal(await controller.pushCloudSnapshot(), false);
  assert.equal(requests.filter(request => request.action === "cloud-push").length, 1);

  assert.equal(await controller.checkMutation(operationId), true);
  assert.equal(requests.at(-1).action, "reconcile");
  assert.equal(requests.at(-1).operationId, operationId);
  assert.deepEqual(controller.getSnapshot().snapshot.operations, [summary("committed")]);
});

test("cloud pull refuses stale UI revisions and retains a staged identity for safe apply", async () => {
  const requests = [];
  const controller = new AccountController("cloud-sync-controller", {
    load: async () => account(),
    operationId: () => operationId,
    dispatch: async request => {
      requests.push(request);
      if (request.action === "organizations") return account(page([{ id: organization, name: "Team", policyRevision: "1" }]));
      if (request.action === "invitations" || request.action === "members") return account(page([]));
      if (request.action === "projects") return account(page([{ id: project, name: "Demo" }]));
      if (request.action === "cloud-binding") return account({ localProjectGuid: project, organizationId: organization, projectId: project });
      if (request.action === "cloud-head") return account(head);
      if (request.action === "cloud-stage-download") return account({ status: "staged", stageId: operationId, revision: "4", manifestDigest: digest, fileCount: 0, totalBytes: "0" });
      if (request.action === "cloud-apply-download") return account({ status: "applied", stageId: operationId, revision: "4", appliedFiles: 0, unchangedFiles: 0 });
      throw new Error(`unexpected action ${request.action}`);
    },
  });
  await controller.start();
  await controller.selectOrganization(organization);
  await controller.loadCloudBinding();
  await controller.loadCloudHead(project);

  assert.equal(await controller.stageCloudDownload("3"), false);
  assert.equal(requests.some(request => request.action === "cloud-stage-download"), false);
  assert.equal(await controller.stageCloudDownload("4"), true, JSON.stringify(controller.getSnapshot()));
  assert.deepEqual(requests.at(-1), { action: "cloud-stage-download", backendEpoch: "cloud-sync-controller", generation: "7", expectedRevision: "4" });
  assert.equal(controller.getSnapshot().cloudSync?.status === "staged" && controller.getSnapshot().cloudSync.stage.stageId, operationId);
  assert.equal(await controller.applyCloudDownload(operationId, "4"), true);
  assert.deepEqual(requests.filter(request => request.action === "cloud-apply-download").at(-1), {
    action: "cloud-apply-download",
    backendEpoch: "cloud-sync-controller",
    generation: "7",
    stageId: operationId,
    expectedRevision: "4",
  });
});

test("cloud resume reuses the original operation ID and sends no Web-owned snapshot", async () => {
  const requests = [];
  let operations = [];
  let operationsRevision = "0";
  const reply = (data = null, error = null) => account(data, operations, error, operationsRevision);
  const controller = new AccountController("cloud-sync-controller", {
    load: async () => reply(), operationId: () => operationId,
    dispatch: async request => {
      requests.push(request);
      if (request.action === "organizations") return reply(page([{ id: organization, name: "Team", policyRevision: "1" }]));
      if (request.action === "invitations" || request.action === "members") return reply(page([]));
      if (request.action === "projects") return reply(page([{ id: project, name: "Demo" }]));
      if (request.action === "cloud-binding") return reply({ localProjectGuid: project, organizationId: organization, projectId: project });
      if (request.action === "cloud-head") return reply(head);
      if (request.action === "cloud-push") {
        operations = [summary("unknown")]; operationsRevision = "1";
        return reply(null, "account_service_outcome_unknown");
      }
      if (request.action === "cloud-resume-push") {
        operations = [summary("committed")]; operationsRevision = "2";
        return reply({ status: "committed", result: { status: "committed", snapshot: {
          organizationId: organization, projectId: project, baseRevision: "4", revision: "5", manifestDigest: digest,
        } } });
      }
      throw new Error(`unexpected action ${request.action}`);
    },
  });
  await controller.start();
  await controller.selectOrganization(organization);
  await controller.loadCloudBinding();
  await controller.loadCloudHead(project);

  assert.equal(await controller.pushCloudSnapshot(), false);
  assert.equal(await controller.retryCloudSnapshot(operationId), true);
  const resume = requests.at(-1);
  assert.deepEqual(resume, {
    action: "cloud-resume-push", backendEpoch: "cloud-sync-controller", generation: "7", operationId,
  });
  assert.equal(Object.keys(resume).some(key => ["path", "manifest", "files", "bytes"].includes(key)), false);
  assert.equal(controller.getSnapshot().cloudSync?.status, "committed");
});

test("failed staged apply can be retried with the same stage and revision", async () => {
  const requests = [];
  let failFirstApply = true;
  const reply = (data = null, error = null) => account(data, [], error, "1");
  const controller = new AccountController("cloud-sync-controller", {
    load: async () => reply(), operationId: () => operationId,
    dispatch: async request => {
      requests.push(request);
      if (request.action === "organizations") return reply(page([{ id: organization, name: "Team", policyRevision: "1" }]));
      if (request.action === "invitations" || request.action === "members") return reply(page([]));
      if (request.action === "projects") return reply(page([{ id: project, name: "Demo" }]));
      if (request.action === "cloud-binding") return reply({ localProjectGuid: project, organizationId: organization, projectId: project });
      if (request.action === "cloud-head") return reply(head);
      if (request.action === "cloud-stage-download") return reply({ status: "staged", stageId: operationId, revision: "4", manifestDigest: digest, fileCount: 0, totalBytes: "0" });
      if (request.action === "cloud-apply-download" && failFirstApply) {
        failFirstApply = false;
        return reply(null, "hub_cloud_sync_project_busy");
      }
      if (request.action === "cloud-apply-download") return reply({ status: "applied", stageId: operationId, revision: "4", appliedFiles: 1, unchangedFiles: 0 });
      throw new Error(`unexpected action ${request.action}`);
    },
  });
  await controller.start();
  await controller.selectOrganization(organization);
  await controller.loadCloudBinding();
  await controller.loadCloudHead(project);
  assert.equal(await controller.stageCloudDownload("4"), true);
  assert.equal(await controller.applyCloudDownload(operationId, "4"), false);
  assert.equal(controller.getSnapshot().cloudSync?.status, "apply-failed");
  assert.equal(await controller.applyCloudDownload(operationId, "4"), true);
  assert.deepEqual(requests.filter(request => request.action === "cloud-apply-download").map(request => [request.stageId, request.expectedRevision]), [
    [operationId, "4"], [operationId, "4"],
  ]);
});

test("download retry repeats only the revision so native staging can recover after restart", async () => {
  const requests = [];
  let failed = true;
  const controller = new AccountController("cloud-sync-controller", {
    load: async () => account(), operationId: () => operationId,
    dispatch: async request => {
      requests.push(request);
      if (request.action === "organizations") return account(page([{ id: organization, name: "Team", policyRevision: "1" }]));
      if (request.action === "invitations" || request.action === "members") return account(page([]));
      if (request.action === "projects") return account(page([{ id: project, name: "Demo" }]));
      if (request.action === "cloud-binding") return account({ localProjectGuid: project, organizationId: organization, projectId: project });
      if (request.action === "cloud-head") return account(head);
      if (request.action === "cloud-stage-download" && failed) {
        failed = false;
        return account(null, [], "hub_cloud_sync_stage_invalid");
      }
      if (request.action === "cloud-stage-download") return account({ status: "staged", stageId: operationId, revision: "4", manifestDigest: digest, fileCount: 0, totalBytes: "0" });
      throw new Error(`unexpected action ${request.action}`);
    },
  });
  await controller.start();
  await controller.selectOrganization(organization);
  await controller.loadCloudBinding();
  await controller.loadCloudHead(project);
  assert.equal(await controller.stageCloudDownload("4"), false);
  assert.equal(controller.getSnapshot().cloudSync?.status, "download-failed");
  assert.equal(await controller.stageCloudDownload("4"), true);
  const downloads = requests.filter(request => request.action === "cloud-stage-download");
  assert.deepEqual(downloads, [
    { action: "cloud-stage-download", backendEpoch: "cloud-sync-controller", generation: "7", expectedRevision: "4" },
    { action: "cloud-stage-download", backendEpoch: "cloud-sync-controller", generation: "7", expectedRevision: "4" },
  ]);
});

test("staged cloud data can be explicitly discarded without sending a path or manifest", async () => {
  const requests = [];
  const controller = new AccountController("cloud-sync-controller", {
    load: async () => account(), operationId: () => operationId,
    dispatch: async request => {
      requests.push(request);
      if (request.action === "organizations") return account(page([{ id: organization, name: "Team", policyRevision: "1" }]));
      if (request.action === "invitations" || request.action === "members") return account(page([]));
      if (request.action === "projects") return account(page([{ id: project, name: "Demo" }]));
      if (request.action === "cloud-binding") return account({ localProjectGuid: project, organizationId: organization, projectId: project });
      if (request.action === "cloud-head") return account(head);
      if (request.action === "cloud-stage-download") return account({ status: "staged", stageId: operationId, revision: "4", manifestDigest: digest, fileCount: 0, totalBytes: "0" });
      if (request.action === "cloud-discard-download") return account({ status: "discarded", stageId: operationId, revision: "4" });
      throw new Error(`unexpected action ${request.action}`);
    },
  });
  await controller.start();
  await controller.selectOrganization(organization);
  await controller.loadCloudBinding();
  await controller.loadCloudHead(project);
  assert.equal(await controller.stageCloudDownload("4"), true);
  assert.equal(await controller.discardCloudDownload("4"), true);
  const discard = requests.at(-1);
  assert.deepEqual(discard, {
    action: "cloud-discard-download", backendEpoch: "cloud-sync-controller", generation: "7", expectedRevision: "4",
  });
  assert.equal(Object.keys(discard).some(key => ["path", "stageId", "manifest", "files", "bytes"].includes(key)), false);
  assert.equal(controller.getSnapshot().cloudSync?.status, "discarded");
});

test("an unknown upload cannot resume against a different native project binding", async () => {
  const secondProject = "00000000-0000-4000-8000-000000000004";
  const requests = [];
  let operations = [];
  let bindingProject = project;
  const reply = (data = null, error = null) => account(data, operations, error, operations.length ? "1" : "0");
  const controller = new AccountController("cloud-sync-controller", {
    load: async () => reply(), operationId: () => operationId,
    dispatch: async request => {
      requests.push(request);
      if (request.action === "organizations") return reply(page([{ id: organization, name: "Team", policyRevision: "1" }]));
      if (request.action === "invitations" || request.action === "members") return reply(page([]));
      if (request.action === "projects") return reply(page([{ id: project, name: "First" }, { id: secondProject, name: "Second" }]));
      if (request.action === "cloud-binding") return reply({ localProjectGuid: bindingProject, organizationId: organization, projectId: bindingProject === project ? project : secondProject });
      if (request.action === "cloud-head") return reply({ ...head, projectId: request.project });
      if (request.action === "cloud-push") {
        operations = [summary("unknown")];
        return reply(null, "account_service_outcome_unknown");
      }
      if (request.action === "reconcile") return reply({ status: "unknown" });
      throw new Error(`unexpected action ${request.action}`);
    },
  });
  await controller.start();
  await controller.selectOrganization(organization);
  await controller.loadCloudBinding();
  await controller.loadCloudHead(project);
  assert.equal(await controller.pushCloudSnapshot(), false);

  bindingProject = secondProject;
  await controller.loadCloudBinding();
  await controller.loadCloudHead(secondProject);
  assert.equal(await controller.retryCloudSnapshot(operationId), false);
  assert.equal(requests.some(request => request.action === "cloud-resume-push"), false);
  assert.equal(await controller.checkMutation(operationId), false);
  assert.equal(requests.at(-1).action, "reconcile");
});
