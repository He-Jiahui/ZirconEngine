import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { createServer, type Server } from 'node:http';
import { createRequire } from 'node:module';
import { mkdir, readFile, rename, writeFile } from 'node:fs/promises';
import { realpathSync, statSync } from 'node:fs';
import { basename, dirname, extname, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium, request, type APIRequestContext, type BrowserContext, type Frame, type FrameLocator, type Page } from 'playwright';
import { cloneZuiDocument, decodeZuiMetadata, normalizeZuiDocument, parseZuiDocument, type ZuiDocument } from '../src/bridge/zui-document.js';
import { applyZuiExportFileToCanonicalSourceOwners } from '../src/bridge/zui-export-apply-files';
import { applyZuiExportToSourceFiles } from '../src/bridge/zui-export-apply-sources';
import { parseFingerprintList, parseSourceExportManifest, validateSourcePath } from '../src/bridge/zui-export-apply-contract';
import { clickAndWaitForDownload } from './zui-layout-download';
import { reviewHost } from '../src/bridge/zui-review-host';
import { importedActualNormalReviewCase, reviewCaseFromSavedMetadata, runtimeSceneMediaForCase,
  runtimeSceneMediaReviewTarget } from '../src/bridge/zui-review-case';
import { viewportToolbarContract, viewportToolbarSourceMapping } from '../src/bridge/zui-viewport-toolbar';
import { assertExistingPenpotContentPreserved, assertNativeInstanceGeometryPreserved, verifySourceOwnerAppendAdmission,
  sourceColorRevisionSnapshot, assertSourceColorRevisionPreserved,
  persistenceClaim, persistedZuiMetadata, persistedViewportToolbarGroupMetadata, requireRealSessionDirectory,
  sha256, verifyReopenedFile, verifyReviewScrollCleanupAdmission, verifySaveResponse, workspaceIdentity, realSessionExecutionPlan } from './zui-layout-real-session-contract.mjs';
import { createRealPenpotProfile } from './zui-layout-real-profile.mjs';
import {
  prepareRealOwnerInput, prepareRealWorkbenchInput, prepareRealOwnerGapEdit,
  validateRealOwnerGapExport, validateRealWorkbenchExportSources,
} from './zui-layout-real-workbench-input';
import { validateLayoutReviewCase } from './zui-layout-review-contract';
import { editWorkbenchSpacingInPenpot, editSourceOwnerSpacingInPenpot, editOutlinerMinimumHeightInPenpot } from './penpot-workbench-canvas-edit';
import { observeRealSaveHttp, type ActualSaveHttp } from './zui-layout-real-save-observer';
import { compareStoredZuiMetadata } from './zui-layout-real-metadata';
import { observeActualExportClick } from './zui-layout-real-export-observer';
import { verifyCurrentPluginUi, verifyCurrentPluginWorker, verifyCurrentPluginWorkerBuild } from './zui-layout-real-ui.mjs';
import { pluginRegistrationFingerprint, pluginRegistrationFingerprintProtocol, pluginRegistrationPermissions,
  verifyExistingPluginRegistration, verifyPluginRegistryUnchanged } from './zui-layout-real-plugin-registration';
import { requestPreviewLayoutAudit, previewLayoutAuditTimeoutForNodes } from './penpot-preview-layout-audit';
import { projectZuiDocument } from '../src/bridge/penpot-projection';

type RecordValue = Record<string, unknown>;
interface IdentifiedRecord { id: string }
interface BackendTeam extends IdentifiedRecord { 'is-default'?: boolean }
interface BackendFile extends IdentifiedRecord {
  revn: number;
  vern: number;
  data: { pages: string[]; 'pages-index': Record<string, { objects: RecordValue }> };
}
interface SaveEvidence {
  httpStatus: number; url: string; requestId: string; responseSha256: string; requestSha256: string;
  responseFile: string; requestFile: string; revn: number; observedAt: number;
}
interface SaveRequest { id: string; revn: number; 'session-id': string; changes: unknown[] }
interface TransitMap { forEach(visitor: (value: unknown, key: unknown) => void): void }
interface TransitLibrary {
  reader(format: string, options: RecordValue): { read(body: string): unknown };
  isSet(value: unknown): boolean;
}
function record(value: unknown): RecordValue {
  assert.ok(value && typeof value === 'object' && !Array.isArray(value), 'Expected an actual transport object');
  return value as RecordValue;
}
async function captureDomFonts(page: Page): Promise<unknown> {
  return page.evaluate(() => {
    const faces: { family: string; weight: string; style: string; status: string }[] = [];
    document.fonts.forEach((face) => faces.push({ family: face.family, weight: face.weight,
      style: face.style, status: face.status }));
    return { url: location.href, status: document.fonts.status, faces };
  });
}
const timeoutMs = 300_000;
type ImportDebuggerSession = Awaited<ReturnType<BrowserContext['newCDPSession']>>;
type NodeDeadlineOutcome<T> = { status: 'completed'; value: T } | { status: 'rejected' } | { status: 'deadline' };
type ImportDebuggerPauseObservation = { kind: 'paused'; event: unknown } |
  { kind: 'command-rejected' } | { kind: 'pause-event-deadline' };
async function settleBeforeNodeDeadline<T>(operation: Promise<T>, deadlineAt: number): Promise<NodeDeadlineOutcome<T>> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const settled: Promise<NodeDeadlineOutcome<T>> = operation.then(
    (value) => ({ status: 'completed', value }), () => ({ status: 'rejected' }));
  const deadline = new Promise<NodeDeadlineOutcome<T>>((resolve) => {
    timer = setTimeout(() => resolve({ status: 'deadline' }), Math.max(0, deadlineAt - Date.now()));
  });
  try { return await Promise.race([settled, deadline]); }
  finally { if (timer) clearTimeout(timer); }
}
async function releaseImportDebuggerSession(session: ImportDebuggerSession, resumeDeadlineAt: number,
  totalDeadlineAt: number): Promise<RecordValue> {
  const resume = await settleBeforeNodeDeadline(Promise.resolve().then(() => session.send('Debugger.resume')), resumeDeadlineAt);
  const detach = await settleBeforeNodeDeadline(Promise.resolve().then(() => session.detach()), totalDeadlineAt);
  return { resumeStatus: resume.status, detachStatus: detach.status };
}
function safeDebuggerScriptURL(value: unknown): string {
  if (typeof value !== 'string' || !value) return '';
  try {
    const parsed = new URL(value);
    if (parsed.protocol === 'http:' || parsed.protocol === 'https:') return `${parsed.origin}${parsed.pathname}`;
    if (parsed.protocol === 'file:') return `file:${parsed.pathname}`;
    if (parsed.protocol === 'about:') return `about:${parsed.pathname}`;
    if (parsed.protocol === 'blob:') return 'blob:';
    if (parsed.protocol === 'data:') return 'data:';
    if (parsed.protocol === 'webpack:' || parsed.protocol === 'chrome-extension:')
      return `${parsed.protocol}//${parsed.host}${parsed.pathname}`;
    return parsed.protocol;
  } catch { return ''; }
}
function summarizeDebuggerCallFrames(event: unknown, scriptURLs: ReadonlyMap<string, string>): RecordValue {
  const payload = event && typeof event === 'object' && !Array.isArray(event) ? event as RecordValue : {};
  const rawFrames = payload['callFrames'];
  const frameIndices = Array.isArray(rawFrames)
    ? rawFrames.length <= 32 ? rawFrames.map((_, index) => index)
      : Array.from({ length: 32 }, (_, index) => index < 16 ? index : rawFrames.length - 32 + index)
    : [];
  const frames = frameIndices.map((frameIndex): RecordValue => {
    const value = Array.isArray(rawFrames) ? rawFrames[frameIndex] : undefined;
    const frame = value && typeof value === 'object' && !Array.isArray(value) ? value as RecordValue : {};
    const locationValue = frame['location'];
    const location = locationValue && typeof locationValue === 'object' && !Array.isArray(locationValue)
      ? locationValue as RecordValue : {};
    const lineNumber = location['lineNumber'];
    const columnNumber = location['columnNumber'];
    const scriptId = location['scriptId'];
    const mappedScriptURL = typeof scriptId === 'string' || typeof scriptId === 'number'
      ? scriptURLs.get(String(scriptId)) ?? '' : '';
    return { frameIndex, functionName: typeof frame['functionName'] === 'string' ? frame['functionName'] : '',
      scriptURL: safeDebuggerScriptURL(frame['url']) || mappedScriptURL,
      location: { lineNumber: typeof lineNumber === 'number' && Number.isFinite(lineNumber) ? lineNumber : null,
        columnNumber: typeof columnNumber === 'number' && Number.isFinite(columnNumber) ? columnNumber : null } };
  });
  return { callFramesAvailable: Array.isArray(rawFrames), callFrameCount: Array.isArray(rawFrames) ? rawFrames.length : 0,
    callFramesTruncated: Array.isArray(rawFrames) && rawFrames.length > frames.length, callFrames: frames };
}

async function captureImportDebuggerStack(session: ImportDebuggerSession, scriptURLs: ReadonlyMap<string, string>,
  importDeadlineAt: number): Promise<RecordValue> {
  const startedAt = Date.now();
  const totalDeadlineAt = Math.min(startedAt + 10_000, importDeadlineAt);
  const availableMs = Math.max(0, totalDeadlineAt - startedAt);
  const resumeBudgetMs = Math.min(1_000, availableMs / 4);
  const latePauseWindowMs = Math.min(500, availableMs / 8);
  const lateResumeBudgetMs = Math.min(1_000, availableMs / 4);
  const detachBudgetMs = Math.min(1_500, availableMs / 4);
  const pauseWindowMs = Math.max(0, Math.min(6_000,
    availableMs - resumeBudgetMs - latePauseWindowMs - lateResumeBudgetMs - detachBudgetMs));
  const resumeDeadlineAt = Math.min(totalDeadlineAt, startedAt + pauseWindowMs + resumeBudgetMs);
  const latePauseWindowDeadlineAt = Math.min(totalDeadlineAt,
    startedAt + pauseWindowMs + resumeBudgetMs + latePauseWindowMs);
  const lateResumeDeadlineAt = Math.min(totalDeadlineAt,
    startedAt + pauseWindowMs + resumeBudgetMs + latePauseWindowMs + lateResumeBudgetMs);
  let pauseTimer: ReturnType<typeof setTimeout> | undefined;
  let pausedEvent: unknown;
  let pausedEventAt: number | undefined;
  let commandAcknowledged = false;
  let cleanupStarted = false;
  let latePauseObservedDuringCleanup = false;
  let resolveLatePause: (() => void) | undefined;
  let lateResumePromise: Promise<NodeDeadlineOutcome<unknown>> | undefined;
  const latePauseSignal = new Promise<void>((resolve) => { resolveLatePause = resolve; });
  const startLateResume = (): void => {
    if (Date.now() >= lateResumeDeadlineAt) return;
    lateResumePromise ??= settleBeforeNodeDeadline(
      Promise.resolve().then(() => session.send('Debugger.resume')), lateResumeDeadlineAt);
  };
  let status = pauseWindowMs > 0 ? 'pause-event-not-observed-before-deadline' :
    'skipped-no-pause-window-before-import-deadline';
  if (pauseWindowMs > 0) {
    try {
      const pauseEvent = new Promise<ImportDebuggerPauseObservation>((resolve) => {
        session.on('Debugger.paused', (event: unknown) => {
          pausedEvent = event;
          pausedEventAt = Date.now();
          if (cleanupStarted) {
            latePauseObservedDuringCleanup = true;
            resolveLatePause?.();
            startLateResume();
          }
          resolve({ kind: 'paused', event });
        });
      });
      const pauseCommand = Promise.resolve().then(() => session.send('Debugger.pause'));
      const pauseSignal: Promise<ImportDebuggerPauseObservation> = pauseCommand.then(
        () => { commandAcknowledged = true; return new Promise<ImportDebuggerPauseObservation>(() => {}); },
        () => ({ kind: 'command-rejected' }));
      const pauseDeadline = new Promise<ImportDebuggerPauseObservation>((resolve) => {
        pauseTimer = setTimeout(() => resolve({ kind: 'pause-event-deadline' }),
          Math.max(0, Math.min(startedAt + pauseWindowMs, totalDeadlineAt -
            resumeBudgetMs - latePauseWindowMs - lateResumeBudgetMs - detachBudgetMs) - Date.now()));
      });
      const observation = await Promise.race([pauseEvent, pauseSignal, pauseDeadline]);
      status = observation.kind === 'paused' ? 'pause-event-observed' :
        observation.kind === 'command-rejected' ? 'pause-command-rejected' : 'pause-event-not-observed-before-deadline';
    } catch { status = 'pause-observation-error'; }
  }
  if (pauseTimer) clearTimeout(pauseTimer);
  cleanupStarted = true;
  const resume = await settleBeforeNodeDeadline(Promise.resolve().then(() => session.send('Debugger.resume')), resumeDeadlineAt);
  const latePauseWaitStatus = pausedEvent === undefined
    ? (await settleBeforeNodeDeadline(latePauseSignal, latePauseWindowDeadlineAt)).status
    : latePauseObservedDuringCleanup ? 'late-pause-observed' : 'not-needed-event-already-observed';
  if (lateResumePromise) await lateResumePromise;
  const detach = await settleBeforeNodeDeadline(Promise.resolve().then(() => session.detach()), totalDeadlineAt);
  if (lateResumePromise) await lateResumePromise;
  if (pausedEvent !== undefined && status !== 'pause-event-observed') status = 'pause-event-arrived-during-cleanup-window';
  const frameSummary = pausedEvent === undefined ? { callFramesAvailable: false, callFrameCount: 0,
    callFramesTruncated: false, callFrames: [] } : summarizeDebuggerCallFrames(pausedEvent, scriptURLs);
  return { status, pauseWindowMs, diagnosticDeadlineMs: 10_000, actualDeadlineAtImportMs: availableMs,
    pauseCommandAcknowledged: commandAcknowledged, pauseEventObserved: pausedEvent !== undefined,
    pauseEventObservedAtUtc: pausedEventAt === undefined ? null : new Date(pausedEventAt).toISOString(),
    ...frameSummary, cleanup: { resumeStatus: resume.status, latePauseWindowStatus: latePauseWaitStatus,
      latePauseResumeStatus: lateResumePromise ? (await lateResumePromise).status : 'not-requested',
      detachStatus: detach.status }, elapsedMs: Date.now() - startedAt, deadlineExceeded: Date.now() > totalDeadlineAt };
}

async function captureFrameStyles(frame: Frame | FrameLocator): Promise<unknown> {
  return frame.locator('html').evaluate(() => {
    const sample = (selector: string) => {
      const element = document.querySelector(selector);
      if (!element) return null;
      const style = getComputedStyle(element);
      return { selector, color: style.color, backgroundColor: style.backgroundColor,
        fontFamily: style.fontFamily, fontSize: style.fontSize, display: style.display,
        theme: element.getAttribute('data-theme'), foreground: style.getPropertyValue('--foreground-primary'),
        accent: style.getPropertyValue('--accent-primary') };
    };
    return { url: location.href, readyState: document.readyState,
      links: Array.from(document.querySelectorAll<HTMLLinkElement>('link[rel="stylesheet"]')).map((link) => ({
        href: link.href, media: link.media, applies: matchMedia(link.media || 'all').matches,
        sheetPresent: Boolean(link.sheet), onload: link.getAttribute('onload') })),
      sheets: Array.from(document.styleSheets).map((sheet) => {
        try { return { href: sheet.href, disabled: sheet.disabled, rules: sheet.cssRules.length }; }
        catch (error) { return { href: sheet.href, error: String(error) }; }
      }), samples: ['body', 'app-root', 'h1', 'button[data-appearance="primary"]'].map(sample) };
  });
}
const required = (key: string): string => {
  const value = process.env[key];
  assert.ok(value, `Set ${key} for the actual locally running Penpot session; real mode has no mock fallback.`);
  return value;
};

function artifactFile(file: string): string {
  const path = resolve(file);
  requireRealSessionDirectory(dirname(path));
  assert.equal(realpathSync(path).toLowerCase(), path.toLowerCase(), 'Artifact file aliases are not allowed');
  assert.ok(statSync(path).isFile());
  return path;
}

/** Serve only the independently built plugin; Penpot API/assets/WebSocket remain untouched. */
async function pluginServer(dist: string, bundle: string): Promise<{ server: Server; host: string }> {
  const mime: Record<string, string> = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css',
    '.json': 'application/json', '.svg': 'image/svg+xml', '.png': 'image/png', '.woff2': 'font/woff2' };
  const server = createServer(async (request, response) => {
    try {
      const url = new URL(request.url ?? '/', 'http://127.0.0.1');
      const pathname = decodeURIComponent(url.pathname);
      const file = pathname === '/assets/plugin.js' ? bundle : resolve(dist, `.${pathname === '/' ? '/index.html' : pathname}`);
      const rel = relative(dist, file);
      assert.ok(file === bundle || (rel !== '..' && !rel.startsWith(`..${sep}`) && !rel.includes(':')));
      const bytes = await readFile(file);
      response.writeHead(200, { 'Content-Type': mime[extname(file)] ?? 'application/octet-stream',
        'Access-Control-Allow-Origin': '*', 'Cache-Control': 'no-store',
        'Cross-Origin-Embedder-Policy': 'require-corp', 'Cross-Origin-Resource-Policy': 'cross-origin' });
      response.end(bytes);
    } catch {
      response.writeHead(404);
      response.end();
    }
  });
  await new Promise<void>((resolveReady, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => { server.removeListener('error', reject); resolveReady(); });
  });
  const address = server.address();
  assert.ok(address && typeof address !== 'string');
  return { server, host: `http://127.0.0.1:${address.port}/` };
}

