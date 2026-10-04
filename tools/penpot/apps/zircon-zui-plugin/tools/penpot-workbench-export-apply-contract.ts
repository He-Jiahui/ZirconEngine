import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync } from 'node:fs';
import { mkdir, readFile, realpath, writeFile } from 'node:fs/promises';
import { dirname, isAbsolute, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium, type FrameLocator, type Page } from 'playwright';
import { applyZuiExportFileToSourceRoot } from '../src/bridge/zui-export-apply-files.js';
import {
  cloneZuiDocument,
  normalizeZuiDocument,
  parseZuiDocument,
  serializeZuiDocument,
  zuiNodes,
} from '../src/bridge/zui-document.js';
import type { LayoutReviewCaseMessage } from '../src/model';
import { clickAndWaitForDownload, downloadPath } from './zui-layout-download';
import { editWorkbenchSpacingInPenpot } from './penpot-workbench-canvas-edit';
import { LayoutDependencies } from './zui-layout-dependencies';
import { prepareComponentReviewHost } from './zui-layout-component-hosts';
import {
  openPenpotSession,
  recordPenpotSessionFailure,
  resolvePluginBundlePath,
  resolvePluginDistRoot,
} from './zui-layout-penpot-session';
import { requireApprovedPenpotArtifactDirectory } from './zui-layout-artifact-path';
import { captureProgramFingerprints } from './zui-layout-capture-provenance';
import type { LayoutReviewCase } from './zui-layout-review-contract';

const appRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const penpotRoot = resolve(appRoot, '../../../../third_party/penpot');
const repoRoot = resolve(process.env['ZUI_LAYOUT_REPO_ROOT'] ?? resolve(penpotRoot, '../..'));
const catalogPath = resolve(repoRoot, 'docs/_data/layout/catalog.json');
const mainFrameSourcePath =
  'zircon_editor/assets/ui/editor/host/editor_main_frame.zui';
const workbenchSourcePath =
  'zircon_editor/assets/ui/editor/windows/workbench_window.zui';
const workbenchShellSourcePath =
  'zircon_editor/assets/ui/editor/host/workbench_shell.zui';
const workbenchTabSourcePath =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_tab.zui';
const workbenchNodeId = 'window_content';
const workbenchGapPath = 'nodes.window_content.layout.container.gap';
const themeSourcePath =
  'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
const penpotBaseUrl = new URL(
  process.env['PENPOT_BASE_URL'] ?? 'https://design.penpot.app',
).origin;
const importTimeoutMs = 300_000;
const exportTimeoutMs = 300_000;
const renderOnly = process.argv.includes('--render-only');
const authoredOnly = process.argv.includes('--authored-only');

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

interface CatalogEntryForCapture {
  sourcePath: string;
  sourceSha256?: string;
  cases?: LayoutReviewCase[];
}

interface LayoutCatalogForCapture {
  entries: CatalogEntryForCapture[];
}

interface ExportSourceFingerprint {
  sourcePath: string;
  sha256: string;
}

