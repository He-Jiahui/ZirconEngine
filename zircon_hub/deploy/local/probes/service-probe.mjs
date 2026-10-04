import { createHash, generateKeyPairSync, randomUUID, sign } from "node:crypto";
import { mkdir, readFile, readdir, writeFile, rename } from "node:fs/promises";
import { spawn } from "node:child_process";
import { createRequire } from "node:module";
import { request as httpRequest } from "node:http";
import { once } from "node:events";
import { isDeepStrictEqual, parseArgs } from "node:util";
import path from "node:path";
import { OidcProbeClient, request } from "./oidc-client.mjs";

const { values } = parseArgs({ options: {
  runtime: { type: "string" }, executable: { type: "string" }, "build-job": { type: "string" },
  "input-hash": { type: "string" }, "executable-hash": { type: "string" },
  "runtime-dll-directory": { type: "string" },
  "playwright-root": { type: "string" }, chromium: { type: "string" },
} });
for (const required of ["runtime", "executable", "build-job", "input-hash", "executable-hash", "playwright-root", "chromium"]) {
  if (!values[required]) throw new Error(`Missing --${required}`);
}
const digest = bytes => createHash("sha256").update(bytes).digest("hex");
const runId = randomUUID();
const runDirectory = path.join(values.runtime, `service-run-${runId}`);
const privateDirectory = path.join(values.runtime, "private", `service-probe-${runId}`);
await mkdir(runDirectory);
await mkdir(privateDirectory);
const evidence = {
  schemaVersion: 1, runId, startedAt: new Date().toISOString(), status: "running", checks: {},
  build: { jobId: values["build-job"], inputManifestHash: values["input-hash"], executable: values.executable, executableSha256: digest(await readFile(values.executable)) },
  processes: [], nativeServiceExecuted: true, nativeBrokerExecuted: false, tauriExecuted: false,
  installationExecuted: false, installationBoundary: "The service currently returns authorized release metadata; package installation is a separate desktop consumer.",
};
let stage = "executable admission";
function check(name, condition) {
  stage = name;
  evidence.checks[name] = Boolean(condition);
  if (!condition) throw new Error(`Probe check failed: ${name}`);
}
check("managedExecutableDigestMatches", evidence.build.executableSha256 === values["executable-hash"]);
const accountConfig = JSON.parse(await readFile(path.join(values.runtime, "private/hub-state/config/account.json"), "utf8"));
const serviceUrl = new URL(accountConfig.service_url);
check("loopbackServiceAndIssuer", serviceUrl.hostname === "127.0.0.1" && new URL(accountConfig.issuer).hostname === "127.0.0.1");
evidence.endpoints = { service: serviceUrl.origin, issuer: accountConfig.issuer, callbackPort: accountConfig.callback_port };
const sourceConfigPath = path.join(values.runtime, "private/hub-state/config/service.toml");
const sourceConfig = await readFile(sourceConfigPath, "utf8");
const sourceConfigHash = digest(sourceConfig);
check("existingConfigHasNoCatalogOverride", !/^\s*catalog_policy_file\s*=/m.test(sourceConfig));
const policyPath = path.join(privateDirectory, "catalog-policy.json");
const configPath = path.join(privateDirectory, "service.toml");
await writeFile(configPath, `catalog_policy_file = ${JSON.stringify(policyPath)}\n${sourceConfig}`, { flag: "wx" });
const require = createRequire(path.join(values["playwright-root"], "package.json"));
const { chromium } = require("playwright");
const browser = await chromium.launch({ executablePath: values.chromium, headless: true });
const oidc = new OidcProbeClient(accountConfig.issuer, accountConfig.client_id, accountConfig.callback_port, browser);
let service;
const serviceEnvironment = { ...process.env };
if (values["runtime-dll-directory"]) {
  const directory = path.resolve(values["runtime-dll-directory"]);
  const runtimeDlls = (await readdir(directory)).filter(name => /^std-[a-f0-9]+\.dll$/.test(name));
  check("developmentRuntimeDllAvailable", runtimeDlls.length > 0);
  evidence.build.runtimeDlls = await Promise.all(runtimeDlls.map(async name => ({ path: path.join(directory, name), sha256: digest(await readFile(path.join(directory, name))) })));
  const pathKey = Object.keys(serviceEnvironment).find(key => key.toLowerCase() === "path") ?? "PATH";
  serviceEnvironment[pathKey] = `${directory}${path.delimiter}${serviceEnvironment[pathKey] ?? ""}`;
}

