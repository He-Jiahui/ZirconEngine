import { resolveLayoutCatalogPath } from './zui-layout-paths';
import assert from 'node:assert/strict';
import {
  capturePenpotCases,
  evidenceAfterPenpotFailure,
  penpotCasesComplete,
} from './zui-layout-penpot-cases';
import { clickAndWaitForDownload, downloadPath } from './zui-layout-download';
import { createHash } from 'node:crypto';
import { existsSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  chromium,
  type FrameLocator,
  type Locator,
  type Page,
} from 'playwright';

import { parseZuiDocument } from '../src/bridge/zui-document.js';
import {
  renderAcceptedResult,
  renderResult,
  renderIndex,
  type CatalogManifest,
} from './zui-layout-catalog.js';
import {
  auditVisualPixels,
  hasVisibleSmallControl,
  hasVisibleThinControl,
} from '../src/visual-pixel-audit.js';
import {
  openPenpotSession,
  recordPenpotSessionFailure,
  resolvePluginBundlePath,
  resolvePluginDistRoot,
} from './zui-layout-penpot-session';
import {
  bytesSha256,
  containedPath,
  currentDualReview,
} from './zui-layout-evidence';
import { structuralShellPendingReason } from './zui-layout-structural-shell';
import { isIntentionallyClosedPopupCase } from './zui-layout-closed-popup';
import {
  compareLayoutReviewOrder,
  layoutReviewPhase,
} from './zui-layout-review-order';
import type { BrowserRuntimeRecord } from './zui-layout-review-contract';
import {
  captureProgramFingerprints,
  currentCaptureProgram,
} from './zui-layout-capture-provenance';
import { runReviewPhases, serializedWriter } from './zui-layout-worker-pool';
import { acquireLayoutCatalogLock } from './zui-layout-catalog-lock';
import { prepareVisualSourceEntries } from './zui-layout-visual-source-scope';
import { nonEmptyRatio, readPng } from './zui-layout-png';
import {
  assertCheckpointSources,
  archiveReview,
  mergeCheckpointReviews,
} from './zui-layout-review-history';

const here = dirname(fileURLToPath(import.meta.url));
const appRoot = resolve(here, '..');
const penpotRoot = resolve(appRoot, '../../../../third_party/penpot');
const repoRoot = resolve(process.env['ZUI_LAYOUT_REPO_ROOT'] ?? resolve(penpotRoot, '../..'));
const pluginDistRoot = resolvePluginDistRoot();
const catalogRoot = resolve(repoRoot, 'docs/_data/layout');
const catalogPath = resolveLayoutCatalogPath(catalogRoot, 'catalog.json');
const summaryPath = resolveLayoutCatalogPath(catalogRoot, 'visual-validation.json');
const validationProfile = 'zircon.penpot.prefabs.v1';
const penpotBaseUrl = new URL(
  process.env['PENPOT_BASE_URL'] ?? 'https://design.penpot.app',
).origin;
let pluginBundleSha256 = '';

interface VisualValidationResult {
  sourcePath: string;
  outputPath: string;
  outputSha256: string;
  previewPath: string;
  status: 'passed' | 'pending' | 'failed';
  importedNodes: number;
  exportedFile: string | null;
  semanticMatch: boolean;
  width: number;
  height: number;
  nonEmptyPixelRatio: number;
  detailPixelRatio: number;
  colorBucketCount: number;
  layoutTotalNodes: number;
  layoutCheckedNodes: number;
  layoutOverflowCount: number;
  layoutInvalidGeometryCount: number;
  previewBoardId: string;
  screenshotSha256: string;
  error?: string;
}

interface VisualValidationSummary {
  schema: 'dev.zircon.zui.layout-visual-validation';
  version: 4;
  generatedAt: string;
  validationProfile: typeof validationProfile;
  pluginBundleSha256: string;
  penpotFrontend: string;
  backend: 'repository-mock';
  total: number;
  passed: number;
  pending: number;
  failed: number;
  results: VisualValidationResult[];
}