async function main(): Promise<void> {
  assert.ok(
    !(renderOnly && authoredOnly),
    'Choose either --render-only or --authored-only.',
  );
  if (authoredOnly) {
    await runAuthoredOnlyDiagnostic();
    return;
  }
  const artifactRoot = process.env['ZIRCON_PENPOT_ARTIFACT_ROOT'];
  assert.ok(
    artifactRoot,
    'Set ZIRCON_PENPOT_ARTIFACT_ROOT to an existing directory below D:, E:, or F: drive-root cargo-targets.',
  );
  const approvedArtifactRoot = requireApprovedPenpotArtifactDirectory(
    artifactRoot,
    'ZIRCON_PENPOT_ARTIFACT_ROOT',
  );
  const artifactDirectory = resolve(
    approvedArtifactRoot,
    'penpot-workbench-export-apply',
  );
  const appliedSourceRoot = resolve(artifactDirectory, 'applied-sources');
  await Promise.all([
    mkdir(artifactDirectory, { recursive: true }),
    mkdir(appliedSourceRoot, { recursive: true }),
  ]);

  const catalog = JSON.parse(
    await readFile(catalogPath, 'utf8'),
  ) as LayoutCatalogForCapture;
  const workbenchEntry = catalog.entries.find(
    (entry) => entry.sourcePath === workbenchSourcePath,
  );
  assert.ok(workbenchEntry, `Missing catalog source ${workbenchSourcePath}.`);
  const reviewCase = workbenchEntry.cases?.find(
    (candidate) =>
      candidate.id === 'default-1280x800-dpi1' &&
      candidate.sourcePath === workbenchSourcePath,
  );
  assert.ok(
    reviewCase,
    'The catalog must contain the default 1280x800 WorkbenchWindow case.',
  );
  assertWorkbenchPresentation(reviewCase);

  const canonicalSource = await readFile(
    resolve(repoRoot, workbenchSourcePath),
    'utf8',
  );
  const canonicalDocument = parseZuiDocument(canonicalSource).document;
  const catalogSources = [
    ...new Set([
      ...catalog.entries.map((entry) => entry.sourcePath),
      mainFrameSourcePath,
      workbenchSourcePath,
      workbenchShellSourcePath,
      workbenchTabSourcePath,
      themeSourcePath,
    ]),
  ];
  const dependencies = new LayoutDependencies();
  await dependencies.load(repoRoot, catalogSources);
  const componentHost = await prepareComponentReviewHost(
    repoRoot,
    workbenchSourcePath,
    canonicalDocument,
    dependencies,
    reviewCase.themeSourcePath ?? themeSourcePath,
    reviewCase.data['workbenchPresentation'] as
      Record<string, unknown> | undefined,
  );
  assert.ok(
    componentHost,
    'WorkbenchWindow did not produce its authored EditorMainFrame and WorkbenchShell review host.',
  );

  const penpotInput = cloneZuiDocument(canonicalDocument);
  penpotInput['penpot_review_host'] = componentHost.projection;
  penpotInput['penpot_dependency_sources'] = [
    ...new Set([
      ...componentHost.consumers.map((item) => item.sourcePath),
      ...((componentHost.projection['penpot_dependency_sources'] as
        string[] | undefined) ?? []),
    ]),
  ];
  const expectedSources = await sourceFingerprintsForHost(
    componentHost.projection,
    componentHost.consumers.map((item) => item.sourcePath),
  );
  assert.ok(
    expectedSources.some((item) => item.sourcePath === workbenchSourcePath),
    'The composed host source manifest must include the canonical WorkbenchWindow owner.',
  );

  const expectedCanonicalSha256 = expectedSources.find(
    (item) => item.sourcePath === workbenchSourcePath,
  )!.sha256;
  assert.equal(
    expectedCanonicalSha256,
    sha256(canonicalSource),
    'The source-root export baseline must match the captured canonical WorkbenchWindow bytes.',
  );
  penpotInput['penpot_original_source'] = canonicalSource;
  penpotInput['penpot_original_source_path'] = workbenchSourcePath;
  penpotInput['penpot_source_fingerprints'] = expectedSources;
  const penpotInputSource = serializeZuiDocument(penpotInput);
  const [compiledPluginBundle, distributionPluginBundle] = await Promise.all([
    readFile(resolvePluginBundlePath()),
    readFile(resolve(resolvePluginDistRoot(), 'assets/plugin.js')),
  ]);
  const pluginBundleSha256 = sha256(distributionPluginBundle);
  assert.equal(
    pluginBundleSha256,
    sha256(compiledPluginBundle),
    'The staged Penpot distribution must contain the current external ZUI plugin bundle.',
  );
  const captureFingerprints = await captureProgramFingerprints(repoRoot);
  const pageErrors: string[] = [];
  const unexpectedRpcs = new Set<string>();
  const browser = await chromium.launch({
    headless: true,
    downloadsPath: artifactDirectory,
    args: ['--ignore-certificate-errors'],
    ...(browserExecutable ? { executablePath: browserExecutable } : {}),
  });
  let runtime: Awaited<ReturnType<typeof openPenpotSession>> | undefined;

  try {
    runtime = await openPenpotSession(browser, {
      repoRoot,
      catalogRoot: artifactDirectory,
      writeEvidence: false,
      penpotBaseUrl,
      pluginBundleSha256,
      pageErrors,
      unexpectedRpcs,
      captureProgramFingerprints: captureFingerprints,
      captureBrowserRuntime,
    });
    await runtime.page.setViewportSize({ width: 1280, height: 800 });
    await runtime.fileInput.setInputFiles({
      name: 'workbench_window.zui',
      mimeType: 'text/plain',
      buffer: Buffer.from(penpotInputSource, 'utf8'),
    });
    await waitForWorkbenchImport(
      runtime.statusPanel,
      canonicalDocument.asset.display_name ?? 'Workbench Window',
    );
    const browserRuntime = await captureBrowserRuntime(runtime.page);
    await setReviewCase(runtime.pluginFrame, reviewCase);
    await runtime.pluginFrame.locator('body').evaluate(() => {
      parent.postMessage(
        { type: 'set-preview-selection', selected: true },
        '*',
      );
    });
    await waitFor(
      async () => runtime!.exportButton.isEnabled(),
      'selected full-workbench ZUI export',
      importTimeoutMs,
    );

    const previewBoardId = await requiredAttribute(
      runtime.statusPanel,
      'data-preview-board-id',
      'composed Workbench preview board id',
    );
    const hostScreenshotPath = resolve(
      artifactDirectory,
      'editor_main_frame.full-workbench.penpot.png',
    );
    await runtime.page.screenshot({
      path: hostScreenshotPath,
      fullPage: true,
      timeout: exportTimeoutMs,
    });

    if (renderOnly) {
      const noEditDownload = await exportSelected(runtime);
      const noEditExport = await readFile(noEditDownload, 'utf8');
      assert.equal(
        noEditExport,
        canonicalSource,
        'An untouched full-workbench Penpot export must preserve the exact canonical WorkbenchWindow source.',
      );
      const noEditExportPath = resolve(
        artifactDirectory,
        'workbench_window.no-edit-penpot-export.zui',
      );
      await writeFile(noEditExportPath, noEditExport, 'utf8');
      const fonts = await runtime.fontResources.snapshotRecords();
      assert.deepEqual([...unexpectedRpcs], []);
      assert.deepEqual(pageErrors, []);
      const summary = {
        contract: 'penpot-full-workbench-render-diagnostic',
        status: 'pending',
        pendingReason:
          'Render-only diagnostic captured the full EditorMainFrame product host; no Penpot edit or source-root export/apply was attempted.',
        sourcePath: workbenchSourcePath,
        reviewHostSourcePath: mainFrameSourcePath,
        sourceOwners: expectedSources,
        canonicalSha256: expectedCanonicalSha256,
        penpotFrontend: penpotBaseUrl,
        backend: 'repository-mock',
        browserRuntime,
        previewBoardId,
        workbenchPresentation: reviewCase.data['workbenchPresentation'],
        hostScreenshotPath,
        noEditExportPath,
        fonts,
        captureFingerprints,
        persistenceVerified: false,
      };
      const summaryPath = resolve(
        artifactDirectory,
        'penpot-full-workbench-render-diagnostic.json',
      );
      await writeFile(
        summaryPath,
        `${JSON.stringify(summary, null, 2)}\n`,
        'utf8',
      );
      console.log(JSON.stringify({ ...summary, summaryPath }));
      return;
    }

    const noEditDownload = await exportSelected(runtime);
    const noEditExport = await readFile(noEditDownload, 'utf8');
    assert.equal(
      noEditExport,
      canonicalSource,
      'An untouched full-workbench Penpot export must preserve the exact canonical WorkbenchWindow source.',
    );
    const noEditExportPath = resolve(
      artifactDirectory,
      'workbench_window.no-edit-penpot-export.zui',
    );
    await writeFile(noEditExportPath, noEditExport, 'utf8');

    const edit = await editWorkbenchSpacingInPenpot(runtime);
    const editedHostScreenshotPath = resolve(
      artifactDirectory,
      'workbench_window.full-workbench-gap-edited.penpot.png',
    );
    await runtime.page.screenshot({
      path: editedHostScreenshotPath,
      fullPage: true,
      timeout: exportTimeoutMs,
    });
    const editedDownload = await exportSelected(runtime);
    const editedExportPath = resolve(
      artifactDirectory,
      'workbench_window.full-workbench-gap-edited.penpot-export.zui',
    );
    await writeFile(editedExportPath, await readFile(editedDownload));

    const applied = await applyZuiExportFileToSourceRoot({
      exportedPath: editedExportPath,
      sourceRootPath: repoRoot,
      outputRootPath: appliedSourceRoot,
      expectedSources,
      expectedRootSha256: expectedCanonicalSha256,
    });
    assert.deepEqual(
      applied.appliedPathsBySource[workbenchSourcePath],
      [workbenchGapPath],
      'The full-host export must route only the edited WorkbenchWindow spacing field to its canonical source owner.',
    );
    const appliedWorkbenchPath = resolve(
      appliedSourceRoot,
      workbenchSourcePath,
    );
    const appliedWorkbenchSource = await readFile(appliedWorkbenchPath, 'utf8');
    const appliedWorkbench = parseZuiDocument(appliedWorkbenchSource).document;
    const expectedWorkbench = parseZuiDocument(canonicalSource).document;
    setWorkbenchLayoutGap(expectedWorkbench, edit.after.rowGap);
    assert.deepEqual(
      normalizeZuiDocument(appliedWorkbench),
      normalizeZuiDocument(expectedWorkbench),
      'Export/apply must change the source-owned WorkbenchWindow gap and preserve its remaining canonical fields.',
    );

    const fonts = await runtime.fontResources.snapshotRecords();
    assert.deepEqual([...unexpectedRpcs], []);
    assert.deepEqual(pageErrors, []);

    const summary = {
      contract: 'penpot-full-workbench-export-apply',
      sourcePath: workbenchSourcePath,
      reviewHostSourcePath: mainFrameSourcePath,
      sourceOwners: expectedSources,
      canonicalEditTarget: workbenchSourcePath,
      canonicalSha256: expectedCanonicalSha256,
      penpotFrontend: penpotBaseUrl,
      backend: 'repository-mock',
      browserRuntime,
      previewBoardId,
      workbenchPresentation: reviewCase.data['workbenchPresentation'],
      hostScreenshotPath,
      editedHostScreenshotPath,
      noEditExportPath,
      editedExportPath,
      editSource: 'penpot-plugin-shape-api',
      edit,
      exportApplyRoot: appliedSourceRoot,
      appliedWorkbenchPath,
      appliedPathsBySource: applied.appliedPathsBySource,
      fonts,
      captureFingerprints,
      persistenceVerified: false,
    };
    const summaryPath = resolve(
      artifactDirectory,
      'penpot-full-workbench-export-apply-contract.json',
    );
    await writeFile(
      summaryPath,
      `${JSON.stringify(summary, null, 2)}\n`,
      'utf8',
    );
    console.log(JSON.stringify({ ...summary, summaryPath }));
  } catch (error) {
    if (runtime)
      await recordPenpotSessionFailure(runtime, artifactDirectory, error);
    throw error;
  } finally {
    runtime?.fontResources.stop();
    await runtime?.context.close();
    await browser.close();
  }
}