async function startService() {
  stage = "service process start";
  if (digest(await readFile(values.executable)) !== evidence.build.executableSha256) throw new Error("Managed executable changed");
  const child = spawn(values.executable, [configPath], { windowsHide: true, env: serviceEnvironment, stdio: ["ignore", "pipe", "pipe"] });
  const record = { pid: child.pid, executableSha256: evidence.build.executableSha256, startedAt: new Date().toISOString(), port: Number(serviceUrl.port), configSha256: digest(await readFile(configPath)) };
  evidence.processes.push(record);
  const output = [];
  child.stdout.on("data", bytes => output.push(bytes));
  child.stderr.on("data", bytes => output.push(bytes));
  const exited = once(child, "exit").then(([code, signal]) => {
    record.exitCode = code; record.signal = signal; record.exitedAt = new Date().toISOString();
    record.outputSha256 = digest(Buffer.concat(output));
    const codeOnly = Buffer.concat(output).toString("utf8").match(/Hub service: [a-z_]+/g);
    if (codeOnly) record.diagnostics = codeOnly;
  });
  service = { child, record, exited };
  for (let attempt = 0; attempt < 100; attempt++) {
    if (child.exitCode !== null) throw new Error("Service exited before readiness");
    const health = await request(`${serviceUrl}health`).catch(() => null);
    if (health?.status === 200 && health.data.status === "alive" && health.data.protocolVersion === 1) {
      record.readyAt = new Date().toISOString();
      evidence.nativeServiceReady = true;
      return;
    }
    await new Promise(resolve => setTimeout(resolve, 300));
  }
  throw new Error("Service readiness timed out");
}

async function terminateService(reason) {
  if (!service) return;
  service.record.stopMode = reason;
  if (service.child.exitCode === null) service.child.kill();
  await service.exited;
  service = null;
}

async function api(session, route, expected = 200, method = "GET", body, extraHeaders = {}) {
  stage = `${method} ${route}`;
  const response = await request(new URL(route, serviceUrl), {
    method, headers: { ...(session ? { authorization: `Bearer ${session.accessToken}` } : {}), ...(body === undefined ? {} : { "content-type": "application/json" }), ...extraHeaders },
    ...(body === undefined ? {} : { body: JSON.stringify(body) }),
  });
  if (response.status !== expected) {
    evidence.httpFailure = { stage, expected, observed: response.status, error: typeof response.data?.error === "string" ? response.data.error : null };
    throw new Error("Unexpected service HTTP status");
  }
  return response.data;
}

async function mutate(session, organization, revision, mutation, expected = 200, operationId = randomUUID()) {
  const payload = { operationId, expectedPolicyRevision: revision, mutation };
  return { payload, result: await api(session, `/v1/organizations/${organization}/mutations`, expected, "POST", payload) };
}

async function publishPolicy(policy) {
  const temporary = `${policyPath}.${randomUUID()}.tmp`;
  await writeFile(temporary, JSON.stringify(policy), { flag: "wx" });
  await rename(temporary, policyPath);
}