const browserExecutable = [
  process.env['PLAYWRIGHT_CHROMIUM_EXECUTABLE'],
  process.env['ProgramFiles']
    ? resolve(
        process.env['ProgramFiles'],
        'Google/Chrome/Application/chrome.exe',
      )
    : undefined,
  process.env['ProgramFiles(x86)']
    ? resolve(
        process.env['ProgramFiles(x86)'],
        'Microsoft/Edge/Application/msedge.exe',
      )
    : undefined,
].find((candidate): candidate is string =>
  Boolean(candidate && existsSync(candidate)),
);

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const writeEvidence = !args.includes('--check');
  const lock = writeEvidence
    ? await acquireLayoutCatalogLock(catalogRoot, {
        command: ['zui-layout-visual-validation.ts', ...args].join(' '),
      })
    : undefined;
  try {
    await runVisualValidation(args, writeEvidence);
  } finally {
    await lock?.release();
  }
}

async function runVisualValidation(
  args: string[],
  writeEvidence: boolean,
): Promise<void> {
  const start = numberOption(args, '--start', 0);
  const limit = numberOption(args, '--limit', Number.POSITIVE_INFINITY);
  const workers = numberOption(args, '--workers', 1);
  const importTimeoutMs = numberOption(args, '--import-timeout-ms', 300_000);
  const reviewTimeoutMs = numberOption(args, '--review-timeout-ms', 300_000);
  // Composite review hosts can spend several minutes rebuilding native
  // instances before the official frontend emits its download event.
  // Keep this bounded, but do not mistake the historical 30s small-fixture
  // timeout for a completion guarantee on a real workbench projection.
  const exportTimeoutMs = numberOption(args, '--export-timeout-ms', 300_000);
  const caseIds = new Set(
    args.flatMap((argument, index) => {
      if (argument !== '--case') return [];
      const value = args[index + 1];
      assert.ok(value && !value.startsWith('--'), '--case requires an id');
      return [value];
    }),
  );
  const manifest = JSON.parse(
    await readFile(catalogPath, 'utf8'),
  ) as CatalogManifest;
  const compiledPluginBundle = await readFile(resolvePluginBundlePath());
  const distPluginBundle = await readFile(
    resolve(pluginDistRoot, 'assets/plugin.js'),
  );
  assert.equal(
    sha256(distPluginBundle),
    sha256(compiledPluginBundle),
    'Penpot visual validation requires a current distribution bundle; run pnpm build first.',
  );
  pluginBundleSha256 = sha256(distPluginBundle);
  const programFingerprints = await captureProgramFingerprints(repoRoot);
  const sourceCandidates = await prepareVisualSourceEntries(
    manifest.entries,
    args,
    async (entry) => {
      if (
        entry.visualPluginSha256 !== pluginBundleSha256 ||
        !currentCaptureProgram(entry.penpotEvidence, programFingerprints)
      ) {
        entry.visualStatus = 'pending';
        archiveReview(entry);
        if (writeEvidence)
          await writeFile(
            resolveLayoutCatalogPath(catalogRoot, entry.resultPath),
            renderResult(entry),
            'utf8',
          );
      }
    },
  );
  const ordered = [...sourceCandidates];
  if (args.includes('--milestone-order'))
    ordered.sort(compareLayoutReviewOrder);
  const entries = ordered
    .filter(
      (entry) => !args.includes('--pending') || entry.visualStatus !== 'passed',
    )
    .slice(start, start + limit);
  await assertCatalogSourcesCurrent(entries);
  const unexpectedRpcs = new Set<string>();
  const pageErrors: string[] = [];
  const browser = await chromium.launch({
    headless: true,
    args: ['--ignore-certificate-errors'],
    ...(browserExecutable ? { executablePath: browserExecutable } : {}),
  });
  let browserRuntimePromise: Promise<BrowserRuntimeRecord> | undefined;
  const captureBrowserRuntime = (page: Page): Promise<BrowserRuntimeRecord> =>
    (browserRuntimePromise ??= browserRuntimeRecord(page));
  const results: VisualValidationResult[] = [];
  const checkpoint = serializedWriter(
    async (value: {
      manifest: CatalogManifest;
      results: VisualValidationResult[];
    }) => writeCheckpoint(value.manifest, value.results),
  );
  const publish = async (entry: CatalogManifest['entries'][number]) => {
    Object.assign(
      manifest.entries.find(
        (candidate) => candidate.sourcePath === entry.sourcePath,
      )!,
      entry,
    );
    await checkpoint({ manifest, results });
  };
  const openWorkerRuntime = () =>
    openPenpotSession(browser, {
      repoRoot,
      catalogRoot,
      writeEvidence,
      penpotBaseUrl,
      pluginBundleSha256,
      pageErrors,
      unexpectedRpcs,
      captureProgramFingerprints: programFingerprints,
      captureBrowserRuntime,
    });
  const runWorker = async (
    take: () => CatalogManifest['entries'][number] | undefined,
  ) => {
    let runtime = await openWorkerRuntime();
    try {
      for (let queued = take(); queued !== undefined; queued = take()) {
        const {
          context,
          page,
          pluginFrame,
          fileInput,
          statusPanel,
          exportButton,
          fontResources,
        } = runtime;
        const entry = structuredClone(queued);
        const selectedCaseIds = new Set(
          (entry.cases ?? [])
            .filter((item) => !caseIds.size || caseIds.has(item.id))
            .map((item) => item.id),
        );
        const absoluteZuiPath = resolveLayoutCatalogPath(
          catalogRoot,
          entry.penpotInputPath ?? entry.outputPath,
        );
        let importedAttempt = false;
        let caseCaptureStarted = false;
        try {
          assert.equal(
            entry.status,
            'prepared',
            `Document preparation failed: ${entry.diagnostics
              .filter((item) => item.severity === 'error')
              .map((item) => item.message)
              .join('; ')}`,
          );
          const source = await readFile(absoluteZuiPath, 'utf8');
          const parsed = parseZuiDocument(source).document;
          const originalSource = parsed['penpot_original_source'];
          assert.equal(
            typeof originalSource,
            'string',
            'Regenerate the catalog with original source metadata',
          );
          const expectedDisplayName = normalizeText(
            parsed.asset.display_name ?? parsed.asset.id,
          );
          assert.equal(
            sha256(Buffer.from(source)),
            entry.penpotInputSha256 ?? entry.outputSha256,
            'Penpot input differs from catalog hash; regenerate catalog',
          );
          importedAttempt = true;
          await requestReviewMaterialization(
            pluginFrame,
            statusPanel,
            reviewMaterializationMode(entry),
          );
          await fileInput.setInputFiles(absoluteZuiPath, { timeout: 300_000 });
          await waitForImportedAsset(
            statusPanel,
            expectedDisplayName,
            entry.projectedShapeCount,
            importTimeoutMs,
          );
          const statusLevel = await statusPanel.getAttribute('data-level');
          const statusText = normalizeText(await statusPanel.innerText());
          assert.notEqual(statusLevel, 'error', statusText);
          assert.match(
            statusText,
            new RegExp(`\\b${entry.projectedShapeCount} nodes\\b`),
          );
          const previewBoardId = await requiredAttribute(
            statusPanel,
            'data-preview-board-id',
            'Penpot preview board id',
          );
          await waitFor(
            async () => exportButton.isEnabled(),
            'enabled ZUI export',
          );
          let download;
          const exportErrorWatcher = new AbortController();
          try {
            download = await clickAndWaitForDownload(
              () =>
                Promise.race([
                  page.waitForEvent('download', { timeout: exportTimeoutMs }),
                  waitForExportError(
                    statusPanel,
                    exportTimeoutMs,
                    exportErrorWatcher.signal,
                  ),
                ]),
              () => exportButton.click({ timeout: exportTimeoutMs }),
            );
          } catch (error) {
            const level = await statusPanel.getAttribute('data-level');
            const message = normalizeText(await statusPanel.innerText());
            const cause = compactErrorMessage(error);
            throw new Error(
              `Export did not produce a download (status=${level ?? 'unknown'}, budget=${exportTimeoutMs}ms): ${message || cause}; cause=${cause}`,
              { cause: error },
            );
          } finally {
            // Once the download wins, stop the status poller rather than
            // leaving a five-minute timer running into the next entry.
            exportErrorWatcher.abort();
          }
          const downloadedPath = await downloadPath(download, exportTimeoutMs);
          assert.ok(
            downloadedPath,
            `missing exported file for ${entry.sourcePath}`,
          );
          const exportedSource = await readFile(downloadedPath, 'utf8');
          assert.equal(
            exportedSource,
            originalSource,
            'No-edit export must restore the exact product source',
          );

          caseCaptureStarted = true;
          const caseCapture = await capturePenpotCases({
            page,
            pluginFrame,
            statusPanel,
            pluginModal: page.locator('plugin-modal'),
            previewBoardId,
            entry,
            writeEvidence,
            catalogRoot,
            pluginBundleSha256,
            captureProgramFingerprints: programFingerprints,
            captureFontResources: () => fontResources.snapshot(),
            captureFontResourceRecords: () => fontResources.snapshotRecords(),
            captureBrowserRuntime: () => captureBrowserRuntime(page),
            caseIds: caseIds.size ? caseIds : undefined,
            reviewTimeoutMs,
            exportTimeoutMs,
            validateImage: async (bytes, reviewCase, layout) => {
              const pixels = await readPng(bytes);
              assert.equal(
                pixels.width,
                Math.round(reviewCase.viewport.width * reviewCase.dpi),
              );
              assert.equal(
                pixels.height,
                Math.round(reviewCase.viewport.height * reviewCase.dpi),
              );
              const compactControl =
                reviewCase.host === 'component' &&
                (hasVisibleSmallControl(
                  pixels,
                  layout.semanticNodes
                    .filter(
                      (node) =>
                        node.visible &&
                        !node.detached &&
                        node.parentNodeId === null,
                    )
                    .map((node) => node.bounds),
                  reviewCase.dpi,
                ) ||
                  hasVisibleThinControl(
                    pixels,
                    layout.semanticNodes
                      .filter(
                        (node) =>
                          node.visible &&
                          !node.detached &&
                          node.parentNodeId === null &&
                          ['Divider', 'Separator'].includes(node.component),
                      )
                      .map((node) => node.bounds),
                    reviewCase.dpi,
                  ));
              const blank =
                !compactControl &&
                nonEmptyRatio(pixels) * pixels.width * pixels.height < 100;
              const audit = auditVisualPixels(
                pixels.width,
                pixels.height,
                pixels.pixels,
              );
              const flat =
                !compactControl &&
                audit.detailPixelRatio <= 0.002 &&
                audit.colorBucketCount <= 2;
              if (isIntentionallyClosedPopupCase(parsed, reviewCase, layout)) {
                assert.ok(
                  flat,
                  `closed popup paints unexpected content for ${reviewCase.id}`,
                );
                return undefined;
              }
              const pendingReason =
                blank || flat
                  ? structuralShellPendingReason(
                      entry.sourceClassification,
                      layout,
                    )
                  : undefined;
              assert.ok(
                pendingReason || !blank,
                `blank Penpot screenshot for ${reviewCase.id}`,
              );
              assert.ok(
                pendingReason || !flat,
                `flat Penpot screenshot for ${reviewCase.id}`,
              );
              return pendingReason;
            },
          });
          const previewBytes = caseCapture.primaryBytes;
          const png = await readPng(previewBytes);
          const penpotEvidence = caseCapture.evidence;
          const layoutAudit = caseCapture.primaryLayout;
          const pendingReason = penpotEvidence.find(
            (item) => item.status === 'pending',
          )?.pendingReason;
          const coveragePendingReason = penpotCasesComplete(
            entry,
            pluginBundleSha256,
            programFingerprints,
          )
            ? undefined
            : `Penpot capture has not passed every current review case (${penpotEvidence.filter((item) => item.status === 'passed').length}/${entry.cases?.length ?? 0}).`;
          const visualPendingReason = pendingReason ?? coveragePendingReason;
          const primaryEvidence = penpotEvidence.find(
            (item) =>
              item.caseId === entry.cases?.[0]?.id &&
              item.status === 'passed' &&
              item.sourceSha256 === entry.sourceSha256 &&
              item.inputSha256 === entry.penpotInputSha256 &&
              currentCaptureProgram([item], programFingerprints),
          );
          entry.visualSha256 = primaryEvidence?.screenshotSha256;
          assert.equal(
            layoutAudit.overflowCount,
            0,
            `semantic layout overflow in ${entry.sourcePath}: ${layoutAudit.overflowDetails}`,
          );
          assert.equal(
            layoutAudit.invalidGeometryCount,
            0,
            `invalid semantic geometry in ${entry.sourcePath}: ${layoutAudit.invalidDetails}`,
          );
          const nonEmptyPixelRatio = nonEmptyRatio(png);
          const pixelAudit = auditVisualPixels(
            png.width,
            png.height,
            png.pixels,
          );
          const result: VisualValidationResult = {
            sourcePath: entry.sourcePath,
            outputPath: entry.outputPath,
            outputSha256: entry.outputSha256,
            previewPath: entry.previewPath,
            status: visualPendingReason ? 'pending' : 'passed',
            importedNodes: entry.projectedShapeCount,
            exportedFile: download.suggestedFilename(),
            semanticMatch: true,
            width: png.width,
            height: png.height,
            nonEmptyPixelRatio,
            detailPixelRatio: pixelAudit.detailPixelRatio,
            colorBucketCount: pixelAudit.colorBucketCount,
            layoutTotalNodes: layoutAudit.totalNodes,
            layoutCheckedNodes: layoutAudit.checkedNodes,
            layoutOverflowCount: layoutAudit.overflowCount,
            layoutInvalidGeometryCount: layoutAudit.invalidGeometryCount,
            previewBoardId,
            screenshotSha256: sha256(previewBytes),
          };
          results.push(result);
          Object.assign(entry, {
            visualStatus: result.status,
            visualPluginSha256: pluginBundleSha256,
            visualError: undefined,
            visualPendingReason,
            visualScreenshot: primaryEvidence?.screenshotPath,
            visualSha256: primaryEvidence?.screenshotSha256,
            visualPixelRatio: result.nonEmptyPixelRatio,
            visualDetailPixelRatio: result.detailPixelRatio,
            visualColorBucketCount: result.colorBucketCount,
            visualLayoutOverflowCount: result.layoutOverflowCount,
            visualLayoutInvalidGeometryCount: result.layoutInvalidGeometryCount,
            penpotEvidence,
          });
          if (!currentDualReview(entry)) archiveReview(entry);
          if (writeEvidence) {
            await writeFile(
              resolveLayoutCatalogPath(catalogRoot, entry.resultPath),
              result.status === 'passed'
                ? renderAcceptedResult(entry)
                : renderResult(entry),
              'utf8',
            );
            await publish(entry);
          }
        } catch (error) {
          console.error(
            JSON.stringify({
              sourcePath: entry.sourcePath,
              error: compactErrorMessage(error),
            }),
          );
          results.push({
            sourcePath: entry.sourcePath,
            outputPath: entry.outputPath,
            outputSha256: entry.outputSha256,
            previewPath: entry.previewPath,
            status: 'failed',
            importedNodes: entry.projectedShapeCount,
            exportedFile: null,
            semanticMatch: false,
            width: 0,
            height: 0,
            nonEmptyPixelRatio: 0,
            detailPixelRatio: 0,
            colorBucketCount: 0,
            layoutTotalNodes: 0,
            layoutCheckedNodes: 0,
            layoutOverflowCount: 0,
            layoutInvalidGeometryCount: 0,
            previewBoardId: '',
            screenshotSha256: '',
            error: errorMessage(error),
          });
          entry.visualStatus = 'failed';
          entry.visualError = errorMessage(error);
          delete entry.visualPendingReason;
          entry.visualPluginSha256 = pluginBundleSha256;
          entry.penpotEvidence = evidenceAfterPenpotFailure(
            entry,
            entry.penpotEvidence,
            selectedCaseIds,
            caseCaptureStarted,
            pluginBundleSha256,
            programFingerprints,
            errorMessage(error),
          );
          delete entry.visualSha256;
          delete entry.visualScreenshot;
          archiveReview(entry);
          if (writeEvidence) {
            await writeFile(
              resolveLayoutCatalogPath(catalogRoot, entry.resultPath),
              renderResult(entry),
              'utf8',
            );
            await publish(entry);
          }
          if (importedAttempt) {
            if (writeEvidence)
              await recordPenpotSessionFailure(
                runtime,
                resolveLayoutCatalogPath(catalogRoot, dirname(entry.outputPath), 'evidence'),
                error,
              );
            // A failed import may leave an active plugin operation behind.
            fontResources.stop();
            await context.close();
            runtime = await openWorkerRuntime();
          }
        }
        if (results.length % 10 === 0 || results.length === entries.length) {
          console.log(
            JSON.stringify({
              completed: results.length,
              total: entries.length,
              failed: results.filter(({ status }) => status === 'failed')
                .length,
              pending: results.filter(({ status }) => status === 'pending')
                .length,
            }),
          );
        }
      }
    } finally {
      runtime.fontResources.stop();
      await runtime.context.close();
    }
  };
  try {
    const phases = args.includes('--milestone-order')
      ? [...new Set(entries.map(layoutReviewPhase))].map((phase) =>
          entries.filter((entry) => layoutReviewPhase(entry) === phase),
        )
      : [entries];
    await runReviewPhases(phases, workers, runWorker);
    assert.deepEqual([...unexpectedRpcs], []);
    assert.deepEqual(pageErrors, []);
  } finally {
    await browser.close();
    if (writeEvidence) await checkpoint({ manifest, results });
  }

  const failures = results.filter(({ status }) => status === 'failed');
  const pending = results.filter(({ status }) => status === 'pending');
  console.log(
    JSON.stringify({
      total: results.length,
      passed: results.filter(({ status }) => status === 'passed').length,
      pending: pending.length,
      failed: failures.length,
      summaryPath,
    }),
  );
  if (failures.length > 0) process.exitCode = 1;
}

