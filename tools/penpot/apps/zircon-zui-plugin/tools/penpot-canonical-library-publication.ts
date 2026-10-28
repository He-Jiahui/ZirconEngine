import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { chromium } from 'playwright';
import { cloneZuiDocument, decodeZuiMetadata, normalizeZuiDocument, parseZuiDocument, serializeZuiDocument, type ZuiDocument } from '../src/bridge/zui-document';
import { FIELD_ICON_PATHS } from '../src/bridge/zui-field-projection';
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

interface CanonicalPluginResourceResponse {
  status(): number; url(): string; headers(): Record<string, string>;
}

interface CanonicalPluginResponseEvidence {
  resourceUrl: string; responseUrl: string; httpStatus: number;
  requestOrigin: string; crossOrigin: boolean; accessControlAllowOrigin: string | null;
}

/** API requests bypass browser CORS. Check the response needed by Penpot's
 * cors/same-origin manifest and code fetches before native Open or SDK writes.
 * Header admission alone does not establish browser, plugin or SDK readiness. */
export async function requestCanonicalPluginResource<T extends CanonicalPluginResourceResponse>(
  workspaceOrigin: string, url: string,
  get: (url: string, options: {headers: {Origin: string}; maxRedirects: 0}) => Promise<T>,
  record: (evidence: CanonicalPluginResponseEvidence) => void,
): Promise<T> {
  const requestOrigin = new URL(workspaceOrigin).origin;
  const resourceUrl = new URL(url).href;
  const response = await get(resourceUrl, {headers: {Origin: requestOrigin}, maxRedirects: 0});
  const accessControlAllowOrigin = response.headers()['access-control-allow-origin'] ?? null;
  const crossOrigin = new URL(resourceUrl).origin !== requestOrigin;
  record({resourceUrl, responseUrl: response.url(), httpStatus: response.status(),
    requestOrigin, crossOrigin, accessControlAllowOrigin});
  assert.equal(response.status(), 200, `Plugin resource HTTP status: ${resourceUrl}`);
  assert.equal(response.url(), resourceUrl, `Plugin resource redirected: ${resourceUrl}`);
  const allowedOrigin = accessControlAllowOrigin?.trim();
  assert.ok(!crossOrigin || allowedOrigin === '*' || allowedOrigin === requestOrigin,
    `Plugin resource CORS rejected ${requestOrigin}: ${resourceUrl}; Access-Control-Allow-Origin=${accessControlAllowOrigin}`);
  return response;
}

interface CanonicalPublicationSourcePin { sourcePath: string; sha256: string }

/** Pin the bytes already embedded by the source loader, never a later replacement
 * read. The ordinary source fence compares these pins to the actual files before
 * SDK writes and owner exports. Font/resource admission remains independent. */