function transitDecoder(frontendPackage: string): (body: string) => unknown {
  const transit = createRequire(frontendPackage)('transit-js') as TransitLibrary;
  // Official shape readers call map->Shape; unset fixed record fields remain present as nil.
  const shapeFields = ['id', 'name', 'type', 'x', 'y', 'width', 'height', 'rotation', 'selrect',
    'points', 'transform', 'transform-inverse', 'parent-id', 'frame-id', 'flip-x', 'flip-y'];
  const reader = transit.reader('json', { handlers: { shape: (value: unknown) => {
    const record: RecordValue = Object.fromEntries(shapeFields.map((field) => [field, null]));
    (value as TransitMap).forEach((entry: unknown, key: unknown) => { record[String(key).replace(/^:/, '')] = entry; });
    return record;
  } }, defaultHandler: (tag: string, value: unknown) => ({ tag, value }) });
  const plain = (value: unknown): unknown => {
    if (Array.isArray(value)) return value.map(plain);
    if (transit.isSet(value)) {
      const entries: unknown[] = [];
      (value as TransitMap).forEach((entry: unknown) => entries.push(plain(entry)));
      return new Set(entries);
    }
    if (value && typeof value === 'object' && 'forEach' in value && typeof value.forEach === 'function') {
      const result: RecordValue = {};
      (value as TransitMap).forEach((entry: unknown, key: unknown) => { result[String(key).replace(/^:/, '')] = plain(entry); });
      return result;
    }
    if (value && typeof value === 'object') {
      const string = String(value);
      if (string !== '[object Object]') return string.replace(/^:/, '');
      return Object.fromEntries(Object.entries(value).map(([key, entry]) => [key, plain(entry)]));
    }
    return value;
  };
  return (body) => plain(reader.read(body));
}

async function rpc<T = IdentifiedRecord>(context: { request: Pick<APIRequestContext, 'post'> }, origin: string, command: string, params: RecordValue,
  decode: (body: string) => unknown): Promise<T> {
  const response = await context.request.post(`${origin}/api/rpc/command/${command}`, {
    data: params, headers: { accept: 'application/transit+json' }, timeout: 30_000,
  });
  assert.ok(response.ok(), `${command} failed with actual HTTP ${response.status()}`);
  const type = response.headers()['content-type'];
  if (type?.includes('application/transit+json')) return decode(await response.text()) as T;
  if (type?.includes('application/json')) return response.json();
  throw new Error(`${command} returned an unsupported actual response content type`);
}

/** Fence an official export with authenticated file/library revisions. This
 * supplies the authority absent from SDK Library and never writes sync status.
 * A concurrent or unacknowledged save remains a failure to reconcile.
 */
export async function withSourceColorRevisionFence<T>(
  context: { request: Pick<APIRequestContext, 'post'> },
  identity: { origin: string; fileId: string; pageId: string }, boardId: string,
  document: ZuiDocument, decode: (body: string) => unknown, receipt: RecordValue,
  operation: () => Promise<T>,
): Promise<T> {
  const fences = receipt['sourceColorRevisionFences'] ?? [];
  assert.ok(Array.isArray(fences)); receipt['sourceColorRevisionFences'] = fences;
  const fence: RecordValue = { boardId, startedAtUtc: new Date().toISOString(), status: 'reading-before' };
  fences.push(fence);
  const readSnapshot = async () => {
    const file = await rpc(context, identity.origin, 'get-file', { id: identity.fileId }, decode);
    assert.equal(file.id, identity.fileId);
    const connected = await rpc<IdentifiedRecord[]>(context, identity.origin, 'get-file-libraries',
      { 'file-id': identity.fileId }, decode);
    assert.ok(Array.isArray(connected), 'Read the actual connected library list');
    assert.equal(new Set(connected.map(library => library.id)).size, connected.length);
    const libraries = await Promise.all(connected.map(async library => {
      const actual = await rpc(context, identity.origin, 'get-file', { id: library.id }, decode);
      assert.equal(actual.id, library.id, 'Color library API returned another file'); return actual;
    }));
    return sourceColorRevisionSnapshot(file, libraries, identity.pageId, boardId, document);
  };
  try {
    const before = await readSnapshot(); fence['before'] = before; fence['status'] = 'exporting';
    const result = await operation();
    const after = await readSnapshot(); fence['after'] = after;
    assertSourceColorRevisionPreserved(before, after);
    fence['status'] = 'exported-with-stable-source-color-revisions'; return result;
  } catch (error) {
    fence['status'] = 'failed'; throw error;
  } finally { fence['finishedAtUtc'] = new Date().toISOString(); }
}

async function openWorkspace(context: BrowserContext, workspaceUrl: string, host: string, manifest: RecordValue,
  bundleSha256: string, errors: string[], decode: (body: string) => unknown,
  beforePluginOpen?: (page: Page) => Promise<void>,
  reuseRegistration?: { pluginId: string; registryEntrySha256: string; registrySha256?: string },
  executionMode: 'reuse-existing' | 'managed-bootstrap' = 'reuse-existing'): Promise<{ page: Page;
    frame: ReturnType<Page['frameLocator']>; installation: RecordValue }> {
  assert.ok(reuseRegistration || executionMode === 'managed-bootstrap',
    'Opening an unregistered plugin requires explicit managed-bootstrap mode');
  assert.ok(!reuseRegistration || executionMode === 'reuse-existing', 'Bootstrap cannot adopt an existing registration');
  const beforeRegistration = reuseRegistration ? verifyExistingPluginRegistration(
    await rpc(context, workspaceIdentity(workspaceUrl).origin, 'get-profile', {}, decode),
    new URL('manifest.json', host).href, manifest['permissions'], reuseRegistration) : undefined;
  const pinnedRegistration = reuseRegistration && beforeRegistration
    ? { ...reuseRegistration, registrySha256: beforeRegistration.registrySha256 } : undefined;
  const page = await context.newPage();
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => {
    const text = message.text();
    if (!text.startsWith('[zircon-import-error]')) return;
    // Collect only this plugin's bounded failure stack; preserve the UI/save gates.
    errors.push(text.slice(0, 8192).replace(/https?:\/\/[^\s)]+/g, safeDebuggerScriptURL));
  });
  page.on('requestfailed', (request) => errors.push(`Request failed: ${request.url()} ${request.failure()?.errorText}`));
  let navigationUrl = workspaceUrl;
  if (process.argv.includes('--wasm-renderer')) {
    const url = new URL(workspaceUrl);
    const question = url.hash.indexOf('?');
    assert.ok(question >= 0, 'The actual workspace URL must retain its route and identity');
    const params = new URLSearchParams(url.hash.slice(question + 1));
    params.set('wasm', 'true');
    url.hash = url.hash.slice(0, question) + '?' + params.toString();
    navigationUrl = url.href;
  }
  await page.goto(navigationUrl, { timeout: timeoutMs, waitUntil: 'domcontentloaded' });
  assert.deepEqual(workspaceIdentity(page.url()), workspaceIdentity(workspaceUrl), 'Penpot redirected away from the actual file');
  await page.getByTestId('viewport').waitFor({ timeout: timeoutMs });
  if (beforePluginOpen) await beforePluginOpen(page);
  assert.ok(typeof manifest['name'] === 'string' && typeof manifest['code'] === 'string');
  const servedBundle = await context.request.get(new URL(manifest['code'], host).href);
  assert.equal(servedBundle.status(), 200);
  assert.equal(sha256(await servedBundle.body()), bundleSha256);
  // Explicit reuse never installs a registry entry or grants permissions again.
  await page.locator('[data-tool="plugins"]').click();
  if (!pinnedRegistration) {
    assert.equal(executionMode, 'managed-bootstrap', 'UI installation is bootstrap-only');
    await page.getByPlaceholder('Write a plugin URL').fill(new URL('manifest.json', host).href);
    await page.getByRole('button', { name: 'Install', exact: true }).click();
    await page.getByRole('button', { name: 'Allow', exact: true }).click();
  }
  const pluginRows = page.locator('[class*="plugins-list-element"]');
  await pluginRows.first().waitFor({ timeout: 30_000 });
  const manifestUrl = new URL('manifest.json', host).href;
  let installedPlugin: RecordValue | undefined;
  let installedIndex = -1;
  const registrationDeadline = Date.now() + 30_000;
  while (Date.now() < registrationDeadline) {
    const profile = await rpc<RecordValue>(context, workspaceIdentity(workspaceUrl).origin, 'get-profile', {}, decode);
    const props = record(profile['props']);
    const registry = props['plugins'] ? record(props['plugins']) : undefined;
    const entries = registry?.['data'] ? Object.values(record(registry['data'])) : [];
    installedPlugin = entries.map(record).find((entry) => entry['url'] === manifestUrl);
    if (installedPlugin) {
      const ids = registry?.['ids'];
      assert.ok(Array.isArray(ids), 'The official plugin registry must persist its ordered ID vector');
      installedIndex = ids.indexOf(installedPlugin['plugin-id']);
      break;
    }
    await page.waitForTimeout(250);
  }
  assert.ok(installedPlugin, 'Official UI installation must persist the plugin registry in the real account');
  assert.ok(typeof installedPlugin['plugin-id'] === 'string'
    && installedPlugin['plugin-id'] !== '00000000-0000-0000-0000-000000000000');
  assert.ok(Array.isArray(manifest['permissions']), 'Manifest permissions must be the actual JSON array');
  const installedPermissions = pluginRegistrationPermissions(installedPlugin['permissions']);
  assert.deepEqual(installedPermissions, pluginRegistrationPermissions(manifest['permissions']));
  assert.ok(installedIndex >= 0);
  if (pinnedRegistration) {
    const profile = await rpc<RecordValue>(context, workspaceIdentity(workspaceUrl).origin, 'get-profile', {}, decode);
    const verified = verifyExistingPluginRegistration(profile, manifestUrl, manifest['permissions'], pinnedRegistration);
    assert.equal(verified.index, installedIndex);
  }
  const pluginRow = pluginRows.nth(installedIndex);
  assert.ok((await pluginRow.innerText()).includes(manifest['name']));
  await pluginRow.getByRole('button', { name: /^(?:Open|打开)$/ }).click();
  const frame = page.frameLocator('plugin-modal iframe');
  await frame.locator('input[type="file"][accept*=".zui"]').first().waitFor({ state: 'attached', timeout: 30_000 });
  await frame.getByRole('button', { name: 'Import .zui', exact: true }).waitFor({ timeout: 30_000 });
  if (pinnedRegistration) {
    const profile = await rpc<RecordValue>(context, workspaceIdentity(workspaceUrl).origin, 'get-profile', {}, decode);
    verifyExistingPluginRegistration(profile, manifestUrl, manifest['permissions'], pinnedRegistration);
  }
  return { page, frame, installation: { registryBackendVerified: true, manifestUrl,
    reusedExistingRegistration: Boolean(reuseRegistration), installedByThisRun: !reuseRegistration,
    pluginId: installedPlugin['plugin-id'], permissions: installedPermissions,
    registryFingerprintProtocol: pluginRegistrationFingerprintProtocol,
    registryEntrySha256: pluginRegistrationFingerprint(installedPlugin),
    registrySha256: beforeRegistration?.registrySha256, registryEntryCount: beforeRegistration?.entryCount } };
}

async function selectActualAssetBoard(page: Page, boardId: string, receipt: RecordValue): Promise<void> {
  assert.match(boardId, /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i);
  assert.equal(await page.locator('plugin-modal').count(), 0);
  const row = page.locator(`[data-testid="layer-row"][id="${boardId}"]`);
  await row.waitFor({ timeout: 30_000 });
  const label = await row.innerText();
  await row.click({ timeout: 30_000 });
  await row.and(page.locator('[aria-checked="true"]')).waitFor({ timeout: 5_000 });
  receipt['reopenedCanvasSelectedViaUiBeforePluginOpen'] = true;
  receipt['actualAssetBoardLayerSelection'] = { boardId, label,
    classAfterClick: await row.getAttribute('class'), ariaCheckedAfterClick: await row.getAttribute('aria-checked'),
    selectionMethod: 'official layer-row click' };
}

async function capturePreviewLayoutAudit(
  frame: FrameLocator, expectedBoardId: string, output: string, receipt: RecordValue, expectedInput: ZuiDocument,
): Promise<RecordValue> {
  const source = reviewHost(expectedInput) ?? expectedInput;
  const projectedNodeCount = projectZuiDocument(source).shapes.length;
  const auditTimeoutMs = previewLayoutAuditTimeoutForNodes(projectedNodeCount);
  receipt['postReopenLayoutAuditRequest'] = {
    projectedNodeCount,
    timeoutMs: auditTimeoutMs,
    budgetAuthority: 'two-native-settlement-phases-and-minimum-refresh-draft',
  };
  const reply = await requestPreviewLayoutAudit(frame, auditTimeoutMs);
  const response = record(reply);
  const requestId = reply.requestId;
  assert.equal(response['type'], 'preview-audit');
  assert.equal(response['previewBoardId'], expectedBoardId,
    'The layout audit must describe the same saved and reopened asset board');
  const layoutAudit = record(response['layoutAudit']);
  const path = resolve(output, 'post-reopen-layout-audit.json');
  await writeFile(path, `${JSON.stringify(response, null, 2)}\n`);
  receipt['postReopenLayoutAudit'] = { path, sha256: sha256(await readFile(path)),
    boardId: response['previewBoardId'], requestId, authority: 'actual-plugin-layout-audit-after-reopen',
    settled: !Object.hasOwn(layoutAudit, 'refreshError') };
  assert.ok(!Object.hasOwn(layoutAudit, 'refreshError'), 'The reopened board layout refresh failed; see the saved audit');
  assert.ok(Array.isArray(layoutAudit['contentMeasurements']), 'Actual text measurements are missing from the reopened audit');
  assert.ok(Array.isArray(layoutAudit['semanticNodes']), 'Actual semantic bounds are missing from the reopened audit');
  record(layoutAudit['assetBounds']);
  const actual = importedActualNormalReviewCase(source);
  const media = actual ? runtimeSceneMediaForCase(actual) : undefined;
  if (media && actual) {
    const target = runtimeSceneMediaReviewTarget(source, actual)!;
    const audit = record(layoutAudit['runtimeSceneMediaAudit']);
    assert.equal(audit['schema'], 'dev.zircon.penpot.runtime-scene-media-audit');
    assert.equal(audit['version'], 1);
    assert.equal(audit['authority'], 'official-original-media-data-after-layout-refresh');
    assert.equal(audit['verified'], true);
    assert.equal(audit['caseId'], actual.id);
    assert.equal(audit['nodeId'], target.nodeId);
    assert.equal(audit['pngSha256'], media.image.pngSha256);
    assert.equal(audit['byteLength'], media.image.byteLength);
    assert.deepEqual(audit['physicalSize'], target.image.runtimeScene.physicalSize);
    assert.equal(audit['bootstrapCaseJsonSha256'], media.bootstrapCase.caseJsonSha256);
    assert.equal(audit['bootstrapCaseSha256'], media.bootstrapCase.caseSha256);
    assert.equal(audit['bootstrapPixelAssociationSha256'], media.bootstrapPixelAssociation.sha256);
    assert.ok(typeof audit['mediaId'] === 'string' && audit['mediaId'].length > 0);
    assert.ok(typeof audit['shapeId'] === 'string' && audit['shapeId'].length > 0);
    const owners = (layoutAudit['semanticNodes'] as unknown[]).map(record).filter((node) => node['nodeId'] === target.nodeId);
    assert.equal(owners.length, 1, 'Reopened scene needs one actual semantic owner');
    assert.equal(owners[0]!['shapeId'], audit['ownerShapeId']);
    receipt['runtimeSceneMediaAudit'] = audit;
  }
  return layoutAudit;
}

async function captureExportUi(frame: FrameLocator, page: Page): Promise<RecordValue> {
  const button = frame.getByRole('button', { name: 'Export selected' });
  await button.waitFor({ timeout: 30_000 });
  return { enabled: await button.isEnabled(),
    selectionBand: await frame.locator('.selection-band').innerText({ timeout: 5_000 }),
    status: await frame.locator('.status-panel').innerText({ timeout: 5_000 }),
    layers: await page.getByTestId('layer-row').evaluateAll((rows) => rows.map((row) => ({
      id: row.id, className: row.className, ariaChecked: row.getAttribute('aria-checked'),
      text: (row.textContent ?? '').slice(0, 200),
    }))) };
}

function inspectCanvasPng(bytes: Buffer): { width: number; height: number } {
  const signature = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  assert.ok(bytes.byteLength >= 24, 'Canvas PNG download is shorter than its IHDR header');
  assert.deepEqual(bytes.subarray(0, 8), signature, 'Canvas PNG download has an invalid PNG signature');
  assert.equal(bytes.toString('ascii', 12, 16), 'IHDR', 'Canvas PNG download is missing the PNG IHDR chunk');
  const width = bytes.readUInt32BE(16);
  const height = bytes.readUInt32BE(20);
  assert.ok(width > 0 && height > 0, 'Canvas PNG download has empty dimensions');
  return { width, height };
}