async function runAuthoredOnlyDiagnostic(): Promise<void> {
  const artifactRoot = process.env['ZIRCON_PENPOT_ARTIFACT_ROOT'];
  assert.ok(
    artifactRoot,
    'Set ZIRCON_PENPOT_ARTIFACT_ROOT to an existing directory below D:, E:, or F: drive-root cargo-targets.',
  );
  const approvedArtifactRoot = requireApprovedPenpotArtifactDirectory(
    artifactRoot,
    'ZIRCON_PENPOT_ARTIFACT_ROOT',
  );
  const artifactDirectory = resolve(
    approvedArtifactRoot,
    'penpot-workbench-authored-only-diagnostic',
  );
  await mkdir(artifactDirectory, { recursive: true });

  const catalog = JSON.parse(
    await readFile(catalogPath, 'utf8'),
  ) as LayoutCatalogForCapture;
  const workbenchEntry = catalog.entries.find(
    (entry) => entry.sourcePath === workbenchSourcePath,
  );
  assert.ok(workbenchEntry, `Missing catalog source ${workbenchSourcePath}.`);
  const reviewCase = workbenchEntry.cases?.find(
    (candidate) =>
      candidate.id === 'default-1280x800-dpi1' &&
      candidate.sourcePath === workbenchSourcePath,
  );
  assert.ok(
    reviewCase,
    'The catalog must contain the default 1280x800 WorkbenchWindow case.',
  );

  const canonicalSource = await readFile(
    resolve(repoRoot, workbenchSourcePath),
    'utf8',
  );
  const canonicalDocument = parseZuiDocument(canonicalSource).document;
  const themePath = reviewCase.themeSourcePath ?? themeSourcePath;
  const dependencyPaths = [
    ...new Set([
      ...catalog.entries.map((entry) => entry.sourcePath),
      workbenchSourcePath,
      themePath,
    ]),
  ];
  const dependencies = new LayoutDependencies();
  await dependencies.load(repoRoot, dependencyPaths);
  const penpotInput = cloneZuiDocument(canonicalDocument);
  dependencies.embed(penpotInput, workbenchSourcePath, themePath);
  const dependencySources = penpotInput['penpot_dependency_sources'];
  assert.ok(
    Array.isArray(dependencySources) &&
      dependencySources.every((sourcePath) => typeof sourcePath === 'string'),
    'The prepared authored document must contain its resolved dependency paths.',
  );
  const expectedSources = await sourceFingerprintsForPaths([
    workbenchSourcePath,
    ...dependencySources.filter((path) => path.endsWith('.zui')),
  ]);
  const sourceClosure = await authoredSourceClosure(
    penpotInput,
    catalog,
    workbenchSourcePath,
  );
  const sourceClosurePath = resolve(
    artifactDirectory,
    'source-closure-manifest.json',
  );
  await writeFile(
    sourceClosurePath,
    `${JSON.stringify(sourceClosure, null, 2)}\n`,
    'utf8',
  );
  penpotInput['penpot_original_source'] = canonicalSource;
  penpotInput['penpot_original_source_path'] = workbenchSourcePath;
  penpotInput['penpot_source_fingerprints'] = expectedSources;
  const penpotInputSource = serializeZuiDocument(penpotInput);
  const [compiledPluginBundle, distributionPluginBundle] = await Promise.all([
    readFile(resolvePluginBundlePath()),
    readFile(resolve(resolvePluginDistRoot(), 'assets/plugin.js')),
  ]);
  const pluginBundleSha256 = sha256(distributionPluginBundle);
  assert.equal(
    pluginBundleSha256,
    sha256(compiledPluginBundle),
    'The staged Penpot distribution must contain the current external ZUI plugin bundle.',
  );
  const captureFingerprints = await captureProgramFingerprints(repoRoot);
  const pageErrors: string[] = [];
  const unexpectedRpcs = new Set<string>();
  const browser = await chromium.launch({
    headless: true,
    downloadsPath: artifactDirectory,
    args: ['--ignore-certificate-errors'],
    ...(browserExecutable ? { executablePath: browserExecutable } : {}),
  });
  let runtime: Awaited<ReturnType<typeof openPenpotSession>> | undefined;
  try {
    runtime = await openPenpotSession(browser, {
      repoRoot,
      catalogRoot: artifactDirectory,
      writeEvidence: false,
      penpotBaseUrl,
      pluginBundleSha256,
      pageErrors,
      unexpectedRpcs,
      captureProgramFingerprints: captureFingerprints,
      captureBrowserRuntime,
    });
    await runtime.page.setViewportSize({ width: 1280, height: 800 });
    await requestAuthoredOnlyMaterialization(runtime);
    await runtime.fileInput.setInputFiles({
      name: 'workbench_window.authored-only.zui',
      mimeType: 'text/plain',
      buffer: Buffer.from(penpotInputSource, 'utf8'),
    });
    await waitForWorkbenchImport(
      runtime.statusPanel,
      canonicalDocument.asset.display_name ?? 'Workbench Window',
    );
    const browserRuntime = await captureBrowserRuntime(runtime.page);
    await setReviewCase(runtime.pluginFrame, reviewCase);
    await runtime.pluginFrame.locator('body').evaluate(() => {
      parent.postMessage(
        { type: 'set-preview-selection', selected: true },
        '*',
      );
    });
    await waitFor(
      async () => runtime!.exportButton.isEnabled(),
      'selected authored WorkbenchWindow export',
      importTimeoutMs,
    );
    const previewBoardId = await requiredAttribute(
      runtime.statusPanel,
      'data-preview-board-id',
      'authored WorkbenchWindow preview board id',
    );
    const hostScreenshotPath = resolve(
      artifactDirectory,
      'workbench_window.authored-only.penpot.png',
    );
    await runtime.page.screenshot({
      path: hostScreenshotPath,
      fullPage: true,
      timeout: exportTimeoutMs,
    });
    const noEditDownload = await exportSelected(runtime);
    const noEditExport = await readFile(noEditDownload, 'utf8');
    assert.equal(
      noEditExport,
      canonicalSource,
      'An untouched authored WorkbenchWindow export must preserve exact canonical source bytes.',
    );
    const noEditExportPath = resolve(
      artifactDirectory,
      'workbench_window.authored-only.no-edit-export.zui',
    );
    await writeFile(noEditExportPath, noEditExport, 'utf8');
    const fonts = await runtime.fontResources.snapshotRecords();
    assert.deepEqual([...unexpectedRpcs], []);
    assert.deepEqual(pageErrors, []);
    const summary = {
      contract: 'penpot-authored-workbench-window-diagnostic',
      status: 'pending',
      pendingReason:
        'This capture renders only the authored WorkbenchWindow source. The product workbenchPresentation snapshot and composed EditorMainFrame plus WorkbenchShell host are not available in the catalog, so full-host parity remains pending.',
      sourcePath: workbenchSourcePath,
      reviewHostSourcePath: null,
      sourceOwners: expectedSources,
      sourceClosurePath,
      sourceClosureStatus: sourceClosure.status,
      canonicalSha256: sha256(canonicalSource),
      penpotFrontend: penpotBaseUrl,
      backend: 'repository-mock',
      browserRuntime,
      previewBoardId,
      workbenchPresentation: null,
      nativeComponentMode: 'semantic',
      nativeComponentNote:
        'All authored semantic nodes are retained for this design-only diagnostic; Penpot native-library promotion is deferred.',
      hostScreenshotPath,
      noEditExportPath,
      fonts,
      captureFingerprints,
      persistenceVerified: false,
    };
    const summaryPath = resolve(
      artifactDirectory,
      'penpot-workbench-authored-only-diagnostic.json',
    );
    await writeFile(
      summaryPath,
      `${JSON.stringify(summary, null, 2)}\n`,
      'utf8',
    );
    console.log(JSON.stringify({ ...summary, summaryPath }));
  } catch (error) {
    if (runtime) {
      await recordPenpotSessionFailure(runtime, artifactDirectory, error);
    } else {
      const startupScreenshotPath = resolve(
        artifactDirectory,
        'penpot-session-startup-failure.png',
      );
      const startupDiagnosticPath = resolve(
        artifactDirectory,
        'penpot-session-startup-failure.json',
      );
      const summaryPath = resolve(
        artifactDirectory,
        'penpot-workbench-authored-only-diagnostic.json',
      );
      const summary = {
        contract: 'penpot-authored-workbench-window-diagnostic',
        status: 'pending',
        captureStage: 'penpot-frontend-startup',
        pendingReason:
          'The official Penpot frontend did not finish loading its application bundle, so the canonical WorkbenchWindow was not imported or rendered.',
        startupError: error instanceof Error ? error.message : String(error),
        sourcePath: workbenchSourcePath,
        canonicalSha256: sha256(canonicalSource),
        sourceOwners: expectedSources,
        sourceClosurePath,
        sourceClosureStatus: sourceClosure.status,
        pluginBundleSha256,
        penpotFrontend: penpotBaseUrl,
        backend: 'repository-mock',
        workbenchPresentation: null,
        hostScreenshotPath: null,
        startupScreenshotPath: existsSync(startupScreenshotPath)
          ? startupScreenshotPath
          : null,
        startupDiagnosticPath: existsSync(startupDiagnosticPath)
          ? startupDiagnosticPath
          : null,
        persistenceVerified: false,
      };
      await writeFile(
        summaryPath,
        `${JSON.stringify(summary, null, 2)}\n`,
        'utf8',
      );
      console.log(JSON.stringify({ ...summary, summaryPath }));
    }
    throw error;
  } finally {
    runtime?.fontResources.stop();
    await runtime?.context.close();
    await browser.close();
  }
}