async function browserRuntimeRecord(page: Page): Promise<BrowserRuntimeRecord> {
  const executablePath = resolve(
    browserExecutable ?? chromium.executablePath(),
  );
  const session = await page.context().newCDPSession(page);
  try {
    const version = await session.send('Browser.getVersion');
    return {
      product: version.product,
      userAgent: version.userAgent,
      executablePath,
      executableSha256: sha256(await readFile(executablePath)),
    };
  } finally {
    await session.detach();
  }
}

function reviewMaterializationMode(
  entry: CatalogManifest['entries'][number],
): 'full' | 'leaf' | 'semantic' {
  // Workbench Window is a retained shell containing several deep composite
  // prefab trees. Its design-only review board stays semantic so the official
  // Penpot frontend does not materialize every inactive workspace branch;
  // product/plugin imports always use the full native policy.
  if (entry.sourcePath.endsWith('/workbench_window.zui')) return 'semantic';
  // Material Component Lab is an authored catalog page containing all MUI
  // prototype families (1,429 projected nodes). Keep the complete semantic
  // projection but defer native-library promotion for the design-only review
  // import. Product/runtime imports still use the full policy; this avoids
  // turning the official Penpot frontend into an inactive component-library
  // expansion while preserving every authored node and its editable mapping.
  if (entry.sourcePath.endsWith('/material_component_lab.zui'))
    return 'semantic';
  return 'full';
}