export function canonicalPublicationEmbeddedSvgPins(
  projection: ZuiDocument, dependencyPaths: readonly string[],
): CanonicalPublicationSourcePin[] {
  const pins = new Map<string, string>();
  const add = (sourcePath: string, svg: unknown) => {
    assert.ok((sourcePath.startsWith('zircon_editor/assets/icons/') ||
      /^zircon_plugins\/[^/]+\/assets\/icons\//.test(sourcePath)) &&
      sourcePath.endsWith('.svg') && !/[:\\?#\u0000]/.test(sourcePath) &&
      sourcePath.split('/').every(part => part !== '' && part !== '.' && part !== '..'),
    'Invalid embedded SVG source path: ' + sourcePath);
    assert.ok(typeof svg === 'string' && svg.length > 0,
      'Missing embedded SVG bytes: ' + sourcePath);
    const digest = sha256(svg);
    const previous = pins.get(sourcePath);
    assert.ok(previous === undefined || previous === digest,
      'Conflicting embedded SVG bytes: ' + sourcePath);
    pins.set(sourcePath, digest);
  };
  const entries = (value: unknown, name: string): [string, unknown][] => {
    if (value === undefined) return [];
    assert.ok(value !== null && typeof value === 'object' && !Array.isArray(value),
      'Invalid embedded SVG table: ' + name);
    return Object.entries(value);
  };
  for (const [key, svg] of entries(projection['penpot_field_icons'], 'field icons')) {
    const sourcePath = Object.entries(FIELD_ICON_PATHS).find(([name]) => name === key)?.[1];
    assert.ok(sourcePath, 'Unknown embedded field SVG: ' + key);
    add(sourcePath, svg);
  }
  for (const [asset, svg] of entries(projection['penpot_native_icon_svgs'], 'native icons'))
    add('zircon_editor/assets/icons/' + asset, svg);
  for (const node of Object.values(projection.nodes ?? {})) {
    const svg = node['penpot_icon_svg'];
    if (svg === undefined) continue;
    const tab = node['penpot_review_native_tab_icon'];
    const tabAsset = tab !== null && typeof tab === 'object' && !Array.isArray(tab)
      ? (tab as Record<string, unknown>)['assetPath'] : undefined;
    const icon = node.props?.['icon'] ??
      (node.component.toLowerCase().includes('icon') ? node.props?.['source'] : undefined);
    // The loader applies the native tab SVG after any authored icon.
    if (typeof tabAsset === 'string') {
      add('zircon_editor/assets/icons/' + tabAsset, svg);
    } else if (typeof icon === 'string' && icon.endsWith('.svg')) {
      const owner = node['penpot_review_source_path'];
      assert.ok(typeof owner === 'string' &&
        /^(zircon_editor|zircon_plugins\/[^/]+)\/assets\/ui\/.+\.zui$/.test(owner),
      'Embedded authored SVG has no source owner');
      const assetRoot = owner.slice(0, owner.indexOf('/assets/')) + '/assets';
      add(assetRoot + '/icons/' + icon.replace(/^res:\/\/(icons\/)?/, ''), svg);
    } else {
      assert.fail('Embedded SVG has no authored or native tab source');
    }
  }
  for (const sourcePath of dependencyPaths) {
    if (sourcePath.endsWith('.zui')) continue;
    assert.ok(pins.has(sourcePath),
      'Canonical dependency has no embedded SVG provenance: ' + sourcePath);
  }
  return [...pins].map(([sourcePath, sha256]) => ({sourcePath, sha256}));
}

/** Bind parsed owners to the exact dependency bytes used by their projection. */
export function canonicalPublicationSourcePins(sources: readonly {
  sourcePath: string; sha256: string; sourceFingerprints: readonly CanonicalPublicationSourcePin[];
}[]): CanonicalPublicationSourcePin[] {
  const pins = new Map<string, string>();
  for (const source of sources) {
    const owner = source.sourceFingerprints.filter(pin => pin.sourcePath === source.sourcePath);
    const [ownerPin] = owner;
    assert.ok(ownerPin && owner.length === 1, `Missing or ambiguous canonical source fingerprint: ${source.sourcePath}`);
    assert.equal(ownerPin.sha256, source.sha256,
      `Canonical source changed during dependency preparation: ${source.sourcePath}`);
    for (const pin of source.sourceFingerprints) {
      assert.match(pin.sha256, /^[0-9a-f]{64}$/, `Invalid canonical source fingerprint: ${pin.sourcePath}`);
      const previous = pins.get(pin.sourcePath);
      assert.ok(previous === undefined || previous === pin.sha256,
        `Conflicting canonical dependency fingerprints: ${pin.sourcePath}`);
      pins.set(pin.sourcePath, pin.sha256);
    }
  }
  assert.ok(pins.size > 0, 'Canonical publication needs pinned source bytes');
  return [...pins].map(([sourcePath, sha256]) => ({sourcePath, sha256}));
}

async function assertCanonicalPublicationSourcePins(
  pins: readonly CanonicalPublicationSourcePin[], readSha256: (sourcePath: string) => Promise<string>,
): Promise<void> {
  assert.ok(pins.length > 0, 'Canonical publication needs pinned source bytes');
  assert.equal(new Set(pins.map(pin => pin.sourcePath)).size, pins.length, 'Duplicate canonical source pins');
  const current = await Promise.all(pins.map(pin => readSha256(pin.sourcePath)));
  for (const [index, pin] of pins.entries())
    assert.equal(current[index], pin.sha256, `Canonical source bytes changed: ${pin.sourcePath}`);
}

/** Detect drift around awaited SDK work; the managed writer still owns exclusion.
 * A post-operation failure retains any actual partial write for reconciliation. */
export async function withCanonicalPublicationSourcePins<T>(
  pins: readonly CanonicalPublicationSourcePin[], readSha256: (sourcePath: string) => Promise<string>,
  operation: () => Promise<T>,
): Promise<T> {
  await assertCanonicalPublicationSourcePins(pins, readSha256);
  const result = await operation();
  await assertCanonicalPublicationSourcePins(pins, readSha256);
  return result;
}

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

type CanonicalPublicationExportResult = Awaited<ReturnType<typeof collectCanonicalOwnerExportEvidence>> & {
  exportPath: string; exportSha256: string;
};

/** Keep both source fences open through download persistence, semantic comparison
 * and any requested PNG. A download event alone does not complete the evidence. */
export async function collectCanonicalPublicationExportEvidence(
  pins: readonly CanonicalPublicationSourcePin[], readSha256: (sourcePath: string) => Promise<string>,
  sourceColorFence: (operation: () => Promise<CanonicalPublicationExportResult>) => Promise<CanonicalPublicationExportResult>,
  original: Parameters<typeof normalizeZuiDocument>[0], requested: CanonicalPublicationExportEvidence,
  exportSource: () => Promise<{exportPath: string; exported: string}>,
  capturePng: () => Promise<{pngPath: string; canvas: Record<string, unknown>}>,
): Promise<CanonicalPublicationExportResult> {
  return withCanonicalPublicationSourcePins(pins, readSha256, () => sourceColorFence(async () => {
    const source = await exportSource();
    const evidence = await collectCanonicalOwnerExportEvidence(original, source.exported, requested, capturePng);
    return {exportPath: source.exportPath, exportSha256: sha256(source.exported), ...evidence};
  }));
}

/** An append may add mains; the previous source owners and styles stay intact. */
export function assertCanonicalLibraryPreserved(before: any, after: any): void {
  assertExistingPenpotContentPreserved(before, after);
}

/** The observer records failures through onError, so fulfilled pending tasks
 * alone do not prove a save succeeded. Include errors raised during close. */
export async function closeCanonicalPublicationSaveContext<T>(
  pending: ReadonlySet<Promise<void>>, saveErrors: readonly string[], close: () => Promise<T>,
): Promise<T> {
  const settle = async () => {
    do { await Promise.all([...pending]); } while (pending.size);
    if (saveErrors.length) throw new Error(saveErrors.join('\n'));
  };
  await settle();
  const result = await close();
  await settle();
  return result;
}

/** Exports are read-only. Verify the complete saved library after the export
 * workspace closes, including changes that materialize during close. */
export async function verifyCanonicalPublicationExportReadback<T extends {
  id: string; revn: number; vern: number; data: unknown;
}>(before: T, close: () => Promise<void>, readFresh: () => Promise<T>): Promise<T> {
  await close();
  const after = await readFresh();
  assert.equal(after.id, before.id, 'Canonical export readback returned a different file');
  assert.equal(after.vern, before.vern, 'Read-only canonical export changed file version');
  assert.equal(after.revn, before.revn, 'Read-only canonical export changed file revision');
  assert.deepEqual(after.data, before.data, 'Read-only canonical export changed file data');
  return after;
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
    const sourceFingerprints = [...new Set([sourcePath,
      ...closure.filter((path): path is string => typeof path === 'string' && path.endsWith('.zui'))])].map(path => {
      const fingerprint = dependencies.sourceFingerprint(path);
      assert.ok(fingerprint, `Missing canonical source fingerprint: ${path}`);
      return fingerprint;
    });
    projection['penpot_source_fingerprints'] = sourceFingerprints;
    projection['penpot_original_source'] = source;
    projection['penpot_original_source_path'] = sourcePath;
    return {sourcePath, source, original, projection: serializeZuiDocument(projection), sha256: sha256(source),
      sourceFingerprints: [...sourceFingerprints, ...canonicalPublicationEmbeddedSvgPins(projection, closure)]};
  }));
  receipt['sources'] = prepared.map(({sourcePath, sha256}) => ({sourcePath, sha256}));
  const sourcePins = canonicalPublicationSourcePins(prepared);
  const readSourceSha256 = async (sourcePath: string) => sha256(await readFile(resolve(options.repoRoot, sourcePath)));
  await assertCanonicalPublicationSourcePins(sourcePins, readSourceSha256);
  receipt['zuiSourcePins'] = sourcePins.filter(pin => pin.sourcePath.endsWith('.zui'));
  receipt['embeddedSvgSourcePins'] = sourcePins.filter(pin => !pin.sourcePath.endsWith('.zui'));
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
    const pluginResponses: CanonicalPluginResponseEvidence[] = [];
    receipt['pluginResponsePreflight'] = pluginResponses;
    const getPluginResource = (url: string) => requestCanonicalPluginResource(identity.origin, url,
      (url, options) => context!.request.get(url, options), evidence => { pluginResponses.push(evidence); });
    const manifest = JSON.parse(await readFile(resolve(options.dist, 'manifest.json'), 'utf8'));
    if (options.existingPlugin) {
      const servedManifest = await getPluginResource(new URL('manifest.json', host).href);
      assert.equal(servedManifest.status(), 200);
      assert.equal(sha256(await servedManifest.body()), options.existingPlugin.manifestSha256,
        'Managed plugin manifest changed');
      assert.deepEqual(JSON.parse(await servedManifest.text()), manifest, 'Served manifest differs from the selected distribution');
    }
    const servedWorker = await getPluginResource(new URL(manifest['code'], host).href);
    assert.equal(servedWorker.status(), 200);
    assert.equal(sha256(await servedWorker.body()), bundleSha256, 'Managed worker differs from the selected canonical build');
    receipt['pluginBundleSha256'] = bundleSha256;
    const index = await context.request.get(host);
    receipt['uiBuildEvidence'] = await verifyCurrentPluginUi({buildReceiptPath: options.uiBuildReceipt,
      repositoryRoot: options.repoRoot, dist: options.dist, host, indexBytes: await index.body(),
      get: (url: string) => context!.request.get(url)});
    const observeSaves = async (page: Parameters<typeof observeRealSaveHttp>[0],
      key: 'saveObserver' | 'exportSaveObserver') => {
      // Each observer numbers its captures from one; retain the original save files.
      const observerOutput = key === 'saveObserver' ? output : resolve(output, 'export-save-observer');
      await mkdir(observerOutput, {recursive: true});
      receipt[`${key}Output`] = observerOutput;
      receipt[key] = await observeRealSaveHttp(page, {fileId: identity.fileId, output: observerOutput, pending,
        diagnostics: [], onError: error => saveErrors.push(String(error)), onResponse: async actual => {
          const request = decode(actual.requestBody.toString('utf8')) as Parameters<typeof verifySaveResponse>[3];
          const revn = verifySaveResponse(decode(actual.responseBody.toString('utf8')), identity,
            initial.revn, request);
          saves.push({revn, requestSha256: sha256(actual.requestBody), responseSha256: sha256(actual.responseBody)});
        }});
    };
    const session = await openWorkspace(context, options.workspaceUrl, host, manifest, bundleSha256, errors, decode,
      page => observeSaves(page, 'saveObserver'), options.existingPlugin);
    receipt['installation'] = session.installation;
    if (options.fontResources) {
      await session.frame.locator('[data-font-resource-manifest]').setInputFiles(options.fontResources);
      await session.frame.locator('.status-panel').filter({hasText: '字体资源清单'}).waitFor();
    }
    for (const item of imports) {
      await withCanonicalPublicationSourcePins(sourcePins, readSourceSha256, async () => {
        await session.frame.locator('[data-component-master-input]').setInputFiles({name: item.sourcePath.split('/').pop()!,
          mimeType: 'text/plain', buffer: Buffer.from(item.projection)});
        await session.frame.locator('.status-panel:not([data-level="working"])').waitFor({timeout: 120_000});
        assert.notEqual(await session.frame.locator('.status-panel').getAttribute('data-level'), 'error',
          await session.frame.locator('.status-panel').innerText());
      });
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
    receipt['existingLibraryPreserved'] = true; receipt['canonicalComponents'] = records;
    await writeFile(resolve(output, 'saved-metadata.json'), JSON.stringify(metadata, null, 2));
    const authentication = await closeCanonicalPublicationSaveContext(pending, saveErrors, async () => {
      const authentication = await context!.storageState();
      await context!.close(); context = undefined; receipt['closedOriginalContext'] = true;
      return authentication;
    });
    receipt['saveAcknowledged'] = imports.length > 0;
    receipt['noSaveRequired'] = imports.length === 0;
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
      page => observeSaves(page, 'exportSaveObserver'), options.existingPlugin);
    const exports: unknown[] = []; receipt['exports'] = exports;
    for (const [index, item] of prepared.entries()) {
      const record = records.find(record => record.sourcePath === item.sourcePath)!;
      assert.ok(record);
      await reopened.page.locator(`[data-testid="layer-row"][id="${record.assetBoardId}"]`).click();
      await reopened.frame.locator(`.selection-band[data-selected-board-id="${record.assetBoardId}"]`).waitFor();
      const evidence = await collectCanonicalPublicationExportEvidence(sourcePins, readSourceSha256,
        operation => withSourceColorRevisionFence(context!, identity, record.assetBoardId,
          parseZuiDocument(item.projection).document, decode, receipt, operation),
        item.original, exportEvidence, async () => {
          const downloadPending = reopened.page.waitForEvent('download', {timeout: 30_000});
          await reopened.frame.getByRole('button', {name: 'Export selected', exact: true}).click();
          const download = await downloadPending;
          const exportPath = resolve(output, `component-${index + 1}.zui`);
          await download.saveAs(exportPath);
          return {exportPath, exported: await readFile(exportPath, 'utf8')};
        }, async () => {
          const canvas: Record<string, unknown> = {};
          await downloadActualCanvasPng(reopened.page, reopened.frame, record.assetBoardId, output, canvas);
          const pngPath = resolve(output, `component-${index + 1}.png`);
          await writeFile(pngPath, await readFile(resolve(output, 'canvas-export.png')));
          return {pngPath, canvas};
        });
      exports.push({sourcePath: item.sourcePath, sourceSha256: item.sha256, ...evidence});
      assert.equal(sha256(await readFile(resolve(options.repoRoot, item.sourcePath))), item.sha256);
      await checkpoint();
    }
    const exportReadback = await verifyCanonicalPublicationExportReadback(reopenedFile,
      () => closeCanonicalPublicationSaveContext(pending, saveErrors, async () => {
        await context!.close(); context = undefined; receipt['closedExportContext'] = true;
      }), async () => {
        context = await browser!.newContext({storageState: authentication, acceptDownloads: true,
          viewport: {width: 1920, height: 1200}, locale: 'en-US', serviceWorkers: 'block'});
        receipt['newExportReadbackContext'] = true;
        return rpc<any>(context, identity.origin, 'get-file', {id: identity.fileId}, decode);
      });
    receipt['exportReadbackVerified'] = true;
    receipt['exportReadbackRevision'] = {revn: exportReadback.revn, vern: exportReadback.vern};
    await assertCanonicalPublicationSourcePins(sourcePins, readSourceSha256);
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