async function captureBrowserRuntime(page: Page): Promise<{
  product: string;
  userAgent: string;
  executablePath: string;
  executableSha256: string;
}> {
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

function assertWorkbenchPresentation(reviewCase: LayoutReviewCase): void {
  const presentation = reviewCase.data['workbenchPresentation'];
  assert.ok(
    typeof presentation === 'object' &&
      presentation !== null &&
      !Array.isArray(presentation),
    'The current WorkbenchWindow review case must carry data.workbenchPresentation.',
  );
  const value = presentation as Record<string, unknown>;
  assert.equal(value['schema'], 'dev.zircon.editor.workbench-presentation');
  assert.equal(value['version'], 1);
  assert.match(
    String(value['stateFingerprint'] ?? ''),
    /^[0-9a-f]{64}$/i,
    'The product presentation snapshot must have a canonical state fingerprint.',
  );
  assert.ok(
    typeof value['layout'] === 'object' &&
      value['layout'] !== null &&
      !Array.isArray(value['layout']),
    'The product presentation snapshot must include the serialized current WorkbenchLayout.',
  );
}

async function sourceFingerprintsForHost(
  projection: ReturnType<typeof cloneZuiDocument>,
  consumerPaths: string[],
): Promise<ExportSourceFingerprint[]> {
  const dependencies = projection['penpot_dependency_sources'];
  assert.ok(
    Array.isArray(dependencies) &&
      dependencies.every((item) => typeof item === 'string'),
    'The composed host must list its imported design sources.',
  );
  const paths = [
    mainFrameSourcePath,
    workbenchSourcePath,
    ...consumerPaths,
    ...dependencies.filter((path) => path.endsWith('.zui')),
  ];
  return sourceFingerprintsForPaths(paths);
}

async function authoredSourceClosure(
  document: ReturnType<typeof cloneZuiDocument>,
  catalog: LayoutCatalogForCapture,
  rootSourcePath: string,
): Promise<{
  schema: 'dev.zircon.penpot.authored-source-closure';
  version: 1;
  status: 'complete' | 'incomplete';
  catalogCoverageComplete: boolean;
  preparedFrom: 'penpot_dependency_sources plus materialized SVG references';
  rootSourcePath: string;
  rootSha256: string;
  dependencySourcePaths: string[];
  materializedSvgSources: Array<{
    resourceUri: string;
    ownerSourcePath: string;
    sourcePath: string;
  }>;
  files: Array<{
    sourcePath: string;
    physicalPath: string | null;
    sha256: string | null;
    sourceKind: 'root-zui' | 'dependency-zui' | 'dependency-media';
    catalogSha256: string | null;
    catalogMatches: boolean | null;
  }>;
  unresolvedSources: Array<{ sourcePath: string; reason: string }>;
}> {
  const dependencySources = document['penpot_dependency_sources'];
  assert.ok(
    Array.isArray(dependencySources) &&
      dependencySources.every((sourcePath) => typeof sourcePath === 'string'),
    'The prepared authored document must contain its resolved dependency paths.',
  );
  const dependencies = [...new Set(dependencySources)];
  const materializedSvgSources: Array<{
    resourceUri: string;
    ownerSourcePath: string;
    sourcePath: string;
  }> = [];
  for (const node of Object.values(zuiNodes(document))) {
    const resourceUri =
      node.props?.['icon'] ??
      (node.component.toLowerCase().includes('icon')
        ? node.props?.['source']
        : undefined);
    if (typeof resourceUri !== 'string' || !resourceUri.endsWith('.svg'))
      continue;
    const ownerSourcePath =
      typeof node['penpot_review_source_path'] === 'string' &&
      node['penpot_review_source_path']
        ? node['penpot_review_source_path']
        : rootSourcePath;
    const owner = ownerSourcePath.split('/assets/')[0];
    const locator = resourceUri.replace(/^res:\/\/(icons\/)?/, '');
    const absolutePath = resolve(repoRoot, owner, 'assets', 'icons', locator);
    const sourcePath = relative(repoRoot, absolutePath).replaceAll('\\', '/');
    materializedSvgSources.push({ resourceUri, ownerSourcePath, sourcePath });
    if (!dependencies.includes(sourcePath)) dependencies.push(sourcePath);
  }

  const sourcePaths = [...new Set([rootSourcePath, ...dependencies])].sort(
    (left, right) => left.localeCompare(right),
  );
  const unresolvedSources: Array<{ sourcePath: string; reason: string }> = [];
  const files = await Promise.all(
    sourcePaths.map(async (sourcePath) => {
      const absolutePath = resolve(repoRoot, sourcePath);
      const repoRelative = relative(repoRoot, absolutePath);
      if (
        isAbsolute(sourcePath) ||
        repoRelative === '..' ||
        repoRelative.startsWith(`..${sep}`) ||
        isAbsolute(repoRelative)
      ) {
        unresolvedSources.push({
          sourcePath,
          reason: 'source path resolves outside the repository root',
        });
        return {
          sourcePath,
          physicalPath: null,
          sha256: null,
          sourceKind:
            sourcePath === rootSourcePath
              ? ('root-zui' as const)
              : sourcePath.endsWith('.zui')
                ? ('dependency-zui' as const)
                : ('dependency-media' as const),
          catalogSha256: null,
          catalogMatches: null,
        };
      }
      try {
        const [bytes, physicalPath] = await Promise.all([
          readFile(absolutePath),
          realpath(absolutePath),
        ]);
        const physicalRelative = relative(repoRoot, physicalPath);
        if (
          physicalRelative === '..' ||
          physicalRelative.startsWith(`..${sep}`) ||
          isAbsolute(physicalRelative)
        )
          throw new Error('source resolves outside the physical repository');
        const sourceSha256 = sha256(bytes);
        const catalogSha256 =
          catalog.entries.find((entry) => entry.sourcePath === sourcePath)
            ?.sourceSha256 ?? null;
        return {
          sourcePath,
          physicalPath,
          sha256: sourceSha256,
          sourceKind:
            sourcePath === rootSourcePath
              ? ('root-zui' as const)
              : sourcePath.endsWith('.zui')
                ? ('dependency-zui' as const)
                : ('dependency-media' as const),
          catalogSha256,
          catalogMatches:
            catalogSha256 === null ? null : catalogSha256 === sourceSha256,
        };
      } catch (error) {
        unresolvedSources.push({
          sourcePath,
          reason: error instanceof Error ? error.message : String(error),
        });
        return {
          sourcePath,
          physicalPath: null,
          sha256: null,
          sourceKind:
            sourcePath === rootSourcePath
              ? ('root-zui' as const)
              : sourcePath.endsWith('.zui')
                ? ('dependency-zui' as const)
                : ('dependency-media' as const),
          catalogSha256: null,
          catalogMatches: null,
        };
      }
    }),
  );
  const root = files.find((file) => file.sourcePath === rootSourcePath);
  assert.ok(
    root?.sha256,
    `Could not fingerprint root source ${rootSourcePath}.`,
  );
  const zuiFiles = files.filter((file) => file.sourcePath.endsWith('.zui'));
  return {
    schema: 'dev.zircon.penpot.authored-source-closure',
    version: 1,
    status: unresolvedSources.length === 0 ? 'complete' : 'incomplete',
    catalogCoverageComplete: zuiFiles.every(
      (file) => file.catalogMatches === true,
    ),
    preparedFrom: 'penpot_dependency_sources plus materialized SVG references',
    rootSourcePath,
    rootSha256: root.sha256,
    dependencySourcePaths: dependencies.sort((left, right) =>
      left.localeCompare(right),
    ),
    materializedSvgSources,
    files,
    unresolvedSources,
  };
}

async function sourceFingerprintsForPaths(
  paths: string[],
): Promise<ExportSourceFingerprint[]> {
  const uniquePaths = [...new Set(paths)].sort((left, right) =>
    left.localeCompare(right),
  );
  return await Promise.all(
    uniquePaths.map(async (sourcePath) => ({
      sourcePath,
      sha256: sha256(await readFile(resolve(repoRoot, sourcePath))),
    })),
  );
}

async function setReviewCase(
  pluginFrame: FrameLocator,
  reviewCase: LayoutReviewCaseMessage,
): Promise<void> {
  await pluginFrame.locator('body').evaluate((_, value) => {
    parent.postMessage({ type: 'set-review-case', reviewCase: value }, '*');
  }, reviewCase);
  const deadline = Date.now() + importTimeoutMs;
  let lastStatus = '';
  while (Date.now() <= deadline) {
    const [reviewCaseId, level, message] = await Promise.all([
      pluginFrame
        .locator('.status-panel')
        .getAttribute('data-review-case-id', { timeout: 1_000 })
        .catch(() => null),
      pluginFrame
        .locator('.status-panel')
        .getAttribute('data-level', { timeout: 1_000 })
        .catch(() => null),
      pluginFrame
        .locator('.status-panel')
        .innerText({ timeout: 1_000 })
        .catch(() => ''),
    ]);
    lastStatus = message.replace(/\s+/g, ' ').trim();
    if (level === 'error')
      throw new Error(lastStatus || `Review case failed: ${reviewCase.id}.`);
    if (reviewCaseId === reviewCase.id && level !== 'working') return;
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 100));
  }
  throw new Error(
    `Review case acknowledgement timed out: ${reviewCase.id}; last status: ${lastStatus || '(unavailable)'}`,
  );
}

