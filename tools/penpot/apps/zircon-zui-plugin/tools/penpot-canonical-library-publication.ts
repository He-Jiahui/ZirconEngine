import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { chromium } from 'playwright';
import { cloneZuiDocument, decodeZuiMetadata, normalizeZuiDocument, parseZuiDocument, serializeZuiDocument } from '../src/bridge/zui-document';
import type { CanonicalComponentRecord } from '../src/penpot-canonical-library';
import { LayoutDependencies } from './zui-layout-dependencies';
import { trackedZuiPaths } from './zui-layout-catalog';
import { transitDecoder, rpc, openWorkspace, downloadActualCanvasPng, withSourceColorRevisionFence } from './zui-layout-real-session';
import { observeRealSaveHttp } from './zui-layout-real-save-observer';
import { assertExistingPenpotContentPreserved, assertNativeInstanceGeometryPreserved, persistedZuiMetadata, requireRealSessionDirectory, sha256, verifySaveResponse, workspaceIdentity } from './zui-layout-real-session-contract.mjs';
import { verifyCurrentPluginUi, verifyCurrentPluginWorkerBuild } from './zui-layout-real-ui.mjs';

export interface CanonicalLibraryPublicationOptions {
  repoRoot: string; output: string; workspaceUrl: string; storageState: string;
  dist: string; bundle: string; uiBuildReceipt: string; executablePath: string;
  workerBuildReceipt: string; workerBuildReceiptSha256: string;
  transitPackage: string; fontResources?: string; sourcePaths: string[];
  append?: { fileId: string; revn: number; vern: number };
  exportEvidence?: CanonicalPublicationExportEvidence;
  existingPlugin: { host: string; pluginId: string; registryEntrySha256: string; manifestSha256: string };
}

export type CanonicalPublicationExportEvidence = 'source-owner' | 'source-owner-and-png';

export function canonicalPublicationExportEvidence(
  requested: CanonicalPublicationExportEvidence | undefined,
): CanonicalPublicationExportEvidence {
  assert.ok(requested === undefined || requested === 'source-owner' || requested === 'source-owner-and-png',
    'Canonical publication exportEvidence must explicitly select source-owner or source-owner-and-png');
  return requested ?? 'source-owner-and-png';
}

/** PNG evidence is independent; opting out never bypasses the original source comparison. */
export async function collectCanonicalOwnerExportEvidence(
  original: Parameters<typeof normalizeZuiDocument>[0], exported: string,
  requested: CanonicalPublicationExportEvidence,
  capturePng: () => Promise<{pngPath: string; canvas: Record<string, unknown>}>,
): Promise<{sourceSemanticMatch: true; pngEvidence: 'not-requested' | 'captured';
  pngPath?: string; canvas?: Record<string, unknown>}> {
  const selected = canonicalPublicationExportEvidence(requested);
  assert.deepEqual(normalizeZuiDocument(parseZuiDocument(exported).document), normalizeZuiDocument(original));
  if (selected === 'source-owner') return {sourceSemanticMatch: true, pngEvidence: 'not-requested'};
  return {sourceSemanticMatch: true, pngEvidence: 'captured', ...await capturePng()};
}

/** An append may add mains; the previous source owners and styles stay intact. */
export function assertCanonicalLibraryPreserved(before: any, after: any): void {
  assertExistingPenpotContentPreserved(before, after);
}