async function requestReviewMaterialization(
  pluginFrame: FrameLocator,
  statusPanel: Locator,
  mode: 'full' | 'leaf' | 'semantic',
): Promise<void> {
  // This helper is intentionally kept as a tiny protocol message instead of
  // encoding review behaviour in file names or runtime source documents.
  await pluginFrame.locator('body').evaluate((_, value) => {
    parent.postMessage(
      { type: 'set-review-materialization', mode: value },
      '*',
    );
  }, mode);
  await waitFor(
    async () => {
      const snapshot = await readableStatus(statusPanel);
      return (
        snapshot?.status.includes(`Review materialization · ${mode}`) ?? false
      );
    },
    `review materialization acknowledgement (${mode})`,
    10_000,
  );
}

async function assertCatalogSourcesCurrent(
  entries: CatalogManifest['entries'],
): Promise<void> {
  for (const entry of entries) {
    const source = await readFile(containedPath(repoRoot, entry.sourcePath));
    assert.equal(
      bytesSha256(source),
      entry.sourceSha256,
      `Catalog source changed since catalog generation: ${entry.sourcePath}. Run pnpm exec tsx tools/zui-layout-catalog.ts before visual validation.`,
    );
  }
}

async function writeCheckpoint(
  manifest: CatalogManifest,
  currentResults: VisualValidationResult[],
): Promise<void> {
  const diskManifest = JSON.parse(
    await readFile(catalogPath, 'utf8'),
  ) as CatalogManifest;
  assertCheckpointSources(manifest, diskManifest);
  for (const entry of manifest.entries) {
    const diskEntry = diskManifest.entries.find(
      (candidate) => candidate.sourcePath === entry.sourcePath,
    );
    mergeCheckpointReviews(entry, diskEntry);
  }
  const existing = existsSync(summaryPath)
    ? (JSON.parse(
        await readFile(summaryPath, 'utf8'),
      ) as VisualValidationSummary)
    : null;
  const merged = new Map<string, VisualValidationResult>();
  const currentEntries = new Map(
    manifest.entries.map((entry) => [entry.sourcePath, entry]),
  );
  if (
    existing?.version === 4 &&
    existing.validationProfile === validationProfile &&
    existing.pluginBundleSha256 === pluginBundleSha256
  ) {
    for (const result of existing.results) {
      const entry = currentEntries.get(result.sourcePath);
      if (
        entry?.outputSha256 === result.outputSha256 &&
        entry.visualStatus === result.status &&
        entry.visualPluginSha256 === pluginBundleSha256
      ) {
        merged.set(result.sourcePath, result);
      }
    }
  }
  for (const result of currentResults) merged.set(result.sourcePath, result);
  // A checkpoint is a batch status projection, not just a log of workers that
  // happened to run in this invocation.  Keep one explicit status row for
  // every catalog entry so `total`, `pending`, and `failed` remain
  // self-consistent when a bounded or filtered run leaves most entries
  // untouched.  These rows intentionally contain no synthetic screenshot,
  // geometry, or board identity; they only expose the catalog's current
  // capture state and the reason it still needs work.
  for (const entry of manifest.entries) {
    if (merged.has(entry.sourcePath)) continue;
    const status = entry.visualStatus;
    merged.set(entry.sourcePath, {
      sourcePath: entry.sourcePath,
      outputPath: entry.outputPath,
      outputSha256: entry.outputSha256,
      previewPath: entry.previewPath,
      status,
      importedNodes: entry.projectedShapeCount,
      exportedFile: null,
      semanticMatch: status === 'passed',
      width: 0,
      height: 0,
      nonEmptyPixelRatio: entry.visualPixelRatio ?? 0,
      detailPixelRatio: entry.visualDetailPixelRatio ?? 0,
      colorBucketCount: entry.visualColorBucketCount ?? 0,
      layoutTotalNodes: 0,
      layoutCheckedNodes: 0,
      layoutOverflowCount: entry.visualLayoutOverflowCount ?? 0,
      layoutInvalidGeometryCount: entry.visualLayoutInvalidGeometryCount ?? 0,
      previewBoardId: '',
      screenshotSha256: entry.visualSha256 ?? '',
      ...(status === 'failed'
        ? { error: entry.visualError ?? 'Capture failed' }
        : status === 'pending'
          ? {
              error:
                entry.visualPendingReason ??
                'Capture has not completed for the current source and renderer fingerprints',
            }
          : {}),
    });
  }
  const results = [...merged.values()].sort((left, right) =>
    left.sourcePath.localeCompare(right.sourcePath),
  );
  const summary: VisualValidationSummary = {
    schema: 'dev.zircon.zui.layout-visual-validation',
    version: 4,
    generatedAt: new Date().toISOString(),
    validationProfile,
    pluginBundleSha256,
    penpotFrontend: penpotBaseUrl,
    backend: 'repository-mock',
    total: manifest.entries.length,
    passed: results.filter(({ status }) => status === 'passed').length,
    pending: results.filter(({ status }) => status === 'pending').length,
    failed: results.filter(({ status }) => status === 'failed').length,
    results,
  };
  await writeFile(summaryPath, `${JSON.stringify(summary, null, 2)}\n`, 'utf8');
  await writeFile(
    catalogPath,
    `${JSON.stringify(manifest, null, 2)}\n`,
    'utf8',
  );
  await writeFile(
    resolveLayoutCatalogPath(catalogRoot, 'index.md'),
    renderIndex(manifest),
    'utf8',
  );
}