async function exportSelected(
  runtime: Awaited<ReturnType<typeof openPenpotSession>>,
): Promise<string> {
  const download = await clickAndWaitForDownload(
    () => runtime.page.waitForEvent('download', { timeout: exportTimeoutMs }),
    () => runtime.exportButton.click({ timeout: exportTimeoutMs }),
  );
  const path = await downloadPath(download, exportTimeoutMs);
  assert.ok(path, 'Penpot did not finish writing its ZUI export.');
  return path;
}

async function waitForWorkbenchImport(
  statusPanel: Awaited<ReturnType<typeof openPenpotSession>>['statusPanel'],
  displayName: string,
): Promise<void> {
  let status = '';
  let level: string | null = null;
  try {
    await waitFor(
      async () => {
        [level, status] = await Promise.all([
          statusPanel
            .getAttribute('data-level', { timeout: 1_000 })
            .catch(() => null),
          statusPanel.innerText({ timeout: 1_000 }).catch(() => ''),
        ]);
        status = status.replace(/\s+/g, ' ').trim();
        if (level === 'error')
          throw new Error(`Full-workbench Penpot import failed: ${status}`);
        return (
          (level === 'success' || level === 'warning') &&
          status.includes(displayName) &&
          /\b\d+ nodes\b/.test(status)
        );
      },
      `completed Penpot import for ${displayName}`,
      importTimeoutMs,
    );
  } catch (error) {
    throw new Error(
      `${error instanceof Error ? error.message : String(error)} Last import status: level=${level ?? '<unavailable>'}; message=${status || '<unavailable>'}.`,
      { cause: error },
    );
  }
}