async function downloadActualCanvasPng(
  page: Page,
  frame: FrameLocator,
  expectedBoardId: string,
  output: string,
  receipt: RecordValue,
): Promise<void> {
  const exportFeatures = await page.evaluate(() => {
    const runtime = globalThis as unknown as {
      features?: { get_team_enabled?: () => unknown };
    };
    const active = runtime.features?.get_team_enabled?.();
    return {
      officialReadbackAvailable: Array.isArray(active),
      renderWasmActive: Array.isArray(active) && active.includes('render-wasm/v1'),
      wasmExportActive: Array.isArray(active) && active.includes('wasm-export/v1'),
    };
  });
  receipt['canvasPngExportFeatures'] = exportFeatures;
  if (process.argv.includes('--wasm-renderer')) {
    assert.ok(exportFeatures.officialReadbackAvailable
      && exportFeatures.renderWasmActive && exportFeatures.wasmExportActive,
    'The actual reopened page must enable both official WASM render and PNG export features');
  }
  const button = frame.getByRole('button', { name: 'Export Canvas PNG', exact: true });
  await button.waitFor({ timeout: 30_000 });
  assert.equal(await button.isEnabled(), true, 'The reopened saved Board does not enable Canvas PNG export');
  const pluginSelectedBoardId = await frame.locator('.selection-band').getAttribute('data-selected-board-id');
  assert.equal(pluginSelectedBoardId, expectedBoardId,
    'The plugin selection does not match the saved Board selected through the reopened Penpot layer tree');
  receipt['canvasPngExportUiSelection'] = {
    boardIdMatchedPersistedMetadata: true,
    buttonEnabled: true,
  };
  const download = await clickAndWaitForDownload(
    () => page.waitForEvent('download', { timeout: timeoutMs }),
    () => button.click(),
  );
  const path = resolve(output, 'canvas-export.png');
  await download.saveAs(path);
  assert.equal(await download.failure(), null);
  const bytes = await readFile(path);
  const dimensions = inspectCanvasPng(bytes);
  const hostExportedBoardId = await frame.locator('.selection-band').getAttribute('data-canvas-png-board-id');
  assert.equal(hostExportedBoardId, expectedBoardId, 'Canvas PNG bytes belong to a different persisted Board');
  receipt['canvasPngExport'] = {
    path,
    fileName: basename(download.suggestedFilename()),
    sha256: sha256(bytes),
    byteLength: bytes.byteLength,
    width: dimensions.width,
    height: dimensions.height,
    boardId: expectedBoardId,
    hostExportedBoardId,
    exportMethod: 'Penpot Shape.export({type: png, scale: 1})',
    scale: 1,
    downloadObserved: true,
    validPngSignatureAndIhdr: true,
  };
}

/** Compare the actual saved root case and original group namespace before export can reflow it. */
function persistedToolbarGroups(file: BackendFile, pageId: string,
  asset: ReturnType<typeof persistedZuiMetadata>[number], source: ZuiDocument) {
  const host = reviewHost(source) ?? source, actual = importedActualNormalReviewCase(host);
  if (actual) {
    const encoded = asset.metadata['review-case'];
    assert.equal(typeof encoded, 'string', 'Actual saved normal review lacks its Board case metadata');
    const restored = reviewCaseFromSavedMetadata(host, encoded as string);
    assert.deepEqual(restored, actual, 'Actual saved Board case differs from the imported complete native observation');
  }
  const contract = viewportToolbarContract(host);
  if (!contract) return [];
  const groups = persistedViewportToolbarGroupMetadata(file, pageId, asset.boardId, contract);
  const mapping = actual ? viewportToolbarSourceMapping(host, actual.data['normal_window_evidence'], actual.dpi) : undefined;
  const packet = mapping?.packet;
  if (actual) assert.ok(packet, 'Actual imported normal review lacks its exact toolbar publication');
  for (const [index, group] of groups.entries()) {
    const node = host.nodes?.[group.nodeId];
    assert.ok(node, 'Actual original toolbar group is absent from Source');
    assert.equal(group.metadata['controlId'], node.control_id ?? '', 'Actual saved group control differs from Source');
    for (const [metadataKey, sourceKey] of [['sourcePath', 'penpot_review_source_path'],
      ['sourceNodeId', 'penpot_review_source_node_id'], ['instancePath', 'penpot_review_instance_path']] as const) {
      const expected: unknown = node[sourceKey];
      if (actual) assert.equal(typeof expected, 'string', 'Actual original group lacks its complete Source tuple');
      if (typeof expected === 'string') assert.equal(group.metadata[metadataKey], expected,
        'Actual saved group differs from its complete Source tuple');
    }
    if (packet) {
      const raw = packet.nodes.find(node => node.nodeId === packet.groupNodeIds[index]);
      assert.ok(raw, 'Actual original group publication is missing');
      assert.equal(mapping!.sourceByNativeId.get(raw.nodeId), group.nodeId,
        'Actual saved group differs from its complete authenticated Source identity');
      const expectedParent: string | undefined = raw.parentNodeId === packet.rowNodeId ? contract.row
        : raw.parentNodeId === packet.popupNodeId ? contract.popup : undefined;
      assert.ok(expectedParent, 'Actual original group has an unrelated published parent');
      assert.equal(group.parentNodeId, expectedParent, 'Actual saved group parent differs from its native observation');
    }
  }
  return groups;
}

export interface RealPenpotPersistenceOptions {
  mode?: 'reuse-existing' | 'managed-bootstrap';
  expectedWorkspace?: { origin: string; teamId: string; fileId: string; pageId: string };
  existingPlugin?: { host: string; pluginId: string; registryEntrySha256: string; manifestSha256: string;
    bundleSha256: string; registrySha256?: string };
  append?: { fileId: string; pageId: string; revn: number; vern: number };
}