async function waitForImportedAsset(
  statusPanel: Locator,
  displayName: string,
  nodeCount: number,
  timeoutMs = 300_000,
): Promise<void> {
  const expectedNodeCount = `${nodeCount} nodes`;
  await waitFor(
    async () => {
      const snapshot = await readableStatus(statusPanel);
      if (!snapshot) return false;
      const { level, status } = snapshot;
      if (level === 'error') return true;
      if (level === null || level === 'working' || level === 'idle')
        return false;
      return status.includes(displayName) && status.includes(expectedNodeCount);
    },
    `completed ZUI import status for ${displayName}`,
    timeoutMs,
  );
}

async function readableStatus(
  statusPanel: Locator,
): Promise<{ level: string | null; status: string } | null> {
  try {
    const [level, status] = await Promise.all([
      statusPanel.getAttribute('data-level', { timeout: 5_000 }),
      statusPanel.innerText({ timeout: 5_000 }),
    ]);
    return { level, status: normalizeText(status) };
  } catch (error) {
    if (errorMessage(error).includes('Timeout')) return null;
    throw error;
  }
}

async function requiredAttribute(
  locator: Locator,
  attribute: string,
  label: string,
): Promise<string> {
  let value = '';
  await waitFor(async () => {
    value = (await locator.getAttribute(attribute))?.trim() ?? '';
    return value.length > 0;
  }, label);
  return value;
}

