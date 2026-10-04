import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";
import test from "node:test";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";
import { accountAdminShell } from "./account_admin_shell.mjs";

const require = createRequire(import.meta.url);
const playwrightRoot = process.env.ZIRCON_PLAYWRIGHT_NODE_MODULES;
const chromium = playwrightRoot ? require(path.join(playwrightRoot, "playwright")).chromium : undefined;
const executablePath = process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH;

test("CloudPage resumes the native operation identity without accepting Web paths or manifests", { skip: !chromium || !executablePath }, async () => {
  const organization = "00000000-0000-4000-8000-000000000001";
  const project = "00000000-0000-4000-8000-000000000002";
  const digest = "a".repeat(64);
  const validationRoot = process.env.ZIRCON_HUB_VALIDATION_ROOT
    ?? "E:/cargo-targets/zircon-engine/hub-web-validation";
  const runRoot = path.join(validationRoot, `cloud-sync-browser-${process.pid}`);
  await mkdir(path.join(runRoot, "vite-cache"), { recursive: true });
  await mkdir(path.join(runRoot, "profile"), { recursive: true });
  process.env.TEMP = path.join(runRoot, "temp");
  process.env.TMP = process.env.TEMP;
  await mkdir(process.env.TEMP, { recursive: true });
  const shellText = await accountAdminShell("English");
  const vite = await createServer({ configFile: false, cacheDir: path.join(runRoot, "vite-cache"), root: fileURLToPath(new URL("../", import.meta.url)), server: { host: "127.0.0.1", port: 0 } });
  await vite.listen();
  const context = await chromium.launchPersistentContext(path.join(runRoot, "profile"), {
    headless: true,
    executablePath,
    viewport: { width: 960, height: 900 },
  });
  try {
    await context.addInitScript(({ shellText, organization, project, digest }) => {
      let operations = [];
      let operationsRevision = "0";
      let committed = false;
      let pendingOperationId = null;
      window.__CLOUD_SYNC_REQUESTS__ = [];
      const accountSnapshot = (data = null, error = null) => ({
        backendEpoch: "cloud-sync-browser",
        account: { configured: true, status: "signed-in", issuer: "https://issuer.test", subject: "alice", displayName: "Alice", generation: "7", error: null },
        data, error, operations, operationsRevision, operationsError: null,
      });
      const head = revision => ({ organizationId: organization, projectId: project, baseRevision: String(BigInt(revision) - 1n), revision,
        manifestDigest: digest, createdAt: 1, createdBy: digest,
        manifest: { schemaVersion: 1, engine: "Zircon", packageLockDigest: digest, ignorePolicy: "zircon-project-v1", sourceRevision: null, files: [] },
      });
      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
      window.__TAURI_INTERNALS__ = {
        transformCallback() { return 1; }, unregisterCallback() {},
        async invoke(command, args) {
          if (command === "plugin:event|listen") return 1;
          if (command === "hub_state") {
            const { fallbackShellState } = await import("/src/data/hubData.ts");
            return { ...fallbackShellState, backendEpoch: "cloud-sync-browser", stateRevision: "1", activePage: "cloud", pageTitle: "Cloud", pageSubtitle: "",
              ui: shellText.ui, comingSoon: [], settings: { ...fallbackShellState.settings, language: "English" } };
          }
          if (command === "account_state") return accountSnapshot();
          if (command !== "account_action") return undefined;
          const request = args.request;
          window.__CLOUD_SYNC_REQUESTS__.push(request);
          if (request.action === "organizations") return accountSnapshot({ items: [{ id: organization, name: "Team", policyRevision: "1" }], nextCursor: null });
          if (request.action === "invitations" || request.action === "members") return accountSnapshot({ items: [], nextCursor: null });
          if (request.action === "projects") return accountSnapshot({ items: [{ id: project, name: "Demo" }], nextCursor: null });
          if (request.action === "cloud-binding") return accountSnapshot({ localProjectGuid: project, organizationId: organization, projectId: project });
          if (request.action === "cloud-head") return accountSnapshot(head(committed ? "5" : "4"));
          if (request.action === "cloud-push") {
            pendingOperationId = request.operationId;
            operations = [{ operationId: request.operationId, status: "unknown", action: "cloud-commit", organizationId: organization, error: "account_service_outcome_unknown" }];
            operationsRevision = "1";
            return accountSnapshot(null, "account_service_outcome_unknown");
          }
          if (request.action === "cloud-resume-push") {
            committed = true;
            operations = [{ operationId: pendingOperationId, status: "committed", action: "cloud-commit", organizationId: organization, error: null }];
            operationsRevision = "2";
            return accountSnapshot({ status: "committed", result: { status: "committed", snapshot: {
              organizationId: organization, projectId: project, baseRevision: "4", revision: "5", manifestDigest: digest,
            } } });
          }
          if (request.action === "cloud-stage-download") {
            return accountSnapshot({ status: "staged", stageId: project, revision: request.expectedRevision, manifestDigest: digest, fileCount: 0, totalBytes: "0" });
          }
          if (request.action === "cloud-discard-download") {
            return accountSnapshot({ status: "discarded", stageId: project, revision: request.expectedRevision });
          }
          throw new Error(`unexpected account action ${request.action}`);
        },
      };
    }, { shellText, organization, project, digest });

    const page = await context.newPage();
    const errors = [];
    page.on("pageerror", error => errors.push(error.message));
    await page.goto(vite.resolvedUrls.local[0]);
    const push = page.getByRole("button", { name: "Upload local snapshot" });
    await push.waitFor();
    await push.click();
    await page.getByText("Cloud outcome unknown. Check the service receipt before another write.").waitFor();
    const initialOperationId = await page.waitForFunction(() =>
      window.__CLOUD_SYNC_REQUESTS__.find(request => request.action === "cloud-push")?.operationId ?? null,
    );
    const capturedOperationId = await initialOperationId.jsonValue();
    assert.match(capturedOperationId, /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
    await page.getByText(capturedOperationId).waitFor();
    await page.getByRole("button", { name: "Resume same upload" }).click();
    await page.getByText("Cloud operation receipt confirmed.").waitFor();
    await page.getByRole("button", { name: "Stage cloud snapshot" }).click();
    await page.getByText("Verified download is staged. Applying adds files only when no local file would be replaced.").waitFor();
    await page.getByRole("button", { name: "Discard staged download" }).click();
    await page.getByRole("button", { name: "Confirm discard" }).click();
    await page.getByText("Staged recovery data was removed. The cloud snapshot is unchanged.").waitFor();

    const requests = await page.evaluate(() => window.__CLOUD_SYNC_REQUESTS__);
    const initial = requests.find(request => request.action === "cloud-push");
    const resume = requests.find(request => request.action === "cloud-resume-push");
    const stage = requests.find(request => request.action === "cloud-stage-download");
    const discard = requests.find(request => request.action === "cloud-discard-download");
    assert.deepEqual(Object.keys(initial).sort(), ["action", "backendEpoch", "baseRevision", "generation", "operationId"].sort());
    assert.deepEqual(Object.keys(resume).sort(), ["action", "backendEpoch", "generation", "operationId"].sort());
    assert.equal(initial.operationId, capturedOperationId);
    assert.equal(resume.operationId, initial.operationId);
    assert.equal(initial.baseRevision, "4");
    assert.deepEqual(Object.keys(stage).sort(), ["action", "backendEpoch", "expectedRevision", "generation"].sort());
    assert.equal(stage.expectedRevision, "5");
    assert.deepEqual(Object.keys(discard).sort(), ["action", "backendEpoch", "expectedRevision", "generation"].sort());
    assert.equal(discard.expectedRevision, stage.expectedRevision);
    assert.deepEqual(errors, []);
    await context.close();
  } finally {
    await context.close();
    await vite.close();
  }
});