async function requestAuthoredOnlyMaterialization(
  runtime: Awaited<ReturnType<typeof openPenpotSession>>,
): Promise<void> {
  await runtime.pluginFrame.locator('body').evaluate((_, mode) => {
    parent.postMessage({ type: 'set-review-materialization', mode }, '*');
  }, 'semantic' as const);
  await waitFor(
    async () => {
      const [level, status] = await Promise.all([
        runtime.statusPanel
          .getAttribute('data-level', { timeout: 1_000 })
          .catch(() => null),
        runtime.statusPanel.innerText({ timeout: 1_000 }).catch(() => ''),
      ]);
      return (
        level === 'idle' &&
        status.replace(/\s+/g, ' ').trim() ===
          'Review materialization · semantic'
      );
    },
    'semantic review materialization acknowledgement',
    10_000,
  );
}

async function waitFor(
  predicate: () => boolean | Promise<boolean>,
  label: string,
  timeoutMs: number,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (!(await predicate())) {
    if (Date.now() >= deadline)
      throw new Error(`Timed out waiting for ${label}.`);
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 50));
  }
}

async function requiredAttribute(
  locator: import('playwright').Locator,
  name: string,
  label: string,
): Promise<string> {
  const value = await locator.getAttribute(name);
  assert.ok(value, `Missing ${label}.`);
  return value;
}

function setWorkbenchLayoutGap(
  document: ReturnType<typeof parseZuiDocument>['document'],
  value: number,
): void {
  const node = zuiNodes(document)[workbenchNodeId];
  assert.ok(node, 'Canonical WorkbenchWindow lost window_content.');
  const layout = recordValue(node.layout, `${workbenchNodeId}.layout`);
  const container = recordValue(
    layout['container'],
    `${workbenchNodeId}.layout.container`,
  );
  container['gap'] = value;
}

function recordValue(value: unknown, label: string): Record<string, unknown> {
  assert.ok(
    typeof value === 'object' && value !== null && !Array.isArray(value),
    `${label} must be a table.`,
  );
  return value as Record<string, unknown>;
}

function sha256(value: Buffer | string): string {
  return createHash('sha256').update(value).digest('hex');
}

void main().catch((error: unknown) => {
  console.error(
    error instanceof Error ? (error.stack ?? error.message) : String(error),
  );
  process.exitCode = 1;
});