try {
  stage = "Keycloak readiness";
  const health = await request("http://127.0.0.1:9000/health/ready");
  check("existingKeycloakReady", health.status === 200 && health.data.status === "UP");
  await oidc.initialize(JSON.parse(await readFile(path.join(values.runtime, "private/secrets/credentials.json"), "utf8")));
  const aliceUser = await oidc.createUser("owner");
  const bobUser = await oidc.createUser("member");
  const outsiderUser = await oidc.createUser("outsider");
  const alice = await oidc.login(aliceUser, path.join(runDirectory, "keycloak-login-1280.png"));
  const bob = await oidc.login(bobUser);
  const outsider = await oidc.login(outsiderUser);
  check("threeRealBrowserPkceLogins", Boolean(alice.accessToken && bob.accessToken && outsider.accessToken));
  const { publicKey, privateKey } = generateKeyPairSync("rsa", { modulusLength: 2048 });
  const kid = `probe-${runId}`;
  const policy = { issuer: "zircon-catalog-probe", audience: "zircon-hub-probe", keys: { keys: [{ ...publicKey.export({ format: "jwk" }), kid, alg: "RS256", use: "sig" }] }, publishers: { [kid]: { issuer: accountConfig.issuer, subject: aliceUser.subject } } };
  await publishPolicy(policy);
  await startService();
  await api(null, "/v1/organizations", 401);
  await api(alice, "/v1/organizations", 403, "GET", undefined, { origin: "http://127.0.0.1:1420" });
  check("nativeAuthenticationAndBrowserOriginGate", true);

  const creation = { operationId: randomUUID(), name: `Astra team ${runId}` };
  const organization = await api(alice, "/v1/organizations", 200, "POST", creation);
  const otherOrganization = await api(outsider, "/v1/organizations", 200, "POST", { operationId: randomUUID(), name: `Astra isolated ${runId}` });
  check("organizationCreateReplayIsStable", isDeepStrictEqual(await api(alice, "/v1/organizations", 200, "POST", creation), organization));
  check("qualifiedReceiptHiddenFromOtherAccount", (await api(bob, `/v1/operations/${creation.operationId}`)).status === "unknown");
  await api(outsider, `/v1/organizations/${organization.id}/members`, 403);
  await api(alice, `/v1/organizations/${otherOrganization.id}/members`, 403);
  check("crossTenantMembersDenied", true);
  let revision = organization.policyRevision;
  const project = await mutate(alice, organization.id, revision, { action: "create-project", name: "Recovery project" });
  revision = project.result.policyRevision;
  const projectId = project.result.resourceId;
  const stale = await mutate(alice, organization.id, "1", { action: "create-project", name: "Rejected stale project" }, 409);
  check("stalePolicyMakesNoProjectWrite", stale.result.error === "policy_conflict" && (await api(alice, `/v1/organizations/${organization.id}/projects`)).items.length === 1);
  const invitation = await mutate(alice, organization.id, revision, { action: "invite", issuer: accountConfig.issuer, subject: bobUser.subject, role: "member", expires_at: Math.floor(Date.now() / 1000) + 3600 });
  revision = invitation.result.policyRevision;
  const inbox = await api(bob, "/v1/invitations");
  check("qualifiedInvitationDiscoverable", inbox.items.some(item => item.id === invitation.result.resourceId && item.policyRevision === revision));
  await mutate(outsider, organization.id, revision, { action: "accept-invite", invitation_id: invitation.result.resourceId }, 403);
  const acceptance = await mutate(bob, organization.id, revision, { action: "accept-invite", invitation_id: invitation.result.resourceId });
  revision = acceptance.result.policyRevision;
  check("invitationAcceptanceReplayIsStable", isDeepStrictEqual(await api(bob, `/v1/organizations/${organization.id}/mutations`, 200, "POST", acceptance.payload), acceptance.result));
  await api(bob, `/v1/organizations/${organization.id}/invitations`, 403);
  await mutate(bob, organization.id, revision, { action: "set-member", issuer: accountConfig.issuer, subject: aliceUser.subject, role: "viewer", active: true }, 403);
  const viewer = await mutate(alice, organization.id, revision, { action: "set-member", issuer: accountConfig.issuer, subject: bobUser.subject, role: "viewer", active: true });
  revision = viewer.result.policyRevision;
  await mutate(bob, organization.id, revision, { action: "create-project", name: "Rejected viewer project" }, 403);
  check("memberAndViewerCannotAdministerOrWrite", true);
  const admin = await mutate(alice, organization.id, revision, { action: "set-member", issuer: accountConfig.issuer, subject: bobUser.subject, role: "admin", active: true });
  revision = admin.result.policyRevision;
  const issued = await mutate(bob, organization.id, revision, { action: "invite", issuer: accountConfig.issuer, subject: outsiderUser.subject, role: "viewer", expires_at: Math.floor(Date.now() / 1000) + 3600 });
  revision = issued.result.policyRevision;
  const issuedPage = await api(bob, `/v1/organizations/${organization.id}/invitations?limit=1`);
  const issuedMore = await api(bob, `/v1/organizations/${organization.id}/invitations?limit=1&after=${issuedPage.nextCursor}`);
  check("issuedInvitationsPageAcrossTerminalAndPending", issuedPage.items.length === 1 && issuedMore.items.length === 1 && [...issuedPage.items, ...issuedMore.items].some(item => item.id === issued.result.resourceId && item.status === "pending" && item.targetSubject === outsiderUser.subject));
  const revoked = await mutate(alice, organization.id, revision, { action: "revoke-invite", invitation_id: issued.result.resourceId });
  revision = revoked.result.policyRevision;
  await mutate(outsider, organization.id, revision, { action: "accept-invite", invitation_id: issued.result.resourceId }, 403);
  check("revokedInvitationCannotBeAccepted", (await api(alice, `/v1/organizations/${organization.id}/invitations`)).items.some(item => item.id === issued.result.resourceId && item.status === "revoked"));
  const transfer = await mutate(alice, organization.id, revision, { action: "transfer-ownership", issuer: accountConfig.issuer, subject: bobUser.subject });
  revision = transfer.result.policyRevision;
  const members = (await api(bob, `/v1/organizations/${organization.id}/members`)).items;
  check("ownershipTransferUpdatesBothMembers", members.find(item => item.subject === bobUser.subject).role === "owner" && members.find(item => item.subject === aliceUser.subject).role === "admin");
  await mutate(alice, organization.id, revision, { action: "set-member", issuer: accountConfig.issuer, subject: bobUser.subject, role: "viewer", active: true }, 403);
  const disabled = await mutate(bob, organization.id, revision, { action: "set-member", issuer: accountConfig.issuer, subject: aliceUser.subject, role: "admin", active: false });
  revision = disabled.result.policyRevision;
  await api(alice, `/v1/organizations/${organization.id}/members`, 403);
  check("membershipRevocationActsOnNextRequest", true);
  const enabled = await mutate(bob, organization.id, revision, { action: "set-member", issuer: accountConfig.issuer, subject: aliceUser.subject, role: "admin", active: true });
  revision = enabled.result.policyRevision;

  const artifact = Buffer.from(`Astra data-only catalog fixture ${runId}\n`);
  const release = { iss: policy.issuer, aud: policy.audience, sub: aliceUser.subject, exp: Math.floor(Date.now() / 1000) + 3600, package_id: randomUUID(), revision: "1", version: "1.0.0", name: `Astra fixture ${runId}`, kind: "asset", description: "Data-only protocol fixture", license_id: "probe-license-v1", license_text: "Local acceptance probe fixture.", artifact_digest: digest(artifact), artifact_size: artifact.length };
  const header = Buffer.from(JSON.stringify({ alg: "RS256", typ: "JWT", kid })).toString("base64url");
  const payload = Buffer.from(JSON.stringify(release)).toString("base64url");
  const envelope = `${header}.${payload}.${sign("RSA-SHA256", Buffer.from(`${header}.${payload}`), privateKey).toString("base64url")}`;
  const publish = { operationId: randomUUID(), envelope };
  await api(outsider, "/v1/catalog", 403, "POST", publish);
  const publication = await api(alice, "/v1/catalog", 200, "POST", publish);
  check("signedPublicationAndReplay", isDeepStrictEqual(await api(alice, "/v1/catalog", 200, "POST", publish), publication));
  const invalidEnvelope = { operationId: randomUUID(), envelope: `${header}.${Buffer.from(JSON.stringify({ ...release, name: "Modified" })).toString("base64url")}.${envelope.split(".")[2]}` };
  await api(alice, "/v1/catalog", 403, "POST", invalidEnvelope);
  await api(alice, "/v1/catalog", 409, "POST", { operationId: randomUUID(), envelope });
  check("catalogRejectsTamperingAndRevisionRollback", true);
  const catalog = await api(alice, `/v1/catalog?query=${encodeURIComponent(runId)}`);
  check("catalogListContainsVerifiedFixture", catalog.items.length === 1 && catalog.items[0].artifact_digest === digest(artifact));
  const artifactRoute = `/v1/organizations/${organization.id}/catalog/${release.package_id}/1`;
  await api(bob, artifactRoute, 403);
  const license = { operationId: randomUUID(), expectedPolicyRevision: revision, packageId: release.package_id, revision: "1", licenseId: release.license_id };
  await api(bob, `/v1/organizations/${organization.id}/licenses`, 200, "POST", license);
  const metadata = await api(bob, artifactRoute);
  check("licenseUnlocksOnlyAuthorizedReleaseMetadata", metadata.artifact_digest === digest(artifact) && metadata.artifact_size === artifact.length);
  await api(outsider, artifactRoute, 403);
  await publishPolicy({ ...policy, publishers: {} });
  check("catalogRevokedPublisherHiddenImmediately", (await api(alice, `/v1/catalog?query=${encodeURIComponent(runId)}`)).items.length === 0);
  await api(bob, artifactRoute, 403);
  await publishPolicy(policy);

  const cloudRoute = `/v1/organizations/${organization.id}/projects/${projectId}/cloud`;
  const files = [Buffer.from(`initial ${runId}`), Buffer.from(`writer-a ${runId}`), Buffer.from(`writer-b ${runId}`)];
  const manifest = bytes => ({ schemaVersion: 1, engine: "zircon-probe", packageLockDigest: "a".repeat(64), ignorePolicy: "zircon-project-v1", sourceRevision: "probe", files: [{ path: "Assets/scene.zr", digest: digest(bytes), bytes: bytes.length }] });
  for (const bytes of files) {
    const uploaded = await request(new URL(`${cloudRoute}/blobs/${digest(bytes)}`, serviceUrl), { method: "PUT", headers: { authorization: `Bearer ${bob.accessToken}` }, body: bytes });
    check(`cloudUpload${files.indexOf(bytes)}`, uploaded.status === 204);
  }
  await api(outsider, `${cloudRoute}/head`, 403);
  check("cloudTenantIsolation", true);
  const firstRequest = { operationId: randomUUID(), baseRevision: "0", manifest: manifest(files[0]) };
  const first = await api(bob, `${cloudRoute}/commit`, 200, "POST", firstRequest);
  check("initialSnapshotAndReplay", first.status === "committed" && first.snapshot.revision === "1" && isDeepStrictEqual(await api(bob, `${cloudRoute}/commit`, 200, "POST", firstRequest), first));
  const writers = files.slice(1).map(bytes => ({ operationId: randomUUID(), baseRevision: "1", manifest: manifest(bytes) }));
  const responses = await Promise.all(writers.map(body => request(new URL(`${cloudRoute}/commit`, serviceUrl), { method: "POST", headers: { authorization: `Bearer ${bob.accessToken}`, "content-type": "application/json" }, body: JSON.stringify(body) })));
  check("sameBaseHasOneWinnerAndOneDurableConflict", responses.filter(result => result.status === 200 && result.data.status === "committed").length === 1 && responses.filter(result => result.status === 409 && result.data.status === "conflict").length === 1);
  const loserIndex = responses.findIndex(result => result.status === 409);
  const loser = writers[loserIndex];
  const conflict = responses[loserIndex].data;
  const conflictReceipt = await api(bob, `/v1/operations/${loser.operationId}`);
  evidence.conflict = { operationId: loser.operationId, response: conflict, receipt: conflictReceipt };
  check("conflictReceiptIsTerminalAndStable", conflictReceipt.status === "committed" && isDeepStrictEqual(conflictReceipt.result, conflict) && isDeepStrictEqual(await api(bob, `${cloudRoute}/commit`, 409, "POST", loser), conflict));
  const rebased = { ...loser, operationId: randomUUID(), baseRevision: "2" };
  const rebasedResult = await api(bob, `${cloudRoute}/commit`, 200, "POST", rebased);
  check("explicitNewOperationResolvesConflict", rebasedResult.status === "committed" && rebasedResult.snapshot.revision === "3");
  const lostReplyRequest = { operationId: randomUUID(), expectedPolicyRevision: revision, mutation: { action: "create-project", name: "Discarded response project" } };
  stage = "discard organization mutation response";
  await new Promise((resolve, reject) => {
    const outgoing = httpRequest(new URL(`/v1/organizations/${organization.id}/mutations`, serviceUrl), { method: "POST", headers: { authorization: `Bearer ${bob.accessToken}`, "content-type": "application/json" } }, incoming => { incoming.destroy(); resolve(); });
    outgoing.once("error", reject);
    outgoing.setTimeout(15000, () => outgoing.destroy(new Error("Discarded response timed out")));
    outgoing.end(JSON.stringify(lostReplyRequest));
  });
  const lostReceipt = await api(bob, `/v1/operations/${lostReplyRequest.operationId}`);
  check("discardedResponseReconcilesThroughOperationId", lostReceipt.status === "committed" && lostReceipt.result.resourceId);
  evidence.recovery = { organizationId: organization.id, projectId, firstOperationId: firstRequest.operationId, conflictOperationId: loser.operationId, resolutionOperationId: rebased.operationId, discardedReplyOperationId: lostReplyRequest.operationId };
  await terminateService("intentional-hard-termination-for-recovery");
  await startService();
  check("restartUsesAnotherNativeProcess", evidence.processes[0].pid !== evidence.processes[1].pid);
  check("snapshotAndConflictSurviveRestart", (await api(bob, `${cloudRoute}/head`)).revision === "3" && isDeepStrictEqual((await api(bob, `/v1/operations/${loser.operationId}`)).result, conflict));
  check("discardedReplySurvivesRestartWithoutDuplicateProject", isDeepStrictEqual(await api(bob, `/v1/organizations/${organization.id}/mutations`, 200, "POST", lostReplyRequest), lostReceipt.result) && (await api(bob, `/v1/organizations/${organization.id}/projects`)).items.length === 2);
  const restored = await request(new URL(`${cloudRoute}/blobs/${loser.manifest.files[0].digest}`, serviceUrl), { headers: { authorization: `Bearer ${bob.accessToken}` } });
  check("encryptedCloudBytesRecoverAfterRestart", restored.status === 200 && restored.bytes.equals(files[loserIndex + 1]) && restored.headers.get("cache-control") === "no-store");
  check("licenseSurvivesRestart", (await api(bob, artifactRoute)).artifact_digest === digest(artifact));
  await oidc.refresh(bob);
  await api(bob, `/v1/organizations/${organization.id}/members`);
  check("refreshedTokenWorksThroughNativeVerifier", true);
  await oidc.logout(bob);
  await api(bob, `/v1/organizations/${organization.id}/members`, 401);
  check("logoutRevokesNextNativeServiceRequest", true);
  check("originalDeploymentConfigPreserved", digest(await readFile(sourceConfigPath)) === sourceConfigHash);
  evidence.status = "passed";
} catch {
  evidence.status = "failed";
  evidence.failedStage = stage;
  process.exitCode = 1;
} finally {
  await terminateService("probe-cleanup-hard-termination").catch(() => { evidence.cleanupServiceFailed = true; process.exitCode = 1; });
  evidence.identityCleanupFailures = await oidc.close();
  if (evidence.identityCleanupFailures.length) { evidence.status = "failed"; process.exitCode = 1; }
  await browser.close();
  evidence.completedAt = new Date().toISOString();
  await writeFile(path.join(runDirectory, "evidence.json"), `${JSON.stringify(evidence, null, 2)}\n`, { flag: "wx" });
  console.log(JSON.stringify({ status: evidence.status, passed: Object.values(evidence.checks).filter(Boolean).length, failedStage: evidence.failedStage ?? null, evidence: path.join(runDirectory, "evidence.json") }));
}