/** Official editable import, actual save ACK, fresh-context readback and source export. */
export async function publishCanonicalLibrary(options: CanonicalLibraryPublicationOptions): Promise<void> {
  assert.ok(options.existingPlugin,
    'Canonical publication requires the existing registered plugin; host creation is not permitted');
  const exportEvidence = canonicalPublicationExportEvidence(options.exportEvidence);
  assert.ok(typeof options.workerBuildReceipt === 'string' && options.workerBuildReceipt.length > 0,
    'Canonical publication requires a pinned strict worker build receipt');
  assert.match(options.workerBuildReceiptSha256 ?? '', /^[0-9a-f]{64}$/, 'Pin the strict worker receipt SHA-256');
  const bundleBytes = await readFile(options.bundle);
  // Markers are only a negative gate; the full strict input closure must also match.
  for (const marker of ['canonical-source', 'canonical-record'])
    assert.ok(bundleBytes.includes(Buffer.from(marker)), `Selected worker lacks ${marker}; do not import into the shared library`);
  const bundleSha256 = sha256(bundleBytes);
  const workerBuildEvidence = await verifyCurrentPluginWorkerBuild({
    buildReceiptPath: options.workerBuildReceipt, buildReceiptSha256: options.workerBuildReceiptSha256,
    repositoryRoot: options.repoRoot, bundleBytes,
  });
  const output = requireRealSessionDirectory(options.output);
  const identity = workspaceIdentity(options.workspaceUrl);
  assert.ok(['127.0.0.1', 'localhost'].includes(new URL(identity.origin).hostname));
  assert.ok(options.sourcePaths.length > 0, 'Canonical publication needs explicit source owners');
  assert.equal(new Set(options.sourcePaths).size, options.sourcePaths.length, 'Duplicate source publication');
  if (options.append) {
    assert.equal(options.append.fileId, identity.fileId, 'Append target differs from the assigned library');
    assert.ok(options.existingPlugin, 'Append requires the existing managed plugin host and registration');
  }
  const decode = transitDecoder(options.transitPackage);
  const receipt: Record<string, unknown> = {status: 'running', backend: 'real', identity,
    workspaceUrl: options.workspaceUrl, globalStylePropagationVerified: false,
    crossFileReuseVerified: false, canonicalSourceWritebackApplied: false,
    nativeEditorAcceptance: false, formalAcceptance: false,
    exportEvidence, officialPngRequested: exportEvidence === 'source-owner-and-png', officialPngDownloaded: false};
  receipt['workerBuildEvidence'] = workerBuildEvidence;
  const checkpoint = async () => writeFile(resolve(output, 'publication-receipt.json'), JSON.stringify(receipt, null, 2));
  const dependencies = new LayoutDependencies();
  await dependencies.load(options.repoRoot, await trackedZuiPaths());
  const prepared = await Promise.all(options.sourcePaths.map(async sourcePath => {
    assert.ok(!sourcePath.includes('..') && sourcePath.startsWith('zircon_editor/assets/ui/'));
    const source = await readFile(resolve(options.repoRoot, sourcePath), 'utf8');
    const original = parseZuiDocument(source).document;
    assert.equal(original.asset.kind, 'component');
    const projection = cloneZuiDocument(original);
    dependencies.embed(projection, sourcePath, 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui');
    // embed returns change descriptions; the actual source closure is metadata.
    const closure = projection['penpot_dependency_sources'];
    assert.ok(Array.isArray(closure) && closure.every(path => typeof path === 'string'));
    projection['penpot_source_fingerprints'] = [...new Set([sourcePath,
      ...closure.filter((path): path is string => typeof path === 'string' && path.endsWith('.zui'))])].map(path => {
      const fingerprint = dependencies.sourceFingerprint(path);
      assert.ok(fingerprint, `Missing canonical source fingerprint: ${path}`);
      return fingerprint;
    });
    projection['penpot_original_source'] = source;
    projection['penpot_original_source_path'] = sourcePath;
    return {sourcePath, source, original, projection: serializeZuiDocument(projection), sha256: sha256(source)};
  }));
  receipt['sources'] = prepared.map(({sourcePath, sha256}) => ({sourcePath, sha256}));
  const temp = resolve(output, 'browser-temp');
  const downloads = resolve(output, 'downloads');
  await mkdir(temp, {recursive: true}); await mkdir(downloads, {recursive: true});
  let browser: Awaited<ReturnType<typeof chromium.launch>> | undefined;
  let context: Awaited<ReturnType<NonNullable<typeof browser>['newContext']>> | undefined;
  const errors: string[] = []; const saveErrors: string[] = []; const pending = new Set<Promise<void>>();
  const saves: {revn: number; requestSha256: string; responseSha256: string}[] = [];
  receipt['browserEvents'] = errors; receipt['saveErrors'] = saveErrors; receipt['saves'] = saves;
  try {
    browser = await chromium.launch({executablePath: options.executablePath, headless: true, downloadsPath: downloads,
      env: {...process.env, TEMP: temp, TMP: temp, TMPDIR: temp, USERPROFILE: temp,
        APPDATA: resolve(temp, 'roaming'), LOCALAPPDATA: resolve(temp, 'local')}});
    context = await browser.newContext({storageState: options.storageState, acceptDownloads: true,
      viewport: {width: 1920, height: 1200}, locale: 'en-US', serviceWorkers: 'block'});
    const initial = await rpc<any>(context, identity.origin, 'get-file', {id: identity.fileId}, decode);
    assert.ok(initial.data['pages-index'][identity.pageId], 'Assigned library page is missing');
    const initialMetadata = persistedZuiMetadata(initial, identity.pageId);
    const existingRecords = initialMetadata.map(asset => {
      assert.equal(typeof asset.metadata['canonical-component'], 'string', 'Existing asset is not a canonical master');
      const record = JSON.parse(asset.metadata['canonical-component']) as CanonicalComponentRecord;
      assert.equal(record.version, 1);
      assert.equal(record.libraryId, identity.fileId);
      assert.equal(record.assetBoardId, asset.boardId);
      assert.equal(initial.data.components?.[record.componentId]?.['main-instance-id'], record.mainInstanceId);
      assert.equal(initial.data['pages-index'][identity.pageId].objects[record.mainInstanceId]?.['component-id'], record.componentId);
      return record;
    });
    assert.equal(new Set(existingRecords.map(record => record.componentId)).size, existingRecords.length);
    assert.equal(new Set(existingRecords.map(record => record.source)).size, existingRecords.length,
      'Ambiguous existing canonical source');
    if (options.append) {
      assert.equal(initial.revn, options.append.revn, 'Library revision changed since append admission');
      assert.equal(initial.vern, options.append.vern);
      assert.equal(existingRecords.length, Object.keys(initial.data.components ?? {}).length,
        'Unmapped native mains require an explicit migration review');
    } else {
      assert.ok(Object.keys(initial.data['pages-index'][identity.pageId].objects).length <= 1,
        'First publication requires the explicitly assigned empty shared-library page');
      assert.equal(Object.keys(initial.data.components ?? {}).length, 0);
    }
    const imports = prepared.filter(item => {
      const names = Object.keys(item.original.components ?? {});
      assert.equal(names.length, 1);
      const source = `${item.original.asset.id}#${names[0]}`;
      const existing = existingRecords.find(record => record.source === source);
      if (!existing) return true;
      assert.equal(existing.sourcePath, item.sourcePath, 'Existing canonical source owner differs');
      const asset = initialMetadata.find(asset => asset.boardId === existing.assetBoardId)!;
      const persisted = decodeZuiMetadata(asset.metadata['document']);
      assert.equal(typeof persisted['penpot_original_source'], 'string', 'Canonical source provenance is missing');
      assert.deepEqual(normalizeZuiDocument(parseZuiDocument(persisted['penpot_original_source'] as string).document),
        normalizeZuiDocument(item.original), 'Existing canonical source changed; reconcile its original main explicitly');
      return false;
    });
    receipt['reusedCanonicalComponents'] = existingRecords;
    receipt['newSourcePaths'] = imports.map(item => item.sourcePath);
    receipt['initialRevision'] = {revn: initial.revn, vern: initial.vern};
    await checkpoint();
    const host = options.existingPlugin.host;
    assert.ok(['127.0.0.1', 'localhost'].includes(new URL(host).hostname), 'Plugin host must be local');
    const manifest = JSON.parse(await readFile(resolve(options.dist, 'manifest.json'), 'utf8'));
    if (options.existingPlugin) {
      const servedManifest = await context.request.get(new URL('manifest.json', host).href);
      assert.equal(servedManifest.status(), 200);
      assert.equal(sha256(await servedManifest.body()), options.existingPlugin.manifestSha256,
        'Managed plugin manifest changed');
      assert.deepEqual(JSON.parse(await servedManifest.text()), manifest, 'Served manifest differs from the selected distribution');
    }
    const servedWorker = await context.request.get(new URL(manifest['code'], host).href);
    assert.equal(servedWorker.status(), 200);
    assert.equal(sha256(await servedWorker.body()), bundleSha256, 'Managed worker differs from the selected canonical build');
    receipt['pluginBundleSha256'] = bundleSha256;
    const index = await context.request.get(host);
    receipt['uiBuildEvidence'] = await verifyCurrentPluginUi({buildReceiptPath: options.uiBuildReceipt,
      repositoryRoot: options.repoRoot, dist: options.dist, host, indexBytes: await index.body(),
      get: (url: string) => context!.request.get(url)});
    const session = await openWorkspace(context, options.workspaceUrl, host, manifest, bundleSha256, errors, decode,
      async page => {
        receipt['saveObserver'] = await observeRealSaveHttp(page, {fileId: identity.fileId, output, pending,
          diagnostics: [], onError: error => saveErrors.push(String(error)), onResponse: async actual => {
            const request = decode(actual.requestBody.toString('utf8')) as Parameters<typeof verifySaveResponse>[3];
            const revn = verifySaveResponse(decode(actual.responseBody.toString('utf8')), identity,
              initial.revn, request);
            saves.push({revn, requestSha256: sha256(actual.requestBody), responseSha256: sha256(actual.responseBody)});
          }});
      }, options.existingPlugin);
    receipt['installation'] = session.installation;
    if (options.fontResources) {
      await session.frame.locator('[data-font-resource-manifest]').setInputFiles(options.fontResources);
      await session.frame.locator('.status-panel').filter({hasText: '字体资源清单'}).waitFor();
    }
    for (const item of imports) {
      await session.frame.locator('[data-component-master-input]').setInputFiles({name: item.sourcePath.split('/').pop()!,
        mimeType: 'text/plain', buffer: Buffer.from(item.projection)});
      await session.frame.locator('.status-panel:not([data-level="working"])').waitFor({timeout: 120_000});
      assert.notEqual(await session.frame.locator('.status-panel').getAttribute('data-level'), 'error',
        await session.frame.locator('.status-panel').innerText());
      await checkpoint();
    }
    let saved: any; let metadata: ReturnType<typeof persistedZuiMetadata> = [];
    const deadline = Date.now() + 60_000;
    while (Date.now() < deadline) {
      if (saveErrors.length) throw new Error(saveErrors.join('\n'));
      saved = await rpc<any>(context, identity.origin, 'get-file', {id: identity.fileId}, decode);
      metadata = persistedZuiMetadata(saved, identity.pageId);
      if (metadata.length === initialMetadata.length + imports.length &&
          (!imports.length || saves.some(save => save.revn >= saved.revn))) break;
      await session.page.waitForTimeout(250);
    }
    assert.equal(metadata.length, initialMetadata.length + imports.length);
    if (imports.length) assert.ok(saves.some(save => save.revn >= saved.revn));
    assertCanonicalLibraryPreserved(initial, saved);
    const records = metadata.map(asset => {
      const value = asset.metadata['canonical-component'];
      assert.equal(typeof value, 'string');
      return JSON.parse(value as string) as CanonicalComponentRecord;
    });
    assert.equal(new Set(records.map(record => record.componentId)).size, metadata.length);
    for (const record of records) {
      assert.equal(record.libraryId, identity.fileId);
      assert.equal(saved.data.components[record.componentId]['main-instance-id'], record.mainInstanceId);
      assert.equal(saved.data['pages-index'][identity.pageId].objects[record.mainInstanceId]['component-id'], record.componentId);
    }
    receipt['saveAcknowledged'] = imports.length > 0;
    receipt['noSaveRequired'] = imports.length === 0;
    receipt['existingLibraryPreserved'] = true; receipt['canonicalComponents'] = records;
    await writeFile(resolve(output, 'saved-metadata.json'), JSON.stringify(metadata, null, 2));
    await Promise.all([...pending]);
    const authentication = await context.storageState();
    await context.close(); context = undefined; receipt['closedOriginalContext'] = true;
    context = await browser.newContext({storageState: authentication, acceptDownloads: true,
      viewport: {width: 1920, height: 1200}, locale: 'en-US', serviceWorkers: 'block'});
    receipt['newAuthenticatedContext'] = true;
    const reopenedFile = await rpc<any>(context, identity.origin, 'get-file', {id: identity.fileId}, decode);
    assert.equal(reopenedFile.id, saved.id); assert.ok(reopenedFile.revn >= saved.revn);
    assert.deepEqual(persistedZuiMetadata(reopenedFile, identity.pageId), metadata);
    assert.deepEqual(reopenedFile.data.components, saved.data.components);
    receipt['reopenedSameFile'] = true; receipt['metadataMatched'] = true;
    receipt['nativeCopyFreshReopen'] = assertNativeInstanceGeometryPreserved(saved, reopenedFile);
    assertCanonicalLibraryPreserved(initial, reopenedFile);
    const reopened = await openWorkspace(context, options.workspaceUrl, host, manifest, bundleSha256, errors, decode,
      undefined, options.existingPlugin);
    const exports: unknown[] = []; receipt['exports'] = exports;
    for (const [index, item] of prepared.entries()) {
      const record = records.find(record => record.sourcePath === item.sourcePath)!;
      assert.ok(record);
      await reopened.page.locator(`[data-testid="layer-row"][id="${record.assetBoardId}"]`).click();
      await reopened.frame.locator(`.selection-band[data-selected-board-id="${record.assetBoardId}"]`).waitFor();
      const download = await withSourceColorRevisionFence(context, identity, record.assetBoardId,
        parseZuiDocument(item.projection).document, decode, receipt, async () => {
          const downloadPending = reopened.page.waitForEvent('download', {timeout: 30_000});
          await reopened.frame.getByRole('button', {name: 'Export selected', exact: true}).click();
          return downloadPending;
        });
      const target = resolve(output, `component-${index + 1}.zui`);
      await download.saveAs(target); const exported = await readFile(target, 'utf8');
      const evidence = await collectCanonicalOwnerExportEvidence(item.original, exported, exportEvidence, async () => {
        const canvas: Record<string, unknown> = {};
        await downloadActualCanvasPng(reopened.page, reopened.frame, record.assetBoardId, output, canvas);
        const pngPath = resolve(output, `component-${index + 1}.png`);
        await writeFile(pngPath, await readFile(resolve(output, 'canvas-export.png')));
        return {pngPath, canvas};
      });
      exports.push({sourcePath: item.sourcePath, sourceSha256: item.sha256, exportPath: target,
        exportSha256: sha256(exported), ...evidence});
      assert.equal(sha256(await readFile(resolve(options.repoRoot, item.sourcePath))), item.sha256);
      await checkpoint();
    }
    receipt['exportSemanticMatch'] = true; receipt['sourceFingerprintMatched'] = true;
    receipt['officialPngDownloaded'] = exportEvidence === 'source-owner-and-png';
    receipt['status'] = exportEvidence === 'source-owner'
      ? 'passed-component-publication-and-owner-source-export'
      : 'passed-component-publication-and-persistence';
  } catch (error) {
    if (error && typeof error === 'object' && 'nativeCopyDelta' in error)
      receipt['nativeCopyFailureDelta'] = error.nativeCopyDelta;
    const page = context?.pages().at(-1);
    if (page) {
      await page.screenshot({path: resolve(output, 'failure.png'), fullPage: true, timeout: 10_000}).catch(() => undefined);
      receipt['failureControls'] = await page.locator('input, button').evaluateAll(elements =>
        elements.slice(0, 80).map(element => ({tag: element.tagName,
          placeholder: element.getAttribute('placeholder'), text: element.textContent?.slice(0, 120)}))).catch(() => []);
    }
    receipt['status'] = 'failed'; receipt['error'] = String(error instanceof Error ? error.stack : error); throw error;
  } finally {
    await context?.close(); await browser?.close(); await Promise.allSettled([...pending]);
    receipt['finishedAtUtc'] = new Date().toISOString(); await checkpoint();
  }
}