async function waitFor(
  predicate: () => boolean | Promise<boolean>,
  label: string,
  timeoutMs = 30_000,
  signal?: AbortSignal,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (!(await predicate())) {
    if (signal?.aborted) throw new Error(`Cancelled waiting for ${label}`);
    if (Date.now() >= deadline)
      throw new Error(`Timed out waiting for ${label}`);
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 50));
  }
}

async function waitForExportError(
  statusPanel: Locator,
  timeoutMs = 30_000,
  signal?: AbortSignal,
): Promise<never> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() <= deadline) {
    if (signal?.aborted)
      throw new Error('Cancelled waiting for ZUI export status');
    try {
      // A large retained projection can block the iframe's main thread for
      // minutes.  A long locator timeout here would win the race and turn a
      // successful export into a false failure, so treat it as a transient
      // unreadable sample and poll again.
      const level = await statusPanel.getAttribute('data-level', {
        timeout: 1_000,
      });
      if (level === 'error') {
        const message = normalizeText(
          await statusPanel.innerText({ timeout: 1_000 }).catch(() => ''),
        );
        throw new Error(
          `ZUI export reported an error: ${message || '<empty>'}`,
        );
      }
    } catch (error) {
      if (
        error instanceof Error &&
        /Timeout|detached|closed/i.test(error.message)
      ) {
        await new Promise((resolvePromise) => setTimeout(resolvePromise, 100));
        continue;
      }
      throw error;
    }
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 100));
  }
  throw new Error('Timed out waiting for ZUI export status');
}

function numberOption(args: string[], name: string, fallback: number): number {
  const index = args.indexOf(name);
  if (index < 0) return fallback;
  const value = Number(args[index + 1]);
  if (!Number.isInteger(value) || value < 0)
    throw new Error(`${name} must be a non-negative integer`);
  return value;
}

function normalizeText(value: string): string {
  return value.replace(/\s+/g, ' ').trim();
}

function sha256(value: Buffer): string {
  return createHash('sha256').update(value).digest('hex');
}

function errorMessage(error: unknown): string {
  if (error instanceof AggregateError)
    return `${error.stack ?? error.message}\n${error.errors.map(errorMessage).join('\n')}`;
  return error instanceof Error
    ? (error.stack ?? error.message)
    : String(error);
}

void main().catch((error: unknown) => {
  console.error(compactErrorMessage(error));
  process.exitCode = 1;
});

function compactErrorMessage(error: unknown, maxLength = 1600): string {
  const message = errorMessage(error);
  return message.length > maxLength
    ? `${message.slice(0, maxLength)}... [truncated]`
    : message;
}