/** Actual import -> HTTP save acknowledgement -> fresh context -> backend metadata -> existing export. */
export async function runRealPenpotPersistence(options: RealPenpotPersistenceOptions = {}): Promise<void> {
  const editSourceGap = process.argv.includes('--edit-source-gap');
  assert.ok(['--edit-outliner-min-height', '--edit-workbench-gap', '--edit-source-gap']
    .filter(flag => process.argv.includes(flag)).length <= 1, 'Choose exactly one real Inspector edit.');
  const root = requireRealSessionDirectory(required('PENPOT_REAL_EVIDENCE_ROOT'));
  const output = resolve(root, `session-${randomUUID()}`);
  await mkdir(output);
  requireRealSessionDirectory(output);
  const receiptPath = resolve(output, 'real-session-receipt.json');
  const pageErrors: string[] = [];
  const receipt: RecordValue = { schema: 'dev.zircon.penpot.real-session-persistence', version: 1,
    generatedAtUtc: new Date().toISOString(), backend: 'real', status: 'pending', persistenceClaim: false,
    saveAcknowledged: false, closedOriginalContext: false, newAuthenticatedContext: false,
    reopenedSameFile: false, metadataMatched: false, exportSemanticMatch: false, sourceFingerprintMatched: false,
    saveResponses: [], pageErrors, protocolSource: 'Penpot 2.17.0 files.clj/files_update.clj/middleware.clj' };
  let receiptSequence = 0;
  let receiptWrites: Promise<void> = Promise.resolve();
  const receiptTemporaryPath = `${receiptPath}.tmp`;
  const checkpointReceipt = (stage: string): Promise<void> => {
    assert.match(stage, /^[a-z][a-z0-9-]*$/);
    const sequence = ++receiptSequence;
    const checkpoint = { sequence, stage, observedAtUtc: new Date().toISOString() };
    // Capture now: concurrent HTTP observers must not rewrite an earlier stage's facts.
    const snapshot = `${JSON.stringify({ ...receipt, status: receipt['status'] === 'failed' ? 'failed' : 'pending',
      persistenceClaim: false, checkpoint }, null, 2)}\n`;
    const checkpointPath = resolve(output, `real-session-checkpoint-${String(sequence).padStart(4, '0')}-${stage}.json`);
    const queued = receiptWrites.then(async () => {
      await writeFile(checkpointPath, snapshot, { flag: 'wx', flush: true });
      await writeFile(receiptTemporaryPath, snapshot, { flush: true });
      await rename(receiptTemporaryPath, receiptPath);
      console.log(JSON.stringify({ realSessionReceipt: receiptPath, checkpointReceipt: checkpointPath,
        stage, status: 'pending', persistenceClaim: false }));
    });
    receiptWrites = queued.catch(() => undefined);
    return queued;
  };

  const fontResponses: RecordValue[] = [];
  const fontPending = new Set<Promise<void>>();
  const savePending = new Set<Promise<void>>();
  const saveHttpResponses: RecordValue[] = [];
  receipt['saveHttpResponses'] = saveHttpResponses;
  receipt['fontResponses'] = fontResponses;
  receipt['fontCanvasBufferRegistrationVerified'] = false;
  const watchFonts = (target: BrowserContext, captureContext: string): void => {
    target.on('response', (response) => {
      const url = response.url();
      const candidate = /\/internal\/gfonts\/(font|css)|\/assets\/by-id\/|fonts\.gstatic\.com|noto.*\.(woff2?|ttf|otf)/i.test(url)
        || response.request().resourceType() === 'font';
      if (!candidate) return;
      const pending = (async () => {
        try {
          const bytes = await response.body();
          const magic = bytes.subarray(0, 4).toString('ascii');
          const font = ['OTTO', 'ttcf', 'wOFF', 'wOF2'].includes(magic)
            || (bytes.length >= 4 && bytes.readUInt32BE(0) === 0x00010000);
          if (!font && /\/assets\/by-id\//.test(url)) return;
          const file = `font-http-response-${fontResponses.length + 1}-${randomUUID()}.bin`;
          await writeFile(resolve(output, file), bytes);
          fontResponses.push({ captureContext, url, httpStatus: response.status(), contentType: response.headers()['content-type'],
            resourceType: response.request().resourceType(), byteLength: bytes.length, fontMagicDetected: font,
            sha256: sha256(bytes), responseFile: file });
        } catch (fontError) { fontResponses.push({ captureContext, url, error: String(fontError) }); }
      })();
      fontPending.add(pending);
      void pending.finally(() => fontPending.delete(pending));
    });
  };
  let browser: Awaited<ReturnType<typeof chromium.launch>> | undefined;
  let server: Server | undefined;
  let context: BrowserContext | undefined;
  let importDebuggerCdp: ImportDebuggerSession | undefined;
  let api: APIRequestContext | undefined;
  let runFailed = false;
  let finalRegistryCheck: (() => Promise<RecordValue>) | undefined;
  let originalAuthenticationFile: { path: string; sha256: string } | undefined;
  const resources: RecordValue = { browser: { ownership: 'session', started: false },
    pluginHost: { ownership: 'none', startedByThisSession: false } };
  receipt['resources'] = resources;
  try {
    await checkpointReceipt('session-created');
    const previousReceiptPath = process.env['PENPOT_REAL_PREVIOUS_RECEIPT']
      ? artifactFile(process.env['PENPOT_REAL_PREVIOUS_RECEIPT']) : undefined;
    const previousReceiptBytes = previousReceiptPath ? await readFile(previousReceiptPath) : undefined;
    const previousReceipt = previousReceiptBytes ? record(JSON.parse(previousReceiptBytes.toString('utf8'))) : undefined;
    const refreshExistingWorkbench = process.argv.includes('--refresh-existing-workbench');
    assert.ok(!refreshExistingWorkbench || (previousReceipt && process.argv.includes('--full-workbench')),
      'Refreshing an existing workbench requires its actual acknowledged-save receipt and complete owning input');
    let workspaceUrl = process.env['PENPOT_REAL_WORKSPACE_URL'];
    if (previousReceipt) {
      assert.equal(previousReceipt['backend'], 'real');
      assert.equal(previousReceipt['saveAcknowledged'], true);
      assert.equal(previousReceipt['closedOriginalContext'], true);
      assert.equal(previousReceipt['metadataMatched'], true);
      assert.equal(typeof previousReceipt['workspaceUrl'], 'string');
      if (workspaceUrl) assert.equal(workspaceUrl, previousReceipt['workspaceUrl']);
      workspaceUrl = previousReceipt['workspaceUrl'] as string;
      receipt['continuationOf'] = { receiptPath: previousReceiptPath, receiptSha256: sha256(previousReceiptBytes!),
        originalContextClosureEvidence: 'closedOriginalContext in the immutable acknowledged-save receipt' };
      receipt['importPerformedInThisContext'] = false;
      receipt['saveRequestReplayed'] = false;
    }
    const execution = realSessionExecutionPlan(options, {
      workspaceUrl, authenticatedStorage: Boolean(process.env['PENPOT_REAL_STORAGE_STATE']),
      uiBuildReceipt: Boolean(process.env['PENPOT_REAL_UI_BUILD_RECEIPT']),
      workerBuildReceipt: Boolean(process.env['PENPOT_REAL_WORKER_BUILD_RECEIPT']),
    });
    receipt['executionMode'] = execution.mode;
    receipt['expectedWorkspace'] = options.expectedWorkspace;
    const origin = workspaceUrl ? workspaceIdentity(workspaceUrl).origin : new URL(required('PENPOT_BASE_URL')).origin;
    assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(new URL(origin).hostname), 'Use the authorized local Penpot runtime');
    const sourcePath = resolve(required('PENPOT_REAL_ZUI_SOURCE'));
    const source = await readFile(sourcePath, 'utf8');
    const original = parseZuiDocument(source).document;
    const sourceClosure = process.argv.includes('--source-closure');
    if (editSourceGap) {
      assert.ok(options.append && sourceClosure && execution.mode === 'reuse-existing',
        'A source gap edit requires a source-owned append with the existing registered plugin');
      assert.equal(options.expectedWorkspace?.fileId, 'b4a1e3ce-9e6a-81fe-8008-d69d60dbec5a');
      assert.equal(options.expectedWorkspace?.pageId, 'b4a1e3ce-9e6a-81fe-8008-d69d60dbec5b');
    }
    if (options.append) {
      assert.ok(workspaceUrl && options.existingPlugin && process.env['PENPOT_REAL_STORAGE_STATE'] &&
        process.env['PENPOT_REAL_UI_BUILD_RECEIPT'],
      'Append requires the assigned workspace, authenticated storage and current managed plugin build evidence');
      assert.ok(sourceClosure && !previousReceipt && !refreshExistingWorkbench &&
        !process.argv.includes('--full-workbench') &&
        !process.argv.includes('--edit-outliner-min-height') && !process.argv.includes('--edit-workbench-gap'),
      'Source append cannot refresh or edit a managed LevelEditor case');
      receipt['append'] = { ...options.append, savedForeignContentPreserved: false, reopenedForeignContentPreserved: false };
    }
    let actualCaseInput: { path: string; sha256: string } | undefined;
    let actualCase;
    if (process.argv.includes('--full-workbench') && !sourceClosure) {
      const casePath = artifactFile(required('PENPOT_REAL_CASE_JSON'));
      const expectedCaseSha = required('PENPOT_REAL_CASE_JSON_SHA256');
      const caseBytes = await readFile(casePath);
      assert.match(expectedCaseSha, /^[0-9a-f]{64}$/);
      assert.equal(sha256(caseBytes), expectedCaseSha, 'Current actual normal case changed');
      actualCase = validateLayoutReviewCase(JSON.parse(caseBytes.toString('utf8')));
      actualCaseInput = { path: casePath, sha256: expectedCaseSha };
      receipt['actualNormalCaseInput'] = actualCaseInput;
      receipt['actualNormalCaseId'] = actualCase.id;
      receipt['actualNormalViewport'] = actualCase.viewport;
      receipt['actualNormalDpi'] = actualCase.dpi;
    }
    let expectedOwnerSources: ReturnType<typeof parseFingerprintList> | undefined;
    if (process.env['PENPOT_REAL_SOURCE_FINGERPRINTS_JSON']) {
      assert.ok(sourceClosure, 'An explicit owner closure requires source-closure mode');
      const fingerprintPath = artifactFile(required('PENPOT_REAL_SOURCE_FINGERPRINTS_JSON'));
      const fingerprintBytes = await readFile(fingerprintPath);
      const fingerprintSha256 = required('PENPOT_REAL_SOURCE_FINGERPRINTS_SHA256');
      assert.match(fingerprintSha256, /^[0-9a-f]{64}$/);
      assert.equal(sha256(fingerprintBytes), fingerprintSha256, 'Admitted owner fingerprint input changed');
      expectedOwnerSources = parseFingerprintList(JSON.parse(fingerprintBytes.toString('utf8')), 'admitted owner sources');
      receipt['admittedOwnerSources'] = { path: fingerprintPath, sha256: fingerprintSha256, sources: expectedOwnerSources };
    } else {
      assert.ok(!process.env['PENPOT_REAL_SOURCE_FINGERPRINTS_SHA256'], 'Owner fingerprint hash has no input file');
    }
    const prepared = sourceClosure
      ? await prepareRealOwnerInput(resolve(required('ZUI_LAYOUT_REPO_ROOT')), sourcePath, source, original,
        { ...(process.env['PENPOT_REAL_THEME_SOURCE_PATH']
          ? { themeSourcePath: process.env['PENPOT_REAL_THEME_SOURCE_PATH']! } : {}),
          ...(expectedOwnerSources ? { expectedSources: expectedOwnerSources } : {}) })
      : process.argv.includes('--full-workbench')
        ? await prepareRealWorkbenchInput(resolve(required('ZUI_LAYOUT_REPO_ROOT')), sourcePath, source, original, true, actualCase)
        : undefined;
    const sourceGapEditTarget = editSourceGap
      ? prepareRealOwnerGapEdit(String(prepared!.input['penpot_original_source_path']), original) : undefined;
    if (sourceGapEditTarget) receipt['sourceGapEditTarget'] = sourceGapEditTarget;
    const importedDocument = prepared?.input ?? original;
    receipt['completeWorkbenchPrepared'] = Boolean(prepared?.managedPresentationPrepared);
    receipt['authoredSourceClosurePrepared'] = Boolean(prepared);
    receipt['actualManagedPresentationPrepared'] = Boolean(prepared?.managedPresentationPrepared);
    receipt['sourceOwners'] = prepared?.sourceFingerprints ?? [];
    if (prepared && 'themeSourcePath' in prepared)
      receipt['authoredThemeSourcePath'] = prepared.themeSourcePath;
    receipt['sourcePath'] = sourcePath;
    const sourceSha256 = sha256(source);
    receipt['sourceSha256'] = sourceSha256;
    if (previousReceipt) {
      assert.equal(previousReceipt['sourcePath'], sourcePath);
      assert.equal(previousReceipt['sourceSha256'], sourceSha256);
    }
    receipt['renderAcceptance'] = false;
    receipt['nativeEditorAcceptance'] = false;
    receipt['editorApplyVerified'] = false;
    if (prepared) {
      const preparedPath = resolve(output, 'prepared-input.zui');
      await writeFile(preparedPath, prepared.source);
      receipt['preparedInputPath'] = preparedPath;
      receipt['preparedInputSha256'] = sha256(prepared.source);
    }
    await checkpointReceipt('source-input-prepared');
    if (process.argv.includes('--prepare-only')) {
      assert.ok(prepared, 'Preparation requires the existing authored source closure or managed workbench host');
      receipt['pendingReason'] = 'Authored source closure prepared; actual browser persistence and managed Editor presentation remain unverified.';
      return;
    }
    const dist = requireRealSessionDirectory(required('ZUI_PLUGIN_DIST_ROOT'));
    const bundle = artifactFile(required('ZUI_PLUGIN_BUNDLE_PATH'));
    const executablePath = artifactFile(required('PLAYWRIGHT_CHROMIUM_EXECUTABLE'));
    const storagePath = process.env['PENPOT_REAL_STORAGE_STATE'] ? artifactFile(process.env['PENPOT_REAL_STORAGE_STATE']) : undefined;
    if (storagePath) originalAuthenticationFile = { path: storagePath, sha256: sha256(await readFile(storagePath)) };
    const fontResourcePath = process.env['PENPOT_REAL_FONT_RESOURCE_MANIFEST']
      ? artifactFile(process.env['PENPOT_REAL_FONT_RESOURCE_MANIFEST']) : undefined;
    const decode = transitDecoder(artifactFile(process.env['PENPOT_REAL_TRANSIT_PACKAGE'] ?? required('PENPOT_REAL_FRONTEND_PACKAGE')));
    const manifestBytes = await readFile(resolve(dist, 'manifest.json'));
    const manifest = JSON.parse(manifestBytes.toString('utf8'));
    const bundleBytes = await readFile(bundle);
    const distributionWorkerBytes = await readFile(resolve(dist, 'assets/plugin.js'));
    const bundleSha256 = sha256(bundleBytes);
    assert.equal(sha256(distributionWorkerBytes), bundleSha256,
      'The actual served distribution must contain the current independent plugin bundle');
    if (options.existingPlugin) {
      assert.equal(sha256(manifestBytes), options.existingPlugin.manifestSha256, 'Selected manifest differs from reuse pin');
      assert.equal(bundleSha256, options.existingPlugin.bundleSha256, 'Selected worker differs from reuse pin');
    }
    receipt['pluginBundleSha256'] = bundleSha256;
    receipt['pluginWorkerBuildEvidence'] = await verifyCurrentPluginWorkerBuild({
      buildReceiptPath: artifactFile(required('PENPOT_REAL_WORKER_BUILD_RECEIPT')),
      buildReceiptSha256: required('PENPOT_REAL_WORKER_BUILD_RECEIPT_SHA256'),
      repositoryRoot: resolve(required('ZUI_LAYOUT_REPO_ROOT')), bundleBytes,
    });
    const verifySelectedPluginHost = async (host: string,
      get: (url: string) => ReturnType<APIRequestContext['get']>): Promise<void> => {
      const hostUrl = new URL(host);
      const manifestCode = manifest['code'];
      assert.ok(typeof manifestCode === 'string' && manifestCode.length > 0, 'The selected manifest needs a worker code URL');
      assert.equal(new URL(manifestCode, hostUrl).href, new URL('assets/plugin.js', hostUrl).href,
        'The selected manifest must reference its own pinned worker');
      const manifestResponse = await get(new URL('manifest.json', hostUrl).href);
      assert.equal(manifestResponse.status(), 200, 'The selected plugin manifest must be reachable');
      assert.equal(manifestResponse.url(), new URL('manifest.json', hostUrl).href, 'Plugin manifest redirected');
      assert.equal(sha256(await manifestResponse.body()), sha256(manifestBytes), 'HTTP manifest differs from selected distribution');
      receipt['pluginWorkerHttpEvidence'] = await verifyCurrentPluginWorker({
        host, bundleBytes, distributionWorkerBytes, get,
      });
      const probe = await get(host);
      assert.equal(probe.status(), 200, 'The selected plugin host must serve its built index.html');
      assert.equal(probe.url(), host, 'Plugin index redirected');
      assert.equal(sha256(await probe.body()), sha256(await readFile(resolve(dist, 'index.html'))));
      receipt['pluginIndexHttpVerified'] = true;
      receipt['pluginIndexResponseHeaders'] = probe.headers();
      if (process.env['PENPOT_REAL_UI_BUILD_RECEIPT']) {
        receipt['pluginUiBuildEvidence'] = await verifyCurrentPluginUi({
          buildReceiptPath: artifactFile(required('PENPOT_REAL_UI_BUILD_RECEIPT')),
          repositoryRoot: resolve(required('ZUI_LAYOUT_REPO_ROOT')), dist,
          host, indexBytes: await probe.body(), get,
        });
      }
      receipt['pluginHost'] = host;
      await checkpointReceipt('plugin-host-verified');
    };

    const downloads = resolve(output, 'downloads');
    const temp = resolve(output, 'browser-temp');
    const browserUser = resolve(output, 'browser-user');
    const browserAppData = resolve(output, 'browser-appdata');
    const browserCache = resolve(output, 'browser-cache');
    for (const directory of [downloads, temp, browserUser, browserAppData, browserCache]) {
      await mkdir(directory, { recursive: true });
      requireRealSessionDirectory(directory);
    }
    // Playwright itself uses os.tmpdir() for its disposable browser profile.
    process.env['TMP'] = temp;
    process.env['TEMP'] = temp;
    process.env['TMPDIR'] = temp;
    receipt['browserOutputDirectories'] = { downloads, temp, browserUser, browserAppData, browserCache };
    // Prove the actual account and backend file before starting any browser process.
    api = await request.newContext(storagePath ? { storageState: storagePath } : {});
    const bootstrap = { request: api };
    let createdProfileId: string | undefined;
    if (!storagePath) {
      assert.ok(execution.bootstrapAllowed, 'Account authentication or creation is bootstrap-only');
      let credentialsPath = process.env['PENPOT_REAL_CREDENTIALS'];
      if (!credentialsPath) {
        const created = await createRealPenpotProfile(required('PENPOT_REAL_PREPL_URL'), resolve(output, 'credentials.json'));
        credentialsPath = created.credentialsPath;
        createdProfileId = created.profileId;
      }
      const credentials = JSON.parse(await readFile(artifactFile(credentialsPath), 'utf8'));
      assert.ok(typeof credentials.email === 'string' && typeof credentials.password === 'string');
      await rpc(bootstrap, origin, 'login-with-password', { email: credentials.email, password: credentials.password }, decode);
      // These are normal saved preferences on this temporary account, through the actual API.
      await rpc(bootstrap, origin, 'update-profile-props', { props: {
        'onboarding-viewed': true, 'onboarding-questions-answered': true,
        'release-notes-viewed': '2.17',
      } }, decode);
    }
    const profile = await rpc(bootstrap, origin, 'get-profile', {}, decode);
    assert.ok(profile.id && profile.id !== '00000000-0000-0000-0000-000000000000');
    if (createdProfileId) assert.equal(profile.id, createdProfileId);
    receipt['profileId'] = profile.id;
    if (previousReceipt) assert.equal(profile.id, previousReceipt['profileId']);
    let existingRegistration: (NonNullable<RealPenpotPersistenceOptions['existingPlugin']> & { registrySha256: string }) | undefined;
    if (options.existingPlugin) {
      const manifestUrl = new URL('manifest.json', options.existingPlugin.host).href;
      const verified = verifyExistingPluginRegistration(profile, manifestUrl, manifest['permissions'], options.existingPlugin);
      existingRegistration = { ...options.existingPlugin, registrySha256: verified.registrySha256 };
      receipt['pluginRegistryBefore'] = { sha256: verified.registrySha256, entryCount: verified.entryCount,
        pluginId: options.existingPlugin.pluginId, registryEntrySha256: verified.fingerprint };
      receipt['pluginRegistryUnchanged'] = false;
      finalRegistryCheck = async () => {
        const finalProfile = await rpc<RecordValue>(bootstrap, origin, 'get-profile', {}, decode);
        assert.equal(finalProfile.id, profile.id, 'Final registry account differs from preflight');
        return verifyPluginRegistryUnchanged(finalProfile, verified.registrySha256);
      };
    }
    if (!workspaceUrl) {
      assert.ok(execution.bootstrapAllowed, 'Workspace creation is bootstrap-only');
      const teams = await rpc<BackendTeam[]>(bootstrap, origin, 'get-teams', {}, decode);
      assert.ok(Array.isArray(teams));
      const defaultTeam = teams.find((team) => team['is-default']) ?? (teams.length === 1 ? teams[0] : undefined);
      assert.ok(defaultTeam?.id, 'The actual created profile needs its own default team');
      const project = await rpc(bootstrap, origin, 'create-project', { 'team-id': defaultTeam.id,
        name: `Zircon real persistence ${randomUUID()}` }, decode);
      const createdFile = await rpc(bootstrap, origin, 'create-file', { 'project-id': project.id,
        name: `ZUI ${basename(sourcePath)}` }, decode);
      const actualFile = await rpc<BackendFile>(bootstrap, origin, 'get-file', { id: createdFile.id }, decode);
      const pageId = actualFile.data?.pages?.[0];
      assert.ok(pageId, 'Actual create-file did not persist a page');
      workspaceUrl = `${origin}/#/workspace?${new URLSearchParams({ 'team-id': defaultTeam.id,
        'file-id': actualFile.id, 'page-id': pageId })}`;
    }
    const identity = workspaceIdentity(workspaceUrl);
    if (options.expectedWorkspace) assert.deepEqual(identity, options.expectedWorkspace, 'Created or restored workspace differs');
    if (previousReceipt) assert.deepEqual(identity, previousReceipt['identity']);
    receipt['identity'] = identity;
    receipt['workspaceUrl'] = workspaceUrl;
    const team = await rpc(bootstrap, identity.origin, 'get-team', { id: identity.teamId }, decode);
    assert.equal(team.id, identity.teamId);
    const initial = await rpc<BackendFile>(bootstrap, identity.origin, 'get-file', { id: identity.fileId }, decode);
    assert.equal(initial.id, identity.fileId);
    assert.ok(Number.isSafeInteger(initial.revn) && Number.isSafeInteger(initial.vern));
    const objects = initial.data?.['pages-index']?.[identity.pageId]?.objects;
    assert.ok(objects);
    if (options.append) {
      verifySourceOwnerAppendAdmission(initial, identity, options.append, {
        sourceClosure, existingPlugin: Boolean(options.existingPlugin), authenticatedStorage: Boolean(storagePath),
        previousReceipt: Boolean(previousReceipt), fullWorkbench: process.argv.includes('--full-workbench'),
        editRequested: process.argv.includes('--edit-outliner-min-height') || process.argv.includes('--edit-workbench-gap'),
      });
      receipt['appendInitialObjectCount'] = Object.keys(objects).length;
    } else if (!previousReceipt) assert.ok(Object.keys(objects).length <= 1,
      'Use a fresh actual file page to make restored Select All unambiguous');
    receipt['initialRevision'] = { revn: initial.revn, vern: initial.vern };
    receipt['apiBootstrapVerified'] = true;
    // Authentication remains in memory across isolated contexts; never duplicate cookies to evidence.
    const initialAuthentication = await api.storageState();
    receipt['authenticationStorage'] = { persistedBySession: false,
      ...(originalAuthenticationFile ? { originalPath: originalAuthenticationFile.path,
        originalSha256: originalAuthenticationFile.sha256 } : {}) };
    await checkpointReceipt('api-bootstrap-verified');
    let selectedPlugin: { host: string } | undefined;
    if (options.existingPlugin) {
      selectedPlugin = { host: new URL(options.existingPlugin.host).href };
      await verifySelectedPluginHost(selectedPlugin.host, url => api!.get(url));
      receipt['existingPlugin'] = { ...options.existingPlugin, startedByThisSession: false,
        stoppedByThisSession: false, nativeBirthVerifiedByThisRunner: false };
      resources['pluginHost'] = { ownership: 'external', startedByThisSession: false,
        stoppedByThisSession: false };
    }
    if (process.argv.includes('--api-only')) {
      receipt['pendingReason'] = 'Actual backend account and file verified; browser import, save, reopen and export remain unverified.';
      return;
    }
    if (!selectedPlugin) {
      assert.ok(execution.bootstrapAllowed, 'Only explicit managed bootstrap can create a plugin host');
      const started = await pluginServer(dist, bundle);
      server = started.server;
      selectedPlugin = { host: started.host };
      resources['pluginHost'] = { ownership: 'session', startedByThisSession: true,
        pid: process.pid, host: started.host, stoppedByThisSession: false };
      await verifySelectedPluginHost(started.host, url => api!.get(url));
    }
    const plugin = selectedPlugin;
    assert.ok(plugin);
    browser = await chromium.launch({ executablePath, headless: true, downloadsPath: downloads,
      env: { ...process.env, TMP: temp, TEMP: temp, TMPDIR: temp, USERPROFILE: browserUser,
        APPDATA: browserAppData, LOCALAPPDATA: browserCache }, args: ['--disable-dev-shm-usage'] });
    resources['browser'] = { ownership: 'session', started: true, nativeTerminationReceiptRequired: true };
    const contextOptions = { acceptDownloads: true, serviceWorkers: 'block' as const,
      viewport: { width: 1920, height: 1200 }, locale: 'en-US' };
    context = await browser.newContext({ ...contextOptions, storageState: initialAuthentication });
    watchFonts(context, 'initial-context');
    const saves: SaveEvidence[] = [];
    let saveError: unknown;
    const startSaveObservation = async (page: Page): Promise<void> => {
      receipt['saveResponses'] = saves;
      const onSave = async (response: ActualSaveHttp): Promise<void> => {
        receipt['lastObservedSaveHttp'] = { httpStatus: response.httpStatus, url: response.url,
          requestId: response.requestId, contentType: response.contentType,
          requestFile: response.requestFile, responseFile: response.responseFile,
          requestSha256: sha256(response.requestBody), responseSha256: sha256(response.responseBody),
          observedAtUtc: new Date().toISOString() };
        await checkpointReceipt('save-http-response-observed');
        const url = new URL(response.url);
        const { requestBody, requestFile, responseFile } = response;
        const saveRequest = decode(requestBody.toString('utf8')) as SaveRequest;
        assert.equal(saveRequest.id, identity.fileId, 'The observed save must identify the actual file');
        assert.equal(url.searchParams.get('id'), saveRequest.id, 'The actual save URL and request must identify the same file');
        const body = response.responseBody.toString('utf8');
        assert.ok(response.httpStatus >= 200 && response.httpStatus < 300,
          `Actual update-file returned HTTP ${response.httpStatus}`);
        assert.ok(response.contentType.includes('transit+json'), 'The real save ACK must use actual Transit content');
        const revision = verifySaveResponse(decode(body), identity, initial.revn, saveRequest);
        saves.push({ httpStatus: response.httpStatus, url: response.url, requestId: response.requestId, responseSha256: sha256(response.responseBody),
          requestSha256: sha256(requestBody), requestFile, responseFile, revn: revision, observedAt: Date.now() });
        receipt['saveAcknowledged'] = true;
        await checkpointReceipt('save-acknowledged');
      };
      receipt['saveCaptureConfiguration'] = await observeRealSaveHttp(page, {
        fileId: identity.fileId, output, pending: savePending, diagnostics: saveHttpResponses,
        onResponse: onSave, onError: (error) => { saveError = error; },
      });
      await checkpointReceipt('save-observer-ready');
    };
    if (previousReceipt && previousReceiptPath) {
      const originalRevision = record(previousReceipt['initialRevision']);
      assert.ok(Number.isSafeInteger(originalRevision['revn']) && Number.isSafeInteger(originalRevision['vern']));
      const priorSaves = previousReceipt['saveResponses'];
      assert.ok(Array.isArray(priorSaves) && priorSaves.length);
      const acknowledgedRevisions: number[] = [];
      const acknowledgedEvidence: RecordValue[] = [];
      for (const entry of priorSaves) {
        const save = record(entry);
        const requestName = save['requestFile'];
        const responseName = save['responseFile'];
        assert.ok(typeof requestName === 'string' && basename(requestName) === requestName);
        assert.ok(typeof responseName === 'string' && basename(responseName) === responseName);
        const requestPath = artifactFile(resolve(dirname(previousReceiptPath), requestName));
        const responsePath = artifactFile(resolve(dirname(previousReceiptPath), responseName));
        const requestBytes = await readFile(requestPath);
        const responseBytes = await readFile(responsePath);
        assert.equal(sha256(requestBytes), save['requestSha256']);
        assert.equal(sha256(responseBytes), save['responseSha256']);
        assert.ok(typeof save['httpStatus'] === 'number' && save['httpStatus'] >= 200 && save['httpStatus'] < 300);
        assert.equal(typeof save['url'], 'string');
        const acknowledgedUrl = new URL(save['url'] as string);
        assert.equal(acknowledgedUrl.origin, identity.origin);
        assert.equal(acknowledgedUrl.pathname, '/api/main/methods/update-file');
        assert.equal(acknowledgedUrl.searchParams.get('id'), identity.fileId);
        const savedRequest = decode(requestBytes.toString('utf8')) as SaveRequest;
        const acknowledgedRevision = verifySaveResponse(decode(responseBytes.toString('utf8')), identity,
          originalRevision['revn'] as number, savedRequest);
        assert.equal(acknowledgedRevision, save['revn']);
        acknowledgedRevisions.push(acknowledgedRevision);
        acknowledgedEvidence.push({ requestPath, responsePath, requestSha256: sha256(requestBytes),
          responseSha256: sha256(responseBytes), acknowledgedRevision });
      }
      receipt['acknowledgedSaveEvidenceRevalidated'] = acknowledgedEvidence;
      if (!refreshExistingWorkbench) {
        receipt['saveAcknowledged'] = true;
        receipt['closedOriginalContext'] = true;
        receipt['newAuthenticatedContext'] = true;
      }
      const savedMetadataPath = artifactFile(resolve(dirname(previousReceiptPath), 'saved-zui-metadata.json'));
      const savedMetadataBytes = await readFile(savedMetadataPath);
      const archivedMetadata = JSON.parse(savedMetadataBytes.toString('utf8')) as ReturnType<typeof persistedZuiMetadata>;
      assert.equal(archivedMetadata.length, 1);
      receipt['priorSavedMetadataEvidence'] = { path: savedMetadataPath, sha256: sha256(savedMetadataBytes) };
      let archivedGroups: ReturnType<typeof persistedViewportToolbarGroupMetadata> = [];
      if (viewportToolbarContract(reviewHost(importedDocument) ?? importedDocument)) {
        const groupPath = artifactFile(resolve(dirname(previousReceiptPath), 'saved-toolbar-group-metadata.json'));
        const groupBytes = await readFile(groupPath);
        archivedGroups = JSON.parse(groupBytes.toString('utf8'));
        assert.ok(Array.isArray(archivedGroups) && archivedGroups.length > 0,
          'Continuation requires the actual previously saved original toolbar group evidence');
        receipt['priorSavedViewportToolbarGroupEvidence'] = { path: groupPath, sha256: sha256(groupBytes) };
      }
      await checkpointReceipt('prior-save-evidence-revalidated');
      if (refreshExistingWorkbench) {
        const currentMetadata = persistedZuiMetadata(initial, identity.pageId);
        const removals = verifyReviewScrollCleanupAdmission(archivedMetadata, currentMetadata);
        const currentGroups = persistedToolbarGroups(initial, identity.pageId, currentMetadata[0], importedDocument);
        const groupRemovals = archivedGroups.length
          ? verifyReviewScrollCleanupAdmission(archivedGroups, currentGroups) : [];
        receipt['reviewScrollCleanupAdmission'] = { removals, groupRemovals,
          metadataBefore: currentMetadata, formalAcceptance: false,
          action: 'source-owned layout refresh; exact metadata or known empty markers; fresh actual save ACK required' };
        await checkpointReceipt('existing-scroll-cleanup-admitted');
      }
      let reopened = await openWorkspace(context, workspaceUrl, plugin.host, manifest, bundleSha256, pageErrors, decode,
        async (page) => {
          if (refreshExistingWorkbench) await startSaveObservation(page);
          await selectActualAssetBoard(page, archivedMetadata[0].boardId, receipt);
        }, existingRegistration, execution.mode);
      let continuedMetadata = archivedMetadata;
      let continuedGroups = archivedGroups;
      let continuedFile = initial;
      let continuedAcknowledgedRevision = Math.max(...acknowledgedRevisions);
      if (refreshExistingWorkbench) {
        const refreshOutput = resolve(output, 'existing-workbench-refresh');
        await mkdir(refreshOutput);
        const refreshReceipt: RecordValue = {};
        receipt['existingWorkbenchRefresh'] = refreshReceipt;
        const refreshButton = reopened.frame.getByRole('button', { name: 'Export selected' });
        await refreshButton.waitFor({ timeout: 30_000 });
        const refreshDownload = await withSourceColorRevisionFence(context, identity, archivedMetadata[0].boardId,
          importedDocument, decode, refreshReceipt, () => clickAndWaitForDownload(
          () => reopened.page.waitForEvent('download', { timeout: timeoutMs }),
          () => observeActualExportClick(reopened.page, refreshButton, refreshOutput, refreshReceipt)));
        const refreshExportPath = resolve(refreshOutput, 'exported.zui');
        await refreshDownload.saveAs(refreshExportPath);
        assert.equal(await refreshDownload.failure(), null);
        const refreshedSource = await readFile(refreshExportPath, 'utf8');
        assert.deepEqual(normalizeZuiDocument(parseZuiDocument(refreshedSource).document), normalizeZuiDocument(original),
          'The real cleanup export must preserve the complete original ZUI contract');
        refreshReceipt['exportPath'] = refreshExportPath;
        refreshReceipt['exportSha256'] = sha256(refreshedSource);
        refreshReceipt['exportSemanticMatch'] = true;
        await reopened.page.locator('plugin-modal').locator('button').first().click();
        await reopened.page.locator('plugin-modal').waitFor({ state: 'detached' });
        refreshReceipt['pluginClosedViaUi'] = true;
        let refreshedFile: BackendFile | undefined;
        const refreshDeadline = Date.now() + timeoutMs;
        while (Date.now() < refreshDeadline) {
          if (saveError) throw saveError;
          if (!saves.length || Date.now() - saves[saves.length - 1].observedAt < 1_500) {
            await reopened.page.waitForTimeout(500);
            continue;
          }
          const file = await rpc<BackendFile>(context, identity.origin, 'get-file', { id: identity.fileId }, decode);
          const metadata = persistedZuiMetadata(file, identity.pageId);
          const acknowledgedRevision = Math.max(...saves.map(save => save.revn));
          if (file.revn >= acknowledgedRevision && Date.now() - saves[saves.length - 1].observedAt >= 1_500) {
            verifyReopenedFile(initial, file, identity, acknowledgedRevision, archivedMetadata, metadata);
            const groups = persistedToolbarGroups(file, identity.pageId, metadata[0], importedDocument);
            assert.deepEqual(groups, archivedGroups, 'Actual cleanup changed original toolbar group metadata');
            continuedMetadata = metadata;
            continuedGroups = groups;
            continuedFile = file;
            continuedAcknowledgedRevision = acknowledgedRevision;
            refreshedFile = file;
            break;
          }
          await reopened.page.waitForTimeout(500);
        }
        assert.ok(refreshedFile, 'The cleanup requires a fresh actual UI save ACK and exact persisted metadata');
        await Promise.all(savePending);
        if (saveError) throw saveError;
        await writeFile(resolve(output, 'saved-zui-metadata.json'), `${JSON.stringify(continuedMetadata, null, 2)}\n`);
        await writeFile(resolve(output, 'saved-toolbar-group-metadata.json'), `${JSON.stringify(continuedGroups, null, 2)}\n`);
        receipt['savedRevision'] = { revn: refreshedFile.revn, vern: refreshedFile.vern };
        await checkpointReceipt('existing-scroll-cleanup-saved');
        const authentication = await context.storageState();
        await context.close();
        receipt['closedOriginalContext'] = true;
        await checkpointReceipt('original-context-closed');
        context = await browser.newContext({ ...contextOptions, storageState: authentication });
        watchFonts(context, 'fresh-reopened-context');
        const reopenedProfile = await rpc(context, identity.origin, 'get-profile', {}, decode);
        assert.equal(reopenedProfile.id, profile.id);
        receipt['newAuthenticatedContext'] = true;
        await checkpointReceipt('fresh-context-authenticated');
        reopened = await openWorkspace(context, workspaceUrl, plugin.host, manifest, bundleSha256, pageErrors, decode,
          page => selectActualAssetBoard(page, continuedMetadata[0].boardId, receipt), existingRegistration, execution.mode);
      }
      const reopenedFile = await rpc<BackendFile>(context, identity.origin, 'get-file', { id: identity.fileId }, decode);
      const restoredMetadata = persistedZuiMetadata(reopenedFile, identity.pageId);
      verifyReopenedFile({ id: identity.fileId, vern: originalRevision['vern'] as number }, reopenedFile, identity,
        continuedAcknowledgedRevision, continuedMetadata, restoredMetadata);
      receipt['nativeCopyFreshReopen'] = assertNativeInstanceGeometryPreserved(continuedFile, reopenedFile);
      assert.equal(restoredMetadata.length, 1);
      const comparison = compareStoredZuiMetadata(decodeZuiMetadata(restoredMetadata[0].metadata.document), importedDocument);
      await writeFile(resolve(output, 'reopened-metadata-comparison.json'), `${JSON.stringify(comparison, null, 2)}\n`);
      assert.ok(comparison.wireMatched, 'The continued file differs from the complete owning metadata codec');
      const restoredGroups = persistedToolbarGroups(reopenedFile, identity.pageId, restoredMetadata[0], importedDocument);
      assert.deepEqual(restoredGroups, continuedGroups, 'Actual continued original group metadata differs from the acknowledged saved file');
      await writeFile(resolve(output, 'reopened-toolbar-group-metadata.json'), `${JSON.stringify(restoredGroups, null, 2)}\n`);
      receipt['viewportToolbarGroupsMatched'] = true;
      receipt['reopenedViewportToolbarGroupCount'] = restoredGroups.length;
      receipt['reopenedSameFile'] = true;
      receipt['metadataMatched'] = true;
      receipt['importedBoardId'] = restoredMetadata[0].boardId;
      receipt['reopenedRevision'] = { revn: reopenedFile.revn, vern: reopenedFile.vern };
      await writeFile(resolve(output, 'reopened-zui-metadata.json'), `${JSON.stringify(restoredMetadata, null, 2)}\n`);
      await checkpointReceipt('continued-reopened-metadata-verified');
      const exportButton = reopened.frame.getByRole('button', { name: 'Export selected' });
      await exportButton.waitFor({ timeout: 30_000 });
      receipt['actualExportUiBeforeClick'] = await captureExportUi(reopened.frame, reopened.page);
      await checkpointReceipt('actual-export-ui-ready');
      await capturePreviewLayoutAudit(reopened.frame, restoredMetadata[0].boardId, output, receipt, importedDocument);
      await checkpointReceipt('post-reopen-layout-audit-written');
      const download = await withSourceColorRevisionFence(context, identity, restoredMetadata[0].boardId,
        importedDocument, decode, receipt, () => clickAndWaitForDownload(
        () => reopened.page.waitForEvent('download', { timeout: timeoutMs }), () => observeActualExportClick(reopened.page, exportButton, output, receipt)));
      const exportPath = resolve(output, 'exported.zui');
      receipt['exportDownloadObserved'] = true;
      await checkpointReceipt('export-download-observed');
      await download.saveAs(exportPath);
      assert.equal(await download.failure(), null);
      receipt['exportPath'] = exportPath;
      receipt['exportSha256'] = sha256(await readFile(exportPath));
      receipt['exportSuggestedFileName'] = basename(download.suggestedFilename());
      await checkpointReceipt('export-download-saved');
      const exported = await readFile(exportPath, 'utf8');
      assert.deepEqual(normalizeZuiDocument(parseZuiDocument(exported).document), normalizeZuiDocument(original),
        'The existing selected-asset export must preserve the complete original ZUI contract');
      receipt['exportSemanticMatch'] = true;
      assert.equal(sha256(await readFile(sourcePath)), sourceSha256);
      for (const fingerprint of prepared?.sourceFingerprints ?? []) {
        assert.equal(sha256(await readFile(resolve(required('ZUI_LAYOUT_REPO_ROOT'), fingerprint.sourcePath))), fingerprint.sha256);
      }
      receipt['sourceFingerprintMatched'] = true;
      receipt['sourceOwnerFingerprintsMatched'] = Boolean(prepared);
      receipt['exportPath'] = exportPath;
      receipt['exportSha256'] = sha256(exported);
      receipt['exportSuggestedFileName'] = basename(download.suggestedFilename());
      await downloadActualCanvasPng(
        reopened.page,
        reopened.frame,
        restoredMetadata[0].boardId,
        output,
        receipt,
      );
      await checkpointReceipt('actual-canvas-png-export-saved');
      await checkpointReceipt('persistence-gates-verified');
      receipt['status'] = 'passed';
      receipt['persistenceClaim'] = persistenceClaim(receipt);
      assert.equal(receipt['persistenceClaim'], true);
      return;
    }
    const session = await openWorkspace(context, workspaceUrl, plugin.host, manifest, bundleSha256, pageErrors, decode,
      undefined, existingRegistration, execution.mode);
    receipt['pluginInstallationViaUi'] = session.installation;
    await checkpointReceipt('plugin-opened');
    receipt['pluginFrameStylesBeforeImport'] = await captureFrameStyles(session.frame);
    const styleLinks = await session.frame.locator('link[rel="stylesheet"]').evaluateAll((elements) =>
      elements.map((element) => (element as HTMLLinkElement).href));
    const stylesheetResponses: RecordValue[] = [];
    for (const href of styleLinks) {
      const response = await context.request.get(href);
      const bytes = await response.body();
      const responseFile = `plugin-stylesheet-${stylesheetResponses.length + 1}.css`;
      await writeFile(resolve(output, responseFile), bytes);
      stylesheetResponses.push({ url: href, httpStatus: response.status(), headers: response.headers(),
        sha256: sha256(bytes), byteLength: bytes.length, responseFile });
    }
    receipt['pluginStylesheetHttpResponses'] = stylesheetResponses;
    await writeFile(resolve(output, 'plugin-style-proof.json'), `${JSON.stringify({
      frame: receipt['pluginFrameStylesBeforeImport'], http: stylesheetResponses,
    }, null, 2)}\n`);
    const cdp = await context.newCDPSession(session.page);
    const browserVersion = await cdp.send('Browser.getVersion');
    await cdp.detach();
    receipt['browserRuntime'] = { ...browserVersion, executablePath,
      executableSha256: sha256(await readFile(executablePath)) };
    await startSaveObservation(session.page);
    if (fontResourcePath) {
      await session.frame.locator('input[data-font-resource-manifest]').setInputFiles(fontResourcePath);
      const fontStatus = session.frame.locator('.status-panel');
      await fontStatus.filter({ hasText: `字体资源清单 · ${basename(fontResourcePath)}` }).waitFor({ timeout: 30_000 });
      assert.equal(await fontStatus.getAttribute('data-level'), 'idle', await fontStatus.innerText());
      await session.frame.locator('button[data-appearance="primary"]:not([disabled])').waitFor({ timeout: 30_000 });
      receipt['fontResourceManifestPath'] = fontResourcePath;
      receipt['fontResourceManifestSha256'] = sha256(await readFile(fontResourcePath));
      receipt['fontResourceManifestLoadedViaUi'] = true;
    }
    const importDebuggerCaptures: RecordValue[] = [];
    const importDebuggerScriptURLs = new Map<string, string>();
    const importDebuggerDiagnostics: RecordValue = { setupStatus: 'pending', noProgressThresholdMs: 20_000,
      pauseCaptureWindowMs: 6_000, diagnosticDeadlineMs: 10_000, captures: importDebuggerCaptures };
    receipt['importDebuggerDiagnostics'] = importDebuggerDiagnostics;
    try {
      const attachPromise = context!.newCDPSession(session.page);
      const attached = await settleBeforeNodeDeadline(attachPromise, Date.now() + 2_000);
      if (attached.status === 'completed') {
        importDebuggerCdp = attached.value;
        importDebuggerCdp.on('Debugger.scriptParsed', (event: unknown) => {
          const parsed = event && typeof event === 'object' && !Array.isArray(event) ? event as RecordValue : {};
          const scriptId = parsed['scriptId'];
          const scriptURL = safeDebuggerScriptURL(parsed['url']);
          if ((typeof scriptId === 'string' || typeof scriptId === 'number') && scriptURL)
            importDebuggerScriptURLs.set(String(scriptId), scriptURL);
        });
        const enabled = await settleBeforeNodeDeadline(
          Promise.resolve().then(() => importDebuggerCdp!.send('Debugger.enable')), Date.now() + 2_000);
        if (enabled.status === 'completed') importDebuggerDiagnostics['setupStatus'] = 'enabled';
        else {
          importDebuggerDiagnostics['setupStatus'] = `debugger-enable-${enabled.status}`;
          const cleanupStartedAt = Date.now();
          importDebuggerDiagnostics['setupCleanup'] = await releaseImportDebuggerSession(importDebuggerCdp,
            cleanupStartedAt + 1_000, cleanupStartedAt + 2_000);
          importDebuggerCdp = undefined;
        }
      } else {
        importDebuggerDiagnostics['setupStatus'] = `cdp-attach-${attached.status}`;
        if (attached.status === 'deadline') {
          void attachPromise.then((lateSession) => {
            const cleanupStartedAt = Date.now();
            return releaseImportDebuggerSession(lateSession, cleanupStartedAt + 1_000, cleanupStartedAt + 2_000);
          }, () => undefined).catch(() => undefined);
        }
      }
    } catch {
      importDebuggerDiagnostics['setupStatus'] = 'debugger-setup-failed';
      if (importDebuggerCdp) {
        const cleanupStartedAt = Date.now();
        importDebuggerDiagnostics['setupCleanup'] = await releaseImportDebuggerSession(importDebuggerCdp,
          cleanupStartedAt + 1_000, cleanupStartedAt + 2_000);
        importDebuggerCdp = undefined;
      }
    }
    if (options.append) {
      const replaceSelection = session.frame.locator('input[data-import-replace-selection]');
      assert.equal(await replaceSelection.count(), 1, 'Append requires the current public import policy control');
      await replaceSelection.uncheck();
      assert.equal(await replaceSelection.isChecked(), false, 'Append must not replace any selected business canvas');
      receipt['importPolicy'] = { replaceSelection: false, createComponentMaster: false,
        nativeComponentMode: 'full', selectedVia: 'existing official Import .zui public checkbox' };
    }
    await session.frame.locator('input[type="file"][accept*=".zui"]').first().setInputFiles(prepared ? {
      name: basename(sourcePath), mimeType: 'text/plain', buffer: Buffer.from(prepared.source, 'utf8'),
    } : sourcePath);
    const importStatus = session.frame.locator('.status-panel');
    const importProgress: RecordValue[] = [];
    receipt['importProgress'] = importProgress;
    const importDeadline = Date.now() + timeoutMs;
    await checkpointReceipt('actual-import-ui-wait');
    let previousImportStatus = '';
    let lastImportProgressAt = Date.now();
    let debuggerCaptureAttempted = false;
    let terminalImportStatus: { level: string | null; previewBoardId: string | null; text: string } | undefined;
    const readImportStatus = () => importStatus.evaluate((element) => ({
      level: element.getAttribute('data-level'),
      previewBoardId: element.getAttribute('data-preview-board-id'),
      text: element.textContent ?? '',
    }), undefined, { timeout: Math.max(1, importDeadline - Date.now()) });
    const captureImportStallOnce = async (): Promise<void> => {
      if (debuggerCaptureAttempted || terminalImportStatus || Date.now() - lastImportProgressAt < 20_000) return;
      debuggerCaptureAttempted = true;
      const captureStartedAt = Date.now();
      const trigger = { triggeredAtUtc: new Date(captureStartedAt).toISOString(),
        noUiProgressMs: captureStartedAt - lastImportProgressAt };
      if (importDebuggerCdp) {
        const capture = await captureImportDebuggerStack(importDebuggerCdp, importDebuggerScriptURLs, importDeadline);
        importDebuggerCdp = undefined;
        importDebuggerCaptures.push({ ...trigger, ...capture });
      } else {
        importDebuggerCaptures.push({ ...trigger, status: 'debugger-not-enabled',
          callFramesAvailable: false, callFrameCount: 0, callFramesTruncated: false, callFrames: [] });
      }
      void checkpointReceipt('import-debugger-stack-captured').catch(() => undefined);
    };
    type ImportObservationResult = { kind: 'observed'; observed: Awaited<ReturnType<typeof readImportStatus>> } |
      { kind: 'error'; error: unknown };
    let importObservation = readImportStatus();
    while (Date.now() < importDeadline) {
      const wakeDelayMs = Math.max(1, Math.min(importDeadline - Date.now(), debuggerCaptureAttempted ? 500 :
        lastImportProgressAt + 20_000 - Date.now()));
      let wakeTimer: ReturnType<typeof setTimeout> | undefined;
      const wake = new Promise<{ kind: 'timer' }>((resolve) => {
        wakeTimer = setTimeout(() => resolve({ kind: 'timer' }), wakeDelayMs);
      });
      let settledObservation: ImportObservationResult | undefined;
      const observation = importObservation.then(
        (observed) => { settledObservation = { kind: 'observed', observed }; return settledObservation; },
        (error: unknown) => { settledObservation = { kind: 'error', error }; return settledObservation; });
      let result: ImportObservationResult | { kind: 'timer' } = await Promise.race([observation, wake])
        .finally(() => { if (wakeTimer) clearTimeout(wakeTimer); });
      if (result.kind === 'timer') {
        // Give a status already delivered by the page one microtask to win over
        // the stall timer, so a terminal result always takes priority.
        await Promise.resolve();
        if (settledObservation) result = settledObservation;
        else {
          if (Date.now() >= importDeadline)
            throw new Error('Actual import UI observation exceeded the import deadline');
          await captureImportStallOnce();
          continue;
        }
      }
      if (result.kind === 'error') {
        if (Date.now() >= importDeadline)
          throw new Error('Actual import UI observation exceeded the import deadline');
        throw result.error;
      }
      const observed = result.observed;
      const signature = JSON.stringify(observed);
      if (signature !== previousImportStatus) {
        previousImportStatus = signature;
        lastImportProgressAt = Date.now();
        importProgress.push({ ...observed, observedAtUtc: new Date().toISOString() });
        await checkpointReceipt('actual-import-ui-progress');
      }
      // A completed warning must be reported immediately, then fail the unchanged
      // success gate below. A font-resource warning without a board is not completion.
      if (observed.level === 'success' || observed.level === 'error' ||
          (observed.level === 'warning' && observed.previewBoardId)) {
        terminalImportStatus = observed;
        break;
      }
      await captureImportStallOnce();
      await new Promise<void>((resolve) => setTimeout(resolve, 500));
      if (Date.now() < importDeadline) importObservation = readImportStatus();
    }
    assert.ok(terminalImportStatus, 'Actual import did not complete; see recorded importProgress');
    assert.equal(terminalImportStatus.level, 'success', terminalImportStatus.text);
    const importedBoardId = terminalImportStatus.previewBoardId;
    assert.ok(importedBoardId, 'The completed import must identify its actual Penpot asset board');
    receipt['importedBoardId'] = importedBoardId;
    receipt['importStatus'] = terminalImportStatus.text;
    await checkpointReceipt('actual-import-completed');
    receipt['pluginFrameStylesAfterImport'] = await captureFrameStyles(session.frame);
    // Closing the actual plugin UI completes the editing interaction and stops
    // its authoring observers before Penpot's normal autosave quiet period.
    await session.page.locator('plugin-modal').locator('button').first().click();
    await session.page.locator('plugin-modal').waitFor({ state: 'detached' });
    receipt['pluginClosedViaUiAfterImport'] = true;
    await checkpointReceipt('import-plugin-closed-via-ui');
    const importedScreenshot = resolve(output, 'penpot-imported.png');
    await session.page.screenshot({ path: importedScreenshot });
    receipt['importedScreenshotPath'] = importedScreenshot;
    let saved: BackendFile | undefined;
    let metadata: ReturnType<typeof persistedZuiMetadata> = [];
    let toolbarGroups: ReturnType<typeof persistedViewportToolbarGroupMetadata> = [];
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      if (saveError) throw saveError;
      // Read persisted state only after the actual save response has been
      // verified. A GET cannot substitute for that ACK or race its large body.
      if (!saves.length || Date.now() - saves[saves.length - 1].observedAt < 1_500) {
        await session.page.waitForTimeout(500);
        continue;
      }
      const file = await rpc<BackendFile>(context, identity.origin, 'get-file', { id: identity.fileId }, decode);
      const stored = persistedZuiMetadata(file, identity.pageId);
      if (saves.length && stored.length === 1 && file.revn >= Math.max(...saves.map((save) => save.revn)) &&
        Date.now() - saves[saves.length - 1].observedAt >= 1_500) {
        await writeFile(resolve(output, 'saved-zui-metadata.json'), `${JSON.stringify(stored, null, 2)}\n`);
        const comparison = compareStoredZuiMetadata(decodeZuiMetadata(stored[0].metadata.document), importedDocument);
        await writeFile(resolve(output, 'saved-metadata-comparison.json'), `${JSON.stringify(comparison, null, 2)}\n`);
        receipt['savedMetadataWireMatch'] = comparison.wireMatched;
        receipt['savedMetadataRawDifferenceCount'] = comparison.rawDifferences.length;
        receipt['savedMetadataRawDifferencesOnlyUndefinedObjectOmissions'] = comparison.rawDifferencesOnlyUndefinedObjectOmissions;
        assert.ok(comparison.wireMatched,
          'Backend metadata differs from the complete owning typed metadata codec; see saved-metadata-comparison.json');
        toolbarGroups = persistedToolbarGroups(file, identity.pageId, stored[0], importedDocument);
        await writeFile(resolve(output, 'saved-toolbar-group-metadata.json'), `${JSON.stringify(toolbarGroups, null, 2)}\n`);
        receipt['savedViewportToolbarGroupCount'] = toolbarGroups.length;
        saved = file;
        metadata = stored;
        receipt['savedRevision'] = { revn: file.revn, vern: file.vern };
        await checkpointReceipt('saved-backend-metadata-verified');
        break;
      }
      await session.page.waitForTimeout(500);
    }
    assert.ok(saved, 'No actual saved backend revision and matching ZUI metadata were observed');
    if (options.append) {
      assert.equal(metadata[0].boardId, importedBoardId, 'Append saved a different source-owned asset');
      assertExistingPenpotContentPreserved(initial, saved);
      if (sourceGapEditTarget) assert.deepEqual(record(saved.data)['components'] ?? {}, record(initial.data)['components'] ?? {},
        'The admitted AI source must not create or change any component master');
      record(receipt['append'])['savedForeignContentPreserved'] = true;
    }
    await Promise.all(savePending);
    if (saveError) throw saveError;
    receipt['saveResponses'] = saves;
    receipt['saveAcknowledged'] = true;
    await checkpointReceipt('save-observation-settled');
    const savedScreenshot = resolve(output, 'penpot-saved.png');
    await session.page.screenshot({ path: savedScreenshot });
    receipt['savedScreenshotPath'] = savedScreenshot;
    receipt['savedRevision'] = { revn: saved.revn, vern: saved.vern };
    await writeFile(resolve(output, 'saved-zui-metadata.json'), `${JSON.stringify(metadata, null, 2)}\n`);
    await checkpointReceipt('saved-metadata-written');
    const authentication = await context.storageState();
    await context.close();
    receipt['closedOriginalContext'] = true;
    await checkpointReceipt('original-context-closed');
    context = await browser.newContext({ ...contextOptions, storageState: authentication });
    await checkpointReceipt('fresh-browser-context-created');
    let activeFontCaptureContext = 'fresh-reopened-context';
    watchFonts(context, activeFontCaptureContext);
    const reopenedProfile = await rpc(context, identity.origin, 'get-profile', {}, decode);
    assert.equal(reopenedProfile.id, profile.id);
    receipt['newAuthenticatedContext'] = true;
    await checkpointReceipt('fresh-context-authenticated');
    let reopened = await openWorkspace(context, workspaceUrl, plugin.host, manifest, bundleSha256, pageErrors, decode,
      (page) => selectActualAssetBoard(page, metadata[0].boardId, receipt), existingRegistration, execution.mode);
    // No import or review rendering is performed in this new context.
    const reloadedFile = await rpc<BackendFile>(context, identity.origin, 'get-file', { id: identity.fileId }, decode);
    const reloadedMetadata = persistedZuiMetadata(reloadedFile, identity.pageId);
    verifyReopenedFile(initial, reloadedFile, identity, Math.max(...saves.map((save) => save.revn)), metadata, reloadedMetadata);
    receipt['nativeCopyFreshReopen'] = assertNativeInstanceGeometryPreserved(saved, reloadedFile);
    if (options.append) {
      assertExistingPenpotContentPreserved(initial, reloadedFile);
      if (sourceGapEditTarget) assert.deepEqual(record(reloadedFile.data)['components'] ?? {}, record(initial.data)['components'] ?? {},
        'Fresh AI source reopen must preserve the exact component master set');
      record(receipt['append'])['reopenedForeignContentPreserved'] = true;
    }
    const reopenedGroups = persistedToolbarGroups(reloadedFile, identity.pageId, reloadedMetadata[0], importedDocument);
    assert.deepEqual(reopenedGroups, toolbarGroups, 'Actual fresh-context original group metadata differs from the acknowledged saved file');
    await writeFile(resolve(output, 'reopened-toolbar-group-metadata.json'), `${JSON.stringify(reopenedGroups, null, 2)}\n`);
    receipt['viewportToolbarGroupsMatched'] = true;
    receipt['reopenedViewportToolbarGroupCount'] = reopenedGroups.length;
    receipt['reopenedSameFile'] = true;
    receipt['metadataMatched'] = true;
    receipt['reopenedRevision'] = { revn: reloadedFile.revn, vern: reloadedFile.vern };
    await writeFile(resolve(output, 'reopened-zui-metadata.json'), `${JSON.stringify(reloadedMetadata, null, 2)}\n`);
    await checkpointReceipt('reopened-metadata-verified');
    const exportButton = reopened.frame.getByRole('button', { name: 'Export selected' });
    await exportButton.waitFor({ timeout: 30_000 });
    receipt['actualExportUiBeforeClick'] = await captureExportUi(reopened.frame, reopened.page);
    await checkpointReceipt('actual-export-ui-ready');
    await capturePreviewLayoutAudit(reopened.frame, reloadedMetadata[0].boardId, output, receipt, importedDocument);
    await checkpointReceipt('post-reopen-layout-audit-written');
    const download = await withSourceColorRevisionFence(context, identity, reloadedMetadata[0].boardId,
      importedDocument, decode, receipt, () => clickAndWaitForDownload(
      () => reopened.page.waitForEvent('download', { timeout: timeoutMs }), () => observeActualExportClick(reopened.page, exportButton, output, receipt)));
    const exportPath = resolve(output, 'exported.zui');
    receipt['exportDownloadObserved'] = true;
    await checkpointReceipt('export-download-observed');
    await download.saveAs(exportPath);
    assert.equal(await download.failure(), null);
    receipt['exportPath'] = exportPath;
    receipt['exportSha256'] = sha256(await readFile(exportPath));
    receipt['exportSuggestedFileName'] = basename(download.suggestedFilename());
    await checkpointReceipt('export-download-saved');
    const exported = await readFile(exportPath, 'utf8');
    assert.deepEqual(normalizeZuiDocument(parseZuiDocument(exported).document), normalizeZuiDocument(original),
      'The existing selected-asset export must preserve the complete original ZUI contract');
    receipt['exportSemanticMatch'] = true;
    if (options.append) {
      assert.equal(sha256(exported), sourceSha256, 'Source append no-op export must preserve the exact original bytes');
      receipt['exportByteMatch'] = true;
    }
    assert.equal(sha256(await readFile(sourcePath)), receipt['sourceSha256'], 'The owning source changed during the real session');
    receipt['sourceFingerprintMatched'] = true;
    if (prepared) {
      for (const fingerprint of prepared.sourceFingerprints) {
        assert.equal(sha256(await readFile(resolve(required('ZUI_LAYOUT_REPO_ROOT'), fingerprint.sourcePath))),
          fingerprint.sha256, `An owning dependency changed during the real session: ${fingerprint.sourcePath}`);
      }
      receipt['sourceOwnerFingerprintsMatched'] = true;
    }
    receipt['exportPath'] = exportPath;
    receipt['exportSha256'] = sha256(exported);
    receipt['exportSuggestedFileName'] = basename(download.suggestedFilename());
    if (options.append) {
      receipt['canvasPngExport'] = { requested: false,
        reason: 'Bounded source-owner append verifies the official ZUI bytes; no raster evidence claimed' };
    } else {
      await downloadActualCanvasPng(
        reopened.page,
        reopened.frame,
        reloadedMetadata[0].boardId,
        output,
        receipt,
      );
      await checkpointReceipt('actual-canvas-png-export-saved');
    }
    await checkpointReceipt('persistence-gates-verified');
    await reopened.page.screenshot({ path: resolve(output, 'reopened-penpot.png'), fullPage: true, timeout: 30_000 });
    const editMinimumHeight = process.argv.includes('--edit-outliner-min-height');
    if (process.argv.includes('--edit-workbench-gap') || editMinimumHeight || sourceGapEditTarget) {
      assert.ok(prepared, 'A real Inspector edit requires its prepared canonical source owner');
      const postEditOutput = resolve(output, 'post-edit');
      await mkdir(postEditOutput);
      const editSaves: SaveEvidence[] = [];
      const editSaveHttpResponses: RecordValue[] = [];
      const postEdit: RecordValue = { saveAcknowledged: false, closedEditedContext: false,
        newAuthenticatedContext: false, reopenedSameFile: false, metadataMatched: false,
        exportSemanticMatch: false, completePostEditChainVerified: false,
        fileId: identity.fileId, pageId: identity.pageId, boardId: reloadedMetadata[0].boardId,
        preEditRevision: { revn: reloadedFile.revn, vern: reloadedFile.vern },
        saveResponses: editSaves, saveHttpResponses: editSaveHttpResponses };
      receipt['postEditPersistence'] = postEdit;
      let editSaveError: unknown;
      postEdit['saveCaptureConfiguration'] = await observeRealSaveHttp(reopened.page, {
        fileId: identity.fileId, output: postEditOutput, pending: savePending, diagnostics: editSaveHttpResponses,
        onResponse: async (response: ActualSaveHttp): Promise<void> => {
          const saveRequest = decode(response.requestBody.toString('utf8')) as SaveRequest;
          assert.equal(saveRequest.id, identity.fileId, 'The edited save must identify the actual file');
          assert.equal(new URL(response.url).searchParams.get('id'), saveRequest.id);
          assert.ok(response.httpStatus >= 200 && response.httpStatus < 300,
            `Actual edited update-file returned HTTP ${response.httpStatus}`);
          assert.ok(response.contentType.includes('transit+json'));
          const revision = verifySaveResponse(decode(response.responseBody.toString('utf8')),
            identity, reloadedFile.revn, saveRequest);
          editSaves.push({ httpStatus: response.httpStatus, url: response.url, requestId: response.requestId,
            responseSha256: sha256(response.responseBody), requestSha256: sha256(response.requestBody),
            requestFile: response.requestFile, responseFile: response.responseFile, revn: revision, observedAt: Date.now() });
          postEdit['saveAcknowledged'] = true;
          await checkpointReceipt('post-edit-save-acknowledged');
        },
        onError: (error) => { editSaveError = error; },
      });
      await checkpointReceipt('post-edit-save-observer-ready');
      const runtime = { page: reopened.page, pluginFrame: reopened.frame,
        fileInput: reopened.frame.locator('input[type="file"][accept*=".zui"]').first(), statusPanel: reopened.frame.locator('.status-panel'),
        exportButton } as unknown as Parameters<typeof editWorkbenchSpacingInPenpot>[0];
      const minimumEdit = editMinimumHeight ? await editOutlinerMinimumHeightInPenpot(runtime) : null;
      const gapEdit = editMinimumHeight ? null : sourceGapEditTarget
        ? await editSourceOwnerSpacingInPenpot(runtime, sourceGapEditTarget)
        : await editWorkbenchSpacingInPenpot(runtime);
      const edit = minimumEdit ?? gapEdit;
      assert.ok(edit);
      if (minimumEdit) {
        assert.equal(edit.identity.sourcePath, 'zircon_editor/assets/ui/editor/unreal/components/outliner.zui');
        assert.equal(edit.identity.sourceNodeId, 'panel');
      } else if (sourceGapEditTarget) {
        assert.equal(edit.identity.sourcePath, sourceGapEditTarget.sourcePath);
        assert.equal(edit.identity.sourceNodeId, sourceGapEditTarget.sourceNodeId);
        assert.equal(edit.identity.controlId, sourceGapEditTarget.controlId);
        assert.deepEqual(edit.identity.instancePath, []);
      } else {
        assert.equal(edit.identity.sourceNodeId, 'editor_content');
        assert.equal(edit.identity.controlId, 'UnrealLevelEditorContent');
      }
      receipt[sourceGapEditTarget ? 'sourceOwnerEdit' : 'workbenchEdit'] = edit;
      await reopened.page.screenshot({ path: resolve(output, minimumEdit ? 'edited-min-height-penpot.png' : 'edited-gap-penpot.png'), fullPage: true, timeout: 30_000 });
      const expected = cloneZuiDocument(original);
      const targetNodeId = minimumEdit ? 'outliner' : sourceGapEditTarget?.sourceNodeId ?? 'editor_content';
      const target = expected.nodes?.[targetNodeId];
      assert.ok(target);
      const expectedAppliedPath = minimumEdit ? 'nodes.outliner.layout.height.min' : `nodes.${targetNodeId}.layout.container.gap`;
      if (minimumEdit) {
        const dimension = record(record(target['layout'])['height']);
        assert.equal(dimension['min'], minimumEdit.before.heightMin);
        assert.equal(dimension['min'], 96);
        assert.equal(minimumEdit.after.heightMin, 128);
        dimension['min'] = minimumEdit.after.heightMin;
      } else {
        assert.ok(gapEdit);
        const container = record(record(target['layout'])['container']);
        const changedGap = gapEdit.after.rowGap !== gapEdit.before.rowGap
          ? gapEdit.after.rowGap : gapEdit.after.columnGap;
        assert.ok(Number.isFinite(changedGap));
        assert.equal(changedGap, gapEdit.before.rowGap + 1,
          'The actual inspector edit must change the scalar owning gap by one logical pixel');
        if (sourceGapEditTarget) {
          assert.equal(container['gap'], sourceGapEditTarget.beforeGap);
          assert.equal(changedGap, sourceGapEditTarget.afterGap);
        }
        container['gap'] = changedGap;
      }
      const editedShapeState = (file: BackendFile): RecordValue => {
        const shape = record(file.data['pages-index'][identity.pageId].objects[edit.shapeId]);
        const geometry = Object.fromEntries(['x', 'y', 'width', 'height'].map((key) => {
          assert.ok(typeof shape[key] === 'number' && Number.isFinite(shape[key]), `Saved shape has invalid ${key}`);
          return [key, shape[key]];
        }));
        if (minimumEdit) {
          const baseline = record(reloadedFile.data['pages-index'][identity.pageId].objects[edit.shapeId]);
          assert.equal(baseline['layout-item-min-h'], minimumEdit.before.heightMin);
          assert.equal(baseline['layout-item-v-sizing'], 'fix');
          assert.equal(shape['layout-item-min-h'], minimumEdit.after.heightMin, 'Saved minimum height must match the actual Inspector edit.');
          assert.equal(shape['layout-item-v-sizing'], 'fix', 'Saved Outliner must retain Fixed sizing.');
          assert.equal(shape['layout-item-h-sizing'], baseline['layout-item-h-sizing']);
          for (const key of ['x', 'y', 'width', 'height']) {
            assert.ok(typeof baseline[key] === 'number' && Number.isFinite(baseline[key]));
            assert.ok(Math.abs(Number(geometry[key]) - Number(baseline[key])) <= 0.01,
              `The saved Fixed Outliner geometry differs from baseline: ${key}`);
          }
          return { shapeId: edit.shapeId, heightMin: shape['layout-item-min-h'],
            verticalSizing: shape['layout-item-v-sizing'], horizontalSizing: shape['layout-item-h-sizing'], geometry };
        }
        assert.ok(gapEdit);
        const gap = record(shape['layout-gap']);
        assert.equal(gap['row-gap'], gapEdit.after.rowGap, 'Saved row gap differs from the actual inspector edit');
        assert.equal(gap['column-gap'], gapEdit.after.columnGap, 'Saved column gap differs from the actual inspector edit');
        return { shapeId: edit.shapeId, gap, geometry };
      };
      let savedAfterEdit: BackendFile | undefined;
      let savedEditMetadata: ReturnType<typeof persistedZuiMetadata> = [];
      const editSaveDeadline = Date.now() + timeoutMs;
      while (Date.now() < editSaveDeadline) {
        if (editSaveError) throw editSaveError;
        if (editSaves.length && Date.now() - editSaves[editSaves.length - 1].observedAt >= 1_500) {
          const file = await rpc<BackendFile>(context, identity.origin, 'get-file', { id: identity.fileId }, decode);
          const stored = persistedZuiMetadata(file, identity.pageId);
          const acknowledgedRevision = Math.max(...editSaves.map((save) => save.revn));
          if (file.revn >= acknowledgedRevision) {
            // Official PNG capture clears these transient values to empty strings.
            // All other metadata remains exact, including every reversible source field.
            const sourceMetadata = (entries: typeof stored) => entries.map((entry) => {
              const metadata = { ...entry.metadata };
              for (const key of ['review-scroll-captured-x', 'review-scroll-captured-y',
                'review-scroll-captured-positions']) {
                if (metadata[key] === '') delete metadata[key];
              }
              return { ...entry, metadata };
            });
            verifyReopenedFile(reloadedFile, file, identity, acknowledgedRevision,
              sourceMetadata(reloadedMetadata), sourceMetadata(stored));
            if (sourceGapEditTarget) {
              assertExistingPenpotContentPreserved(initial, file);
              assert.deepEqual(record(file.data)['components'] ?? {}, record(initial.data)['components'] ?? {},
                'Editing the isolated AI source must not create or change component masters');
              postEdit['savedForeignContentPreserved'] = true;
            }
            postEdit['savedShape'] = editedShapeState(file);
            savedAfterEdit = file;
            savedEditMetadata = stored;
            break;
          }
        }
        await reopened.page.waitForTimeout(500);
      }
      assert.ok(savedAfterEdit, 'No acknowledged backend revision containing the actual edited shape was observed');
      await Promise.all(savePending);
      if (editSaveError) throw editSaveError;
      postEdit['savedRevision'] = { revn: savedAfterEdit.revn, vern: savedAfterEdit.vern };
      await writeFile(resolve(postEditOutput, 'saved-zui-metadata.json'), `${JSON.stringify(savedEditMetadata, null, 2)}\n`);
      await checkpointReceipt('post-edit-saved-shape-verified');
      const editedAuthentication = await context.storageState();
      await context.close();
      postEdit['closedEditedContext'] = true;
      await checkpointReceipt('post-edit-context-closed');
      context = await browser.newContext({ ...contextOptions, storageState: editedAuthentication });
      activeFontCaptureContext = 'post-edit-fresh-reopened-context';
      watchFonts(context, activeFontCaptureContext);
      const editedReopenedProfile = await rpc(context, identity.origin, 'get-profile', {}, decode);
      assert.equal(editedReopenedProfile.id, profile.id);
      postEdit['newAuthenticatedContext'] = true;
      reopened = await openWorkspace(context, workspaceUrl, plugin.host, manifest, bundleSha256, pageErrors, decode,
        (page) => selectActualAssetBoard(page, reloadedMetadata[0].boardId, postEdit), existingRegistration, execution.mode);
      const editedReopenedFile = await rpc<BackendFile>(context, identity.origin, 'get-file', { id: identity.fileId }, decode);
      const editedReopenedMetadata = persistedZuiMetadata(editedReopenedFile, identity.pageId);
      verifyReopenedFile(savedAfterEdit, editedReopenedFile, identity,
        Math.max(...editSaves.map((save) => save.revn)), savedEditMetadata, editedReopenedMetadata);
      postEdit['nativeCopyFreshReopen'] = assertNativeInstanceGeometryPreserved(savedAfterEdit, editedReopenedFile);
      assert.equal(editedReopenedMetadata[0].boardId, reloadedMetadata[0].boardId);
      if (sourceGapEditTarget) {
        assertExistingPenpotContentPreserved(initial, editedReopenedFile);
        assert.deepEqual(record(editedReopenedFile.data)['components'] ?? {}, record(initial.data)['components'] ?? {},
          'Edited source fresh reopen must preserve the exact original masters');
        postEdit['reopenedForeignContentPreserved'] = true;
      }
      assert.deepEqual(editedShapeState(editedReopenedFile), postEdit['savedShape'],
        'The fresh authenticated context must reopen the same persisted edited shape and geometry');
      postEdit['reopenedSameFile'] = true;
      postEdit['metadataMatched'] = true;
      postEdit['reopenedRevision'] = { revn: editedReopenedFile.revn, vern: editedReopenedFile.vern };
      await writeFile(resolve(postEditOutput, 'reopened-zui-metadata.json'), `${JSON.stringify(editedReopenedMetadata, null, 2)}\n`);
      const editedReopenedAudit = await capturePreviewLayoutAudit(reopened.frame, editedReopenedMetadata[0].boardId, postEditOutput, postEdit, importedDocument);
      if (sourceGapEditTarget) {
        assert.deepEqual(editedReopenedAudit['invalidGeometryNodeIds'], []);
        assert.deepEqual(editedReopenedAudit['overflowNodeIds'], []);
        const matching = (editedReopenedAudit['semanticNodes'] as unknown[]).map(record).filter(node =>
          node['shapeId'] === edit.shapeId && node['sourcePath'] === sourceGapEditTarget.sourcePath &&
          node['sourceNodeId'] === sourceGapEditTarget.sourceNodeId && node['controlId'] === sourceGapEditTarget.controlId &&
          node['instancePath'] === '[]' && node['visible'] === true && node['detached'] === false);
        assert.equal(matching.length, 1, 'Fresh source edit audit must retain the exact admitted source and shape');
        postEdit['freshReopenedSourceIdentityVerified'] = true;
      }
      if (minimumEdit) {
        assert.deepEqual(editedReopenedAudit['invalidGeometryNodeIds'], []);
        assert.deepEqual(editedReopenedAudit['overflowNodeIds'], []);
        const candidates = (editedReopenedAudit['semanticNodes'] as unknown[]).map(record).filter((node) =>
          node['shapeId'] === edit.shapeId && node['sourcePath'] === edit.identity.sourcePath &&
          node['sourceNodeId'] === edit.identity.sourceNodeId &&
          node['instancePath'] === JSON.stringify(edit.identity.instancePath));
        assert.equal(candidates.length, 1, 'The fresh reopened audit must retain the exact Outliner source and shape.');
        const actual = candidates[0]!;
        assert.equal(actual['visible'], true);
        assert.equal(actual['detached'], false);
        const bounds = record(actual['bounds']);
        for (const axis of ['x', 'y', 'width', 'height'] as const) {
          assert.ok(typeof bounds[axis] === 'number' && Number.isFinite(bounds[axis]));
          assert.ok(Math.abs(Number(bounds[axis]) - minimumEdit.before.bounds[axis]) <= 0.01,
            `The fresh reopened Outliner must retain its baseline Fixed bounds: ${axis}`);
        }
        postEdit['freshReopenedOutlinerBoundsVerified'] = true;
      }
      await checkpointReceipt('post-edit-fresh-reopened-shape-verified');
      const postExportButton = reopened.frame.getByRole('button', { name: 'Export selected' });
      await postExportButton.waitFor({ timeout: 30_000 });
      const editedDownload = await withSourceColorRevisionFence(context, identity, editedReopenedMetadata[0].boardId,
        importedDocument, decode, postEdit, () => clickAndWaitForDownload(
        () => reopened.page.waitForEvent('download', { timeout: timeoutMs }),
        () => observeActualExportClick(reopened.page, postExportButton, postEditOutput, postEdit)));
      const editedPath = resolve(output, minimumEdit ? 'outliner-min-height-edited.penpot-export.zui'
        : sourceGapEditTarget ? 'source-gap-edited.penpot-export.zui' : 'workbench-gap-edited.penpot-export.zui');
      await editedDownload.saveAs(editedPath);
      assert.equal(await editedDownload.failure(), null);
      const editedSource = await readFile(editedPath, 'utf8');
      const owner = sourceGapEditTarget?.sourcePath ?? 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
      const appliedRoot = resolve(required('ZUI_LAYOUT_REPO_ROOT'));
      assert.ok(prepared);
      const sourceExportManifest = parseSourceExportManifest(parseZuiDocument(editedSource).document['penpot_source_exports']);
      const sourceOwnerCount = sourceGapEditTarget
        ? validateRealOwnerGapExport(sourceExportManifest, prepared.sourceFingerprints, sourceSha256, sourceGapEditTarget)
        : validateRealWorkbenchExportSources(sourceExportManifest, prepared.sourceFingerprints, sourceSha256);
      // Trust the prepared canonical closure, never source contents claimed by the downloaded envelope.
      const canonicalSources: Record<string, string> = Object.fromEntries(await Promise.all(
        prepared.sourceFingerprints.map(async (fingerprint) => {
          const canonicalSource = await readFile(resolve(appliedRoot, ...validateSourcePath(fingerprint.sourcePath)), 'utf8');
          assert.equal(sha256(canonicalSource), fingerprint.sha256, 'Canonical source changed before no-write owner validation');
          return [fingerprint.sourcePath, canonicalSource] as const;
        })));
      assert.equal(canonicalSources[owner], source, 'Validate the original owning source');
      const planned = applyZuiExportToSourceFiles(editedSource, canonicalSources, prepared.sourceFingerprints);
      assert.deepEqual(planned.outputs.map((item) => item.sourcePath), [owner]);
      assert.deepEqual(planned.appliedPathsBySource, { [owner]: [expectedAppliedPath] });
      const projectedSources = { ...canonicalSources,
        ...Object.fromEntries(planned.outputs.map((item) => [item.sourcePath, item.source])) };
      for (const fingerprint of prepared.sourceFingerprints) {
        if (fingerprint.sourcePath !== owner) {
          assert.equal(projectedSources[fingerprint.sourcePath], canonicalSources[fingerprint.sourcePath],
            'Every other canonical owner must remain byte exact');
          assert.deepEqual(normalizeZuiDocument(parseZuiDocument(projectedSources[fingerprint.sourcePath]).document),
            normalizeZuiDocument(parseZuiDocument(canonicalSources[fingerprint.sourcePath]).document));
        }
        assert.equal(await readFile(resolve(appliedRoot, ...validateSourcePath(fingerprint.sourcePath)), 'utf8'),
          canonicalSources[fingerprint.sourcePath], 'No-write owner validation must preserve all canonical sources');
      }
      assert.deepEqual(normalizeZuiDocument(parseZuiDocument(planned.outputs[0].source).document),
        normalizeZuiDocument(expected), 'The fresh-reopened actual export must preserve exactly the owning Inspector edit');
      postEdit['sourceOwnerExportValidation'] = { ownerCount: sourceOwnerCount, unchangedOtherOwnerCount: sourceOwnerCount - 1,
        appliedPathsBySource: planned.appliedPathsBySource, filesystemWrites: false,
        plannedRootSha256: sha256(planned.outputs[0].source) };
      postEdit['exportSemanticMatch'] = true;
      postEdit['exportPath'] = editedPath;
      postEdit['exportSha256'] = sha256(editedSource);
      if (!sourceGapEditTarget) {
        await downloadActualCanvasPng(reopened.page, reopened.frame, editedReopenedMetadata[0].boardId, postEditOutput, postEdit);
      } else {
        postEdit['canvasPngExport'] = { requested: false, reason: 'This source-owner edit proves ZUI bytes, not raster or Engine acceptance' };
      }
      await reopened.page.screenshot({ path: resolve(postEditOutput, 'reopened-penpot.png'), fullPage: true, timeout: 30_000 });
      await checkpointReceipt('post-edit-official-exports-verified');
      assert.equal(await readFile(sourcePath, 'utf8'), source, 'Owning source changed before real Penpot apply');
      assert.equal(sha256(await readFile(editedPath)), postEdit['exportSha256'], 'Validated export changed before real Penpot apply');
      if (sourceGapEditTarget) {
        // The original domain owner applies the validated export in a separate authorized step.
        receipt['editedExportPath'] = editedPath;
        receipt['editedExportSha256'] = sha256(editedSource);
        receipt['sourceOwnerEditedExportVerified'] = true;
        receipt['sourceOwnerApplyVerified'] = false;
        receipt['protectedFieldsPreserved'] = true;
        postEdit['canonicalSourceApplyPerformed'] = false;
        postEdit['completeEditedSourceRoundtripVerified'] = true;
        await checkpointReceipt('post-edit-source-owner-validated-no-write');
      } else {
        const applied = await applyZuiExportFileToCanonicalSourceOwners({ exportedPath: editedPath,
          sourceRootPath: appliedRoot, writableSourcePaths: [owner],
          expectedSources: prepared.sourceFingerprints, expectedRootSha256: sourceSha256 });
        assert.deepEqual(applied.outputs.map((item) => item.outputPath), [resolve(appliedRoot, owner)]);
        assert.deepEqual(applied.appliedPathsBySource[owner], [expectedAppliedPath]);
        const appliedSource = await readFile(resolve(appliedRoot, owner), 'utf8');
        assert.deepEqual(normalizeZuiDocument(parseZuiDocument(appliedSource).document), normalizeZuiDocument(expected),
          'Only the owning Inspector leaf may change; stable IDs, references, events, bindings and other fields must remain exact');
        receipt['editedExportPath'] = editedPath;
        receipt['editedExportSha256'] = sha256(await readFile(editedPath));
        receipt['appliedSourceRoot'] = appliedRoot;
        receipt['appliedSourcePath'] = resolve(appliedRoot, owner);
        receipt['appliedSourceSha256'] = sha256(appliedSource);
        receipt['engineVerificationAfterApply'] = 'pending-new-normal-window-capture';
        receipt['sourceOwnerApplyVerified'] = true;
        receipt['protectedFieldsPreserved'] = true;
        postEdit['completePostEditChainVerified'] = true;
        await checkpointReceipt('post-edit-persistence-and-owner-apply-verified');
      }
    }
    if (process.env['PENPOT_VERIFY_WASM_FONT_REGISTRATION'] === '1') {
      let manifestSha256: string | null = null;
      let registrationReceiptSha256: string | null = null;
      let observation: RecordValue = { observationContext: activeFontCaptureContext, bridgeState: 'manifest-unavailable',
        manifestSha256, boardTextUsage: { state: 'not-evaluated' }, faces: [] };
      await Promise.all([...fontPending]);
      try {
        assert.ok(fontResourcePath, 'WASM font observation requires the source font resource manifest');
        const manifestBytes = await readFile(fontResourcePath);
        manifestSha256 = sha256(manifestBytes);
        const fontManifest = JSON.parse(manifestBytes.toString('utf8')) as RecordValue;
        assert.equal(fontManifest['schema'], 'zircon.penpot.font-resources.v1');
        assert.ok(Array.isArray(fontManifest['selections']));
        const registrationPath = process.env['PENPOT_REAL_FONT_REGISTRATION_RECEIPT'];
        assert.ok(registrationPath, 'WASM font observation requires the actual exact-font registration receipt');
        const registrationBytes = await readFile(artifactFile(registrationPath));
        registrationReceiptSha256 = sha256(registrationBytes);
        const registration = JSON.parse(registrationBytes.toString('utf8')) as RecordValue;
        assert.equal(registration['schema'], 'zircon.penpot.exact-font-registration.v1');
        assert.equal(registration['status'], 'passed');
        assert.ok(Array.isArray(registration['faces']));
        const registeredFaces = registration['faces'] as RecordValue[];
        const faces: RecordValue[] = [];
        for (const rawSelection of fontManifest['selections']) {
          assert.ok(rawSelection && typeof rawSelection === 'object');
          const selection = rawSelection as RecordValue;
          const source = selection['source'] as RecordValue;
          const engine = selection['engine'] as RecordValue;
          const penpot = selection['penpot'] as RecordValue;
          const fontId = penpot?.['fontId'];
          if (typeof fontId !== 'string' || !fontId.startsWith('custom-')) continue;
          assert.equal(typeof penpot['variantId'], 'string');
          assert.equal(typeof source?.['weight'], 'string');
          assert.match(String(source['weight']), /^[1-9][0-9]{0,2}$|^1000$/);
          assert.ok(source['style'] === 'normal' || source['style'] === 'italic');
          assert.equal(String(penpot['sfntSha256']).toLowerCase(), String(engine?.['sfntSha256']).toLowerCase());
          const registered = registeredFaces.filter((face) =>
            face['pluginFontId'] === fontId && face['pluginVariantId'] === penpot['variantId']);
          assert.equal(registered.length, 1, 'Each font tuple requires one exact registration receipt face');
          const ttf = registered[0]['ttf'] as RecordValue;
          assert.ok(ttf && typeof ttf['assetId'] === 'string');
          assert.match(ttf['assetId'], /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i);
          assert.equal(String(ttf['sha256']).toLowerCase(), String(penpot['sfntSha256']).toLowerCase());
          const expectedFontAssetPath = '/assets/by-id/' + ttf['assetId'];
          faces.push({ fontId, variantId: penpot['variantId'], weight: Number(source['weight']),
            style: source['style'] === 'normal' ? 0 : 1, sourceFamily: source['family'], sourceStyle: source['style'],
            expectedEngineAssetPath: engine['assetPath'], expectedEngineAssetSha256: engine['assetSha256'],
            expectedSfntSha256: penpot['sfntSha256'], expectedFontAssetId: ttf['assetId'],
            expectedFontAssetPath, expectedFontAssetUrl: new URL(expectedFontAssetPath, identity.origin).href });
        }
        const bridge = await reopened.page.evaluate(async (inputFaces: RecordValue[]) => {
          const emptyRows = (state: string, ready: boolean | null) =>
            inputFaces.map((face) => ({ ...face, queryState: state, ready, uuidWords: null, rawResult: null, stored: null }));
          try {
            const registry = await import(new URL('/js/shared.js', location.href).href);
            const app = (registry as unknown as { $APP: RecordValue }).$APP;
            const readyFn = app['$app$common$render_wasm$wasm$ready_QMARK_$$'] as (() => boolean) | undefined;
            const uuidFn = app['$cljs$core$uuid$$'] as ((value: string) => unknown) | undefined;
            const wordsFn = app['$app$common$uuid$get_u32$$'] as ((value: unknown) => Uint32Array) | undefined;
            if (!readyFn || !uuidFn || !wordsFn)
              return { state: 'bridge-member-unavailable', ready: null, faces: emptyRows('bridge-member-unavailable', null) };
            const ready = readyFn();
            const wasm = app['$app$common$render_wasm$wasm$internal_module$$'] as RecordValue | undefined;
            const predicate = wasm?.['_is_font_uploaded'] as
              ((a: number, b: number, c: number, d: number, weight: number, style: number, emoji: boolean) => unknown) | undefined;
            const faces = inputFaces.map((face) => {
              const uuidMatch = /^custom-([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})$/i
                .exec(String(face['fontId']));
              if (!uuidMatch) return { ...face, queryState: 'invalid-font-id', ready, uuidWords: null, rawResult: null, stored: null };
              try {
                const words = wordsFn(uuidFn(uuidMatch[1]));
                if (!(words instanceof Uint32Array) || words.length !== 4)
                  return { ...face, queryState: 'invalid-uuid-words', ready, uuidWords: null, rawResult: null, stored: null };
                const uuidWords = Array.from(words);
                if (!ready) return { ...face, queryState: 'wasm-not-ready', ready, uuidWords, rawResult: null, stored: null };
                if (!predicate) return { ...face, queryState: 'predicate-unavailable', ready, uuidWords, rawResult: null, stored: null };
                const rawResult = predicate(words[0], words[1], words[2], words[3],
                  Number(face['weight']), Number(face['style']), false);
                const stored = rawResult === 1 || rawResult === true ? true
                  : rawResult === 0 || rawResult === false ? false : null;
                const serializableRawResult = typeof rawResult === 'number' || typeof rawResult === 'boolean'
                  ? rawResult : String(rawResult);
                return { ...face, queryState: stored === null ? 'invalid-result' : 'queried', ready, uuidWords,
                  rawResult: serializableRawResult, rawResultType: typeof rawResult, stored };
              } catch (error) {
                return { ...face, queryState: 'tuple-query-error', ready, uuidWords: null, rawResult: null,
                  stored: null, error: String(error) };
              }
            });
            return { state: ready ? (predicate ? 'queried' : 'predicate-unavailable') : 'wasm-not-ready', ready, faces };
          } catch (error) {
            return { state: 'probe-error', ready: null, error: String(error),
              faces: emptyRows('probe-error', null) };
          }
        }, faces);
        await Promise.all([...fontPending]);
        const freshContextHttpResponses = fontResponses.filter((response) =>
          response['captureContext'] === activeFontCaptureContext);
        const probedFaces = Array.isArray(bridge['faces']) ? bridge['faces'] as RecordValue[] : [];
        observation = { observationContext: activeFontCaptureContext, manifestSha256, registrationReceiptSha256, bridgeState: bridge['state'],
          wasmReady: bridge['ready'], boardTextUsage: { state: 'not-evaluated' },
          faces: probedFaces.map((face) => {
            const matchingHttpResponses = freshContextHttpResponses.filter((response) => {
              if (response['httpStatus'] !== 200 || response['fontMagicDetected'] !== true
                || String(response['sha256']).toLowerCase() !== String(face['expectedSfntSha256']).toLowerCase()) return false;
              try { return new URL(String(response['url'])).href === face['expectedFontAssetUrl']; }
              catch { return false; }
            });
            return { ...face, responseObserved: matchingHttpResponses.length > 0, matchingHttpResponses };
          }),
          freshContextHttpResponses };
      } catch (error) {
        observation = { observationContext: activeFontCaptureContext, bridgeState: 'manifest-or-probe-error',
          manifestSha256, registrationReceiptSha256, error: String(error), boardTextUsage: { state: 'not-evaluated' }, faces: [] };
      }
      receipt['wasmFontRegistrations'] = observation;
      await checkpointReceipt('wasm-font-registration-observed');
    }
    receipt['domFontStates'] = await Promise.all((context?.pages() ?? []).map(captureDomFonts));
    if (sourceGapEditTarget) {
      assert.equal(receipt['sourceOwnerEditedExportVerified'], true);
      assert.equal(record(receipt['postEditPersistence'])['completeEditedSourceRoundtripVerified'], true);
      assert.equal(receipt['sourceOwnerApplyVerified'], false);
      assert.equal(await readFile(sourcePath, 'utf8'), source, 'AI source stays unchanged until its original owner applies the real export');
    }
    if (actualCaseInput)
      assert.equal(sha256(await readFile(actualCaseInput.path)), actualCaseInput.sha256,
        'Actual normal case changed during real Penpot persistence');
    receipt['status'] = 'passed';
    receipt['persistenceClaim'] = persistenceClaim(receipt);
    assert.equal(receipt['persistenceClaim'], true);
  } catch (error) {
    runFailed = true;
    if (error && typeof error === 'object' && 'nativeCopyDelta' in error)
      receipt['nativeCopyFailureDelta'] = error.nativeCopyDelta;
    // Playwright error call logs can contain authentication request headers.
    // Keep the concrete first-line failure without forwarding those headers.
    const failure = new Error((error instanceof Error ? error.message : String(error)).split(/\r?\n/, 1)[0]);
    receipt['status'] = 'failed';
    receipt['persistenceClaim'] = false;
    receipt['error'] = failure.message;
    receipt['failureStackLocations'] = error instanceof Error
      ? (error.stack ?? '').split(/\r?\n/).filter((line) => /^\s+at /.test(line)).slice(0, 12)
        .map((line) => line.replace(/(?:https?|file):\/\/[^\s)]+/g, safeDebuggerScriptURL)) : [];
    await checkpointReceipt('session-failed');
    const diagnostics: RecordValue[] = [];
    const observedImportProgress = receipt['importProgress'];
    const lastObservedImportStatus = Array.isArray(observedImportProgress)
      ? observedImportProgress[observedImportProgress.length - 1] : undefined;
    const diagnosticDeadlineMs = 10_000;
    const diagnosticDeadlineAt = Date.now() + diagnosticDeadlineMs;
    receipt['failureDiagnosticsDeadlineMs'] = diagnosticDeadlineMs;
    receipt['failureDiagnosticsDeadlineScope'] =
      'all failed-page CDP session creation, screenshot capture, PNG write and detach operations';
    for (const [index, page] of (context?.pages() ?? []).entries()) {
      if (Date.now() >= diagnosticDeadlineAt) {
        diagnostics.push({ diagnosticError: 'Actual CDP failure screenshot timed out after 10 seconds',
          lastObservedImportStatus });
        continue;
      }
      const screenshot = resolve(output, 'failed-page-' + index + '.png');
      let deadlineExpired = false;
      let diagnosticCdp: Awaited<ReturnType<BrowserContext['newCDPSession']>> | undefined;
      let detachPromise: Promise<void> | undefined;
      const detachOnce = (): Promise<void> => {
        if (!diagnosticCdp) return Promise.resolve();
        detachPromise ??= Promise.resolve().then(() => diagnosticCdp!.detach());
        void detachPromise.catch(() => undefined);
        return detachPromise;
      };
      const diagnosticTask = (async (): Promise<RecordValue> => {
        diagnosticCdp = await context!.newCDPSession(page);
        try {
          if (deadlineExpired || Date.now() >= diagnosticDeadlineAt)
            throw new Error('Actual CDP failure screenshot timed out after 10 seconds');
          const captured = await diagnosticCdp.send('Page.captureScreenshot',
            { format: 'png', captureBeyondViewport: false });
          if (deadlineExpired || Date.now() >= diagnosticDeadlineAt)
            throw new Error('Actual CDP failure screenshot timed out after 10 seconds');
          await writeFile(screenshot, Buffer.from(captured.data, 'base64'));
          if (deadlineExpired || Date.now() >= diagnosticDeadlineAt)
            throw new Error('Actual CDP failure screenshot timed out after 10 seconds');
          return { url: page.url(), screenshot, screenshotMethod: 'actual CDP capture; no font readiness claim',
            frameUrls: page.frames().map((frame) => frame.url()), lastObservedImportStatus };
        } finally {
          await detachOnce();
        }
      })();
      void diagnosticTask.catch(() => undefined);
      let diagnosticTimer: ReturnType<typeof setTimeout> | undefined;
      try {
        const diagnostic = await Promise.race([
          diagnosticTask,
          new Promise<never>((_, reject) => {
            diagnosticTimer = setTimeout(() => {
              deadlineExpired = true;
              void detachOnce().catch(() => undefined);
              reject(new Error('Actual CDP failure screenshot timed out after 10 seconds'));
            }, Math.max(0, diagnosticDeadlineAt - Date.now()));
          }),
        ]);
        diagnostics.push(diagnostic);
      } catch (diagnosticError) {
        diagnostics.push({ diagnosticError: String(diagnosticError), lastObservedImportStatus });
      } finally {
        clearTimeout(diagnosticTimer);
      }
    }
    receipt['failureDiagnostics'] = diagnostics;
    receipt['failureDiagnosticsFinishedAtUtc'] = new Date().toISOString();
    receipt['failureDiagnosticsDeadlineExceeded'] = Date.now() >= diagnosticDeadlineAt;
    await checkpointReceipt('failure-diagnostics-complete');
    throw failure;
  } finally {
    await Promise.allSettled([...savePending]);
    if (importDebuggerCdp) {
      const cleanupStartedAt = Date.now();
      receipt['importDebuggerFinalCleanup'] = await releaseImportDebuggerSession(importDebuggerCdp,
        cleanupStartedAt + 1_000, cleanupStartedAt + 2_000);
      importDebuggerCdp = undefined;
    }
    const cleanupErrors: string[] = [];
    const closeOwned = async (name: string, operation: () => Promise<unknown>): Promise<void> => {
      let detail: string | undefined;
      const outcome = await settleBeforeNodeDeadline(
        operation().catch(error => { detail = String(error).split(/\r?\n/, 1)[0]; throw error; }), Date.now() + 15_000);
      resources[name] = { ...(resources[name] ? record(resources[name]) : {}), ownership: 'session', closeOutcome: outcome.status,
        ...(detail ? { error: detail } : {}), closedByThisSession: outcome.status === 'completed',
        ...(name === 'pluginHost' ? { stoppedByThisSession: outcome.status === 'completed' } : {}) };
      if (outcome.status !== 'completed') cleanupErrors.push(name + ': ' + (detail ?? outcome.status));
    };
    if (finalRegistryCheck) {
      try {
        receipt['pluginRegistryAfter'] = await finalRegistryCheck();
        receipt['pluginRegistryUnchanged'] = true;
      } catch (error) {
        receipt['pluginRegistryUnchanged'] = false;
        receipt['pluginRegistryVerificationError'] = String(error).split(/\r?\n/, 1)[0];
        cleanupErrors.push('Final registry verification: ' + String(error).split(/\r?\n/, 1)[0]);
      }
    }
    if (context) await closeOwned('browserContext', () => context!.close());
    if (browser) await closeOwned('browser', () => browser!.close());
    await Promise.allSettled([...fontPending]);
    if (api) await closeOwned('apiContext', () => api!.dispose());
    if (server) {
      await closeOwned('pluginHost', () => new Promise<void>((done, reject) =>
        server!.close(error => error ? reject(error) : done())));
    }
    if (originalAuthenticationFile) {
      try {
        assert.equal(sha256(await readFile(originalAuthenticationFile.path)), originalAuthenticationFile.sha256,
          'Original authentication storage changed during the session');
        receipt['originalAuthenticationUnchanged'] = true;
      } catch (error) {
        receipt['originalAuthenticationUnchanged'] = false;
        cleanupErrors.push('Original authentication verification: ' + String(error).split(/\r?\n/, 1)[0]);
      }
    }
    receipt['resourceCleanupErrors'] = cleanupErrors;
    receipt['nativeProcessTerminationVerifiedByRunner'] = false;
    if (cleanupErrors.length) {
      receipt['status'] = 'failed';
      receipt['persistenceClaim'] = false;
    }

    receipt['finishedAtUtc'] = new Date().toISOString();
    await receiptWrites;
    await writeFile(receiptTemporaryPath, `${JSON.stringify(receipt, null, 2)}\n`, { flush: true });
    await rename(receiptTemporaryPath, receiptPath);
    console.log(JSON.stringify({ realSessionReceipt: receiptPath, status: receipt['status'], persistenceClaim: receipt['persistenceClaim'] }));
    if (cleanupErrors.length && !runFailed) throw new Error('Real session cleanup/registry verification failed: ' + cleanupErrors.join('; '));
  }
}

export { pluginServer, transitDecoder, rpc, openWorkspace, selectActualAssetBoard, downloadActualCanvasPng };

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const run = async (): Promise<void> => {
    const optionsPath = process.env['PENPOT_REAL_SESSION_OPTIONS_JSON'];
    if (!optionsPath) {
      assert.ok(!process.env['PENPOT_REAL_SESSION_OPTIONS_SHA256'], 'Session options hash has no input file');
      return runRealPenpotPersistence();
    }
    const bytes = await readFile(artifactFile(optionsPath));
    const expected = required('PENPOT_REAL_SESSION_OPTIONS_SHA256');
    assert.match(expected, /^[0-9a-f]{64}$/);
    assert.equal(sha256(bytes), expected, 'Explicit session options changed');
    const options = record(JSON.parse(bytes.toString('utf8')));
    assert.ok(Object.keys(options).every(key => ['mode', 'expectedWorkspace', 'existingPlugin', 'append'].includes(key)), 'Unknown session option');
    await runRealPenpotPersistence(options as RealPenpotPersistenceOptions);
  };
  run().catch((error) => { console.error(error); process.exitCode = 1; });
}
