import { parse as parseToml } from 'smol-toml';
import { parseZuiLayoutSource } from '../src/bridge/zui-layout-optimizer';
import { FIELD_ICON_PATHS } from '../src/bridge/zui-field-projection';
import { existsSync, lstatSync, realpathSync } from 'node:fs';
import { mkdir, open, readFile, writeFile, type FileHandle } from 'node:fs/promises';
import { dirname, isAbsolute, relative, resolve } from 'node:path';
import {
  assertLayoutCurrentRoots, createLayoutCurrentContext, layoutCurrentOutputPath,
  layoutCurrentSourcePath, type LayoutCurrentContext,
} from './zui-layout-paths';
import { readCurrentLayoutOutputCatalog } from './zui-layout-gallery';
import { withLayoutCatalogLock } from './zui-layout-catalog-lock';
import { bytesSha256 } from './zui-layout-evidence';
import type { CatalogEntry, CatalogManifest } from './zui-layout-catalog';

export interface SourceRefinementContexts {
  readonly catalog: LayoutCurrentContext;
  /** One explicitly shared coordination root for cooperating source writers. */
  readonly report: LayoutCurrentContext;
}
export interface CurrentLayoutSnapshot {
  readonly context: LayoutCurrentContext;
  readonly manifest: CatalogManifest;
  readonly catalogSha256: string;
}
interface SnapshotState { hashes: Map<string, string>; catalogPath: string; manifestHash: string; absentInputs: Set<string> }
const snapshots = new WeakMap<CurrentLayoutSnapshot, SnapshotState>();
const normalize = (path: string) => process.platform === 'win32' ? path.toLowerCase() : path;
const contains = (root: string, path: string) => {
  const suffix = relative(root, path);
  return !isAbsolute(suffix) && suffix !== '..' && !/^\.\.[/\\]/.test(suffix);
};
function noAliases(path: string): void {
  let ancestor = resolve(path);
  while (!existsSync(ancestor)) {
    try { lstatSync(ancestor); throw new Error('Unresolved layout alias: ' + ancestor); }
    catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error; }
    const parent = dirname(ancestor);
    if (parent === ancestor) throw new Error('No physical layout parent');
    ancestor = parent;
  }
  if (normalize(realpathSync.native(ancestor)) !== normalize(ancestor))
    throw new Error('Layout aliases are not admitted: ' + path);
  for (let current = ancestor; ; current = dirname(current)) {
    if (lstatSync(current).isSymbolicLink()) throw new Error('Layout alias: ' + path);
    if (dirname(current) === current) break;
  }
}
export function assertSeparateCurrentReport(contexts: SourceRefinementContexts): void {
  for (const context of [contexts.catalog, contexts.report]) {
    assertLayoutCurrentRoots(context, context.repoRoot, context.outputRoot);
    noAliases(context.repoRoot); noAliases(context.outputRoot);
  }
  assertLayoutCurrentRoots(contexts.catalog, contexts.report.repoRoot, contexts.catalog.outputRoot);
  const input = realpathSync.native(contexts.catalog.outputRoot);
  const output = realpathSync.native(contexts.report.outputRoot);
  if (contains(input, output) || contains(output, input))
    throw new Error('Source report root must be disjoint from readonly current catalog');
  noAliases(layoutCurrentOutputPath(contexts.report, '.zui-layout-catalog.lock/owner.json'));
}
function sourceFile(context: LayoutCurrentContext, sourcePath: string): string {
  if (!sourcePath || isAbsolute(sourcePath) || /[:\\\x00-\x1f]/.test(sourcePath) ||
      sourcePath.split('/').some((part) => !part || part === '.' || part === '..' ||
        /[. ]$/.test(part) || /^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(part)))
    throw new Error('Invalid portable current source path');
  const path = layoutCurrentSourcePath(context, sourcePath);
  noAliases(path);
  if (!lstatSync(path).isFile()) throw new Error('Source is not a regular file: ' + sourcePath);
  return path;
}
function absent(path: string): void {
  noAliases(path);
  if (existsSync(path)) throw new Error('Refusing existing refinement output: ' + path);
  let parent = dirname(path);
  while (!existsSync(parent)) parent = dirname(parent);
  if (!lstatSync(parent).isDirectory()) throw new Error('Output parent is not a directory');
}

/** Readonly input closure; no lock, directory, browser or output is created. */
export async function prepareCurrentLayoutSnapshot(
  context: LayoutCurrentContext, additionalSources: readonly string[] = [], includeLoaderInputs = false,
): Promise<CurrentLayoutSnapshot> {
  assertLayoutCurrentRoots(context, context.repoRoot, context.outputRoot);
  noAliases(context.repoRoot); noAliases(context.outputRoot);
  const { manifest, catalogSha256 } = await readCurrentLayoutOutputCatalog(context);
  const catalogPath = layoutCurrentOutputPath(context, 'catalog.json');
  noAliases(catalogPath);
  const hashes = new Map<string, string>();
  const fingerprint = async (sourcePath: string, expected?: string) => {
    const path = sourceFile(context, sourcePath);
    const hash = bytesSha256(await readFile(path));
    if (expected !== undefined && (!/^[a-f0-9]{64}$/i.test(expected) || hash !== expected))
      throw new Error('Current source/dependency hash differs: ' + sourcePath);
    const prior = hashes.get(path);
    if (prior && prior !== hash) throw new Error('Input changed during preflight: ' + sourcePath);
    hashes.set(path, hash);
  };
  for (const entry of manifest.entries) {
    if (typeof entry.sourceSha256 !== 'string') throw new Error('Missing current source hash');
    await fingerprint(entry.sourcePath, entry.sourceSha256);
    for (const dependency of entry.dependencyFingerprints ?? [])
      await fingerprint(dependency.sourcePath, dependency.sha256);
    for (const reviewCase of entry.cases ?? []) {
      if (reviewCase.themeSourcePath) await fingerprint(reviewCase.themeSourcePath);
      const presentation = reviewCase.data['workbenchPresentation'] as
        { sourceFingerprint?: { sourcePath?: string; sha256?: string } } | undefined;
      if (presentation?.sourceFingerprint?.sourcePath)
        await fingerprint(presentation.sourceFingerprint.sourcePath, presentation.sourceFingerprint.sha256);
    }
  }
  for (const sourcePath of additionalSources) await fingerprint(sourcePath);
  const absentInputs = new Set<string>();
  if (includeLoaderInputs) {
    // Exact extra reads performed by LayoutDependencies.load; pin optional absence as well as bytes.
    const optionalInputs = new Set<string>([...Object.values(FIELD_ICON_PATHS),
      'zircon_runtime/src/dynamic_api/session/tests/runtime_ui_surface.rs']);
    for (const sourcePath of new Set([...manifest.entries.map((entry) => entry.sourcePath), ...additionalSources])) {
      const text = (await readFile(sourceFile(context, sourcePath))).toString('utf8');
      let document = parseToml(text) as {
        asset: { version?: number };
        nodes?: Record<string, { component?: unknown; props?: Record<string, unknown> }>;
      };
      // Match LayoutDependencies.load before inspecting inline legacy roots/children.
      if (document.asset.version === 1)
        document = parseZuiLayoutSource(text, sourcePath).document;
      for (const node of Object.values(document.nodes ?? {})) {
        const icon = node.props?.['icon'] ??
          (String(node.component).toLowerCase().includes('icon') ? node.props?.['source'] : undefined);
        if (typeof icon !== 'string' || !icon.endsWith('.svg')) continue;
        const locator = icon.replace(/^res:\/\/(icons\/)?/, '');
        const iconRoot = sourcePath.split('/assets/')[0] + '/assets/icons';
        const iconPath = layoutCurrentSourcePath(context, iconRoot + '/' + locator);
        if (!contains(layoutCurrentSourcePath(context, iconRoot), iconPath))
          throw new Error('Icon escapes current asset root');
        optionalInputs.add(relative(context.repoRoot, iconPath).replaceAll('\\', '/'));
      }
    }
    for (const sourcePath of optionalInputs) {
      const path = layoutCurrentSourcePath(context, sourcePath); noAliases(path);
      if (existsSync(path)) await fingerprint(sourcePath);
      else absentInputs.add(path);
    }
  }

  const snapshot = Object.freeze({ context, manifest: structuredClone(manifest), catalogSha256 });
  snapshots.set(snapshot, { hashes, catalogPath, absentInputs, manifestHash: bytesSha256(JSON.stringify(snapshot.manifest)) });
  await assertCurrentLayoutSnapshot(snapshot);
  return snapshot;
}
export async function assertCurrentLayoutSnapshot(snapshot: CurrentLayoutSnapshot): Promise<void> {
  const state = snapshots.get(snapshot);
  if (!state || bytesSha256(JSON.stringify(snapshot.manifest)) !== state.manifestHash)
    throw new Error('Unknown or mutated current layout snapshot');
  assertLayoutCurrentRoots(snapshot.context, snapshot.context.repoRoot, snapshot.context.outputRoot);
  noAliases(state.catalogPath);
  if (bytesSha256(await readFile(state.catalogPath)) !== snapshot.catalogSha256)
    throw new Error('Current catalog changed after preflight');
  for (const path of state.absentInputs) {
    noAliases(path);
    if (existsSync(path)) throw new Error('Optional current input appeared after preflight: ' + path);
  }
  for (const [path, expected] of state.hashes) {
    noAliases(path);
    if (bytesSha256(await readFile(path)) !== expected)
      throw new Error('Current input changed after preflight: ' + path);
  }
}

export interface SourceRefinementSpec {
  readonly id: 'material-source' | 'material-sample' | 'style-source-normalization' | 'workbench-composite';
  readonly select: (entry: CatalogEntry) => boolean;
  readonly requiredSources?: readonly string[];
  readonly allowSourceFilters?: boolean;
  readonly transform: (source: string, sourcePath: string) => {
    source: string; changes?: string[]; changed?: boolean; [key: string]: unknown;
  };
}
interface RefinementRecord {
  sourcePath: string; beforeSha256: string; afterSha256: string;
  changes?: string[]; baselineCases: number;
  state: 'planned' | 'backup-created' | 'write-started' | 'applied' | 'write-failed';
  observedSha256?: string;
  [key: string]: unknown;
}
export interface SourceRefinementReport {
  schema: 'dev.zircon.zui.source-refinement'; version: 1; tool: SourceRefinementSpec['id'];
  generatedAt: string; applied: boolean; applyRequested: boolean;
  status: 'planned' | 'proposed' | 'applied' | 'failed-before-source-write' | 'partial-source-write';
  changedFiles: number; changes: number; records: RefinementRecord[]; failure?: string;
  inputIdentity: { repoRoot: string; catalogRoot: string; reportRoot: string; catalogSha256: string };
}
export interface PreparedCurrentSourceRefinement { readonly report: SourceRefinementReport }
interface RefinementPlan {
  contexts: SourceRefinementContexts; snapshot: CurrentLayoutSnapshot;
  report: SourceRefinementReport; output: string; consumed: boolean;
  targets: Array<{ path: string; original: Buffer; updated: Buffer; backup: string; record: RefinementRecord }>;
  copies: Array<{ path: string; hash: string; bytes: Buffer; output: string }>;
}
const plans = new WeakMap<PreparedCurrentSourceRefinement, RefinementPlan>();

export async function prepareCurrentSourceRefinement(
  spec: SourceRefinementSpec, args: readonly string[] = [], supplied?: SourceRefinementContexts,
): Promise<PreparedCurrentSourceRefinement> {
  let apply = false, reportRoot: string | undefined;
  const filters: string[] = [];
  for (let i = 0; i < args.length; i++) {
    const flag = args[i]!;
    if (flag === '--apply' && !apply) apply = true;
    else if ((flag === '--report-root' && reportRoot === undefined) ||
             (flag === '--source' && spec.allowSourceFilters)) {
      const value = args[++i];
      if (!value || value.startsWith('--')) throw new Error('Missing value for ' + flag);
      if (flag === '--source') filters.push(value); else reportRoot = value;
    } else throw new Error('Unknown or duplicate refinement flag: ' + flag);
  }
  if (supplied && reportRoot !== undefined) throw new Error('Supplied contexts cannot be overridden');
  if (!supplied) {
    for (const key of ['ZUI_LAYOUT_REPO_ROOT', 'ZUI_LAYOUT_OUTPUT_ROOT']) {
      const value = process.env[key];
      if (value !== undefined && (!value.trim() || !isAbsolute(value)))
        throw new Error('Refinement requires absolute ' + key);
    }
    const catalog = createLayoutCurrentContext();
    reportRoot ??= process.env['ZUI_LAYOUT_SOURCE_REPORT_ROOT'];
    if (!reportRoot?.trim() || !isAbsolute(reportRoot))
      throw new Error('Refinement requires explicit absolute source report root');
    supplied = { catalog, report: createLayoutCurrentContext({ repoRoot: catalog.repoRoot, outputRoot: reportRoot }) };
  }
  const contexts = Object.freeze({ catalog: supplied.catalog, report: supplied.report });
  assertSeparateCurrentReport(contexts);
  const snapshot = await prepareCurrentLayoutSnapshot(contexts.catalog);
  const output = layoutCurrentOutputPath(contexts.report,
    'evidence/' + spec.id + '-' + (apply ? 'applied' : 'proposed') + '.json');
  absent(output);
  const targets: RefinementPlan['targets'] = [], copies: RefinementPlan['copies'] = [];
  const destinations = new Set<string>([normalize(output)]);
  const reserve = (path: string) => {
    absent(path);
    if (destinations.has(normalize(path))) throw new Error('Duplicate refinement output');
    destinations.add(normalize(path));
  };
  for (const required of spec.requiredSources ?? [])
    if (!snapshot.manifest.entries.some((entry) => entry.sourcePath === required))
      throw new Error('Missing required refinement source: ' + required);
  for (const entry of snapshot.manifest.entries) {
    if (!spec.select(entry) || (filters.length && !filters.some((filter) => entry.sourcePath.includes(filter)))) continue;
    const path = sourceFile(contexts.catalog, entry.sourcePath);
    const original = await readFile(path);
    if (bytesSha256(original) !== entry.sourceSha256) throw new Error('Stale refinement source');
    const result = spec.transform(original.toString('utf8'), entry.sourcePath);
    const updated = Buffer.from(result.source);
    if (updated.equals(original)) continue;
    const { source: _source, ...details } = result;
    const record: RefinementRecord = { ...details, sourcePath: entry.sourcePath,
      beforeSha256: bytesSha256(original), afterSha256: bytesSha256(updated),
      baselineCases: entry.penpotEvidence?.length ?? 0, state: 'planned' };
    const backup = layoutCurrentOutputPath(contexts.report, 'backups/' + spec.id + '/' + entry.sourcePath);
    reserve(backup);
    record.backupPath = relative(contexts.report.outputRoot, backup).replaceAll('\\', '/');
    targets.push({ path, original, updated, backup, record });
    for (const evidence of entry.penpotEvidence ?? []) {
      if (!evidence.screenshotPath) continue;
      if (!/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(evidence.caseId))
        throw new Error('Invalid screenshot case identity');
      const input = layoutCurrentOutputPath(contexts.catalog, evidence.screenshotPath);
      noAliases(input);
      if (!lstatSync(input).isFile()) throw new Error('Screenshot is not a regular file');
      const bytes = await readFile(input);
      if (evidence.screenshotSha256 && bytesSha256(bytes) !== evidence.screenshotSha256)
        throw new Error('Screenshot hash differs from catalog');
      const destination = layoutCurrentOutputPath(contexts.report,
        'backups/' + spec.id + '/' + entry.sourcePath + '.evidence/' + evidence.caseId + '.png');
      reserve(destination);
      copies.push({ path: input, hash: bytesSha256(bytes), bytes, output: destination });
    }
  }
  await assertCurrentLayoutSnapshot(snapshot);
  const report: SourceRefinementReport = {
    schema: 'dev.zircon.zui.source-refinement', version: 1, tool: spec.id,
    generatedAt: new Date().toISOString(), applied: false, applyRequested: apply, status: 'planned',
    changedFiles: targets.length, changes: targets.reduce((sum, item) => sum + (item.record.changes?.length ?? 0), 0),
    records: targets.map((target) => target.record),
    inputIdentity: { repoRoot: realpathSync.native(contexts.catalog.repoRoot),
      catalogRoot: realpathSync.native(contexts.catalog.outputRoot),
      reportRoot: realpathSync.native(contexts.report.outputRoot), catalogSha256: snapshot.catalogSha256 },
  };
  JSON.stringify(report); // Reject nonserializable metadata before locks, backups or source writes.
  const prepared = Object.freeze({ report: structuredClone(report) });
  plans.set(prepared, { contexts, snapshot, report, output, targets, copies, consumed: false });
  return prepared;
}

export async function commitCurrentSourceRefinement(
  prepared: PreparedCurrentSourceRefinement,
): Promise<SourceRefinementReport> {
  const plan = plans.get(prepared);
  if (!plan || plan.consumed) throw new Error('Unknown or consumed refinement plan');
  const { contexts, snapshot, report } = plan;
  const verify = async (backupsAbsent = true) => {
    assertSeparateCurrentReport(contexts);
    await assertCurrentLayoutSnapshot(snapshot);
    absent(plan.output);
    for (const copy of plan.copies) {
      noAliases(copy.path);
      if (bytesSha256(await readFile(copy.path)) !== copy.hash) throw new Error('Screenshot changed after preflight');
      if (backupsAbsent) absent(copy.output);
    }
    if (backupsAbsent) for (const target of plan.targets) absent(target.backup);
  };
  await verify();
  plan.consumed = true;
  // Only the SAME explicit report/coordination root is serialized; there is no cross-root mutex.
  return withLayoutCatalogLock(contexts.report.outputRoot,
    { command: 'source-refinement:' + report.tool, context: contexts.report }, async () => {
      const handles: Array<{ target: RefinementPlan['targets'][number]; handle: FileHandle; dev: number; ino: number }> = [];
      try {
        await verify();
        if (report.applyRequested) {
          for (const target of plan.targets) {
            const handle = await open(target.path, 'r+');
            const stat = await handle.stat();
            handles.push({ target, handle, dev: stat.dev, ino: stat.ino });
            const current = lstatSync(target.path);
            if (!stat.isFile() || current.dev !== stat.dev || current.ino !== stat.ino ||
                bytesSha256(await handle.readFile()) !== target.record.beforeSha256)
              throw new Error('Refinement source handle CAS conflict');
          }
          await verify();
          for (const { target } of handles) {
            absent(target.backup);
            await mkdir(dirname(target.backup), { recursive: true });
            absent(target.backup);
            await writeFile(target.backup, target.original, { flag: 'wx' });
            target.record.state = 'backup-created';
          }
          for (const copy of plan.copies) {
            absent(copy.output);
            await mkdir(dirname(copy.output), { recursive: true });
            absent(copy.output);
            await writeFile(copy.output, copy.bytes, { flag: 'wx' });
          }
          for (const { target, handle, dev, ino } of handles) {
            await verify(false);
            noAliases(target.path);
            const current = lstatSync(target.path);
            if (current.dev !== dev || current.ino !== ino ||
                bytesSha256(await readFile(target.path)) !== target.record.beforeSha256)
              throw new Error('Refinement source identity/CAS conflict before write');
            target.record.state = 'write-started';
            try {
              let offset = 0;
              while (offset < target.updated.length) {
                const { bytesWritten } = await handle.write(target.updated, offset, target.updated.length - offset, offset);
                if (!bytesWritten) throw new Error('Refinement write made no progress');
                offset += bytesWritten;
              }
              await handle.truncate(target.updated.length); await handle.sync();
              target.record.observedSha256 = bytesSha256(await readFile(target.path));
              if (target.record.observedSha256 !== target.record.afterSha256)
                throw new Error('Refinement source changed during write');
              snapshots.get(snapshot)!.hashes.set(target.path, target.record.afterSha256);
              target.record.state = 'applied';
            } catch (error) {
              target.record.state = 'write-failed';
              try { target.record.observedSha256 = bytesSha256(await readFile(target.path)); } catch { /* not success */ }
              throw error;
            }
          }
        }
        await verify(false);
        report.applied = report.applyRequested;
        report.status = report.applyRequested ? 'applied' : 'proposed';
      } catch (error) {
        report.applied = false;
        report.status = report.records.some((record) =>
          ['write-started', 'write-failed', 'applied'].includes(record.state))
          ? 'partial-source-write' : 'failed-before-source-write';
        report.failure = error instanceof Error ? error.message : String(error);
        let reportWriteFailure: string | undefined;
        try {
          assertSeparateCurrentReport(contexts); absent(plan.output);
          await mkdir(dirname(plan.output), { recursive: true }); absent(plan.output);
          await writeFile(plan.output, JSON.stringify(report, null, 2) + '\n', { flag: 'wx' });
        } catch (writeError) { reportWriteFailure = String(writeError); }
        throw Object.assign(new Error('Source refinement failed: ' + report.failure, { cause: error }),
          { report: structuredClone(report), reportWriteFailure });
      } finally { await Promise.all(handles.map(({ handle }) => handle.close())); }
      try {
        assertSeparateCurrentReport(contexts); absent(plan.output);
        await mkdir(dirname(plan.output), { recursive: true }); absent(plan.output);
        await writeFile(plan.output, JSON.stringify(report, null, 2) + '\n', { flag: 'wx' });
      } catch (error) {
        report.applied = false;
        report.status = report.records.some((record) => record.state === 'applied')
          ? 'partial-source-write' : 'failed-before-source-write';
        report.failure = String(error);
        throw Object.assign(new Error('Refinement report write failed', { cause: error }), { report: structuredClone(report) });
      }
      return structuredClone(report);
    });
}
export async function runCurrentSourceRefinement(
  spec: SourceRefinementSpec, args: readonly string[] = [], contexts?: SourceRefinementContexts,
): Promise<SourceRefinementReport> {
  return commitCurrentSourceRefinement(await prepareCurrentSourceRefinement(spec, args, contexts));
}

/** Optional, explicitly rooted report input. No fallback into catalog/evidence or docs/_data. */
export async function readCurrentSourceReport<T extends { sourcePath: string }>(
  catalog: LayoutCurrentContext, name: string, root = process.env['ZUI_LAYOUT_SOURCE_REPORT_ROOT'],
): Promise<Map<string, T>> {
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*\.json$/.test(name)) throw new Error('Invalid source report filename');
  if (!root) return new Map();
  if (!isAbsolute(root)) throw new Error('Source report input root must be absolute');
  const reportContext = createLayoutCurrentContext({ repoRoot: catalog.repoRoot, outputRoot: root });
  assertSeparateCurrentReport({ catalog, report: reportContext });
  const path = layoutCurrentOutputPath(reportContext, 'evidence/' + name);
  noAliases(path);
  if (!existsSync(path)) return new Map();
  const bytes = await readFile(path);
  const value = JSON.parse(bytes.toString('utf8')) as Partial<SourceRefinementReport>;
  if (typeof value.applied !== 'boolean' || !Array.isArray(value.records))
    throw new Error('Invalid explicit source report');
  if (!value.applied) return new Map();
  if (value.status !== undefined && value.status !== 'applied')
    throw new Error('Applied report has nonterminal status');
  if (value.inputIdentity &&
      (normalize(value.inputIdentity.repoRoot) !== normalize(realpathSync.native(catalog.repoRoot)) ||
       normalize(value.inputIdentity.reportRoot) !== normalize(realpathSync.native(root))))
    throw new Error('Source report belongs to different current roots');
  const records = new Map<string, T>();
  for (const record of value.records) {
    if (typeof record.sourcePath !== 'string' || !/^[a-f0-9]{64}$/i.test(record.beforeSha256) ||
        !/^[a-f0-9]{64}$/i.test(record.afterSha256) ||
        (record.state !== undefined && record.state !== 'applied') || records.has(record.sourcePath))
      throw new Error('Invalid applied source report record');
    sourceFile(catalog, record.sourcePath);
    if (record.changes !== undefined && (!Array.isArray(record.changes) ||
        (record.changes as unknown[]).some((item) => name === 'table-source-cells-applied.json'
          ? !item || typeof item !== 'object' || typeof (item as { nodeId?: unknown }).nodeId !== 'string' ||
            !Array.isArray((item as { options?: unknown }).options) ||
            (item as { options: unknown[] }).options.some((option) => typeof option !== 'string')
          : typeof item !== 'string'))) throw new Error('Invalid source report changes');
    // Preserve all historical diagnostic/geometry details without synthesizing evidence.
    if (bytesSha256(await readFile(sourceFile(catalog, record.sourcePath))) !== record.afterSha256) continue;
    records.set(record.sourcePath, { ...record } as unknown as T);
  }
  if (bytesSha256(await readFile(path)) !== bytesSha256(bytes)) throw new Error('Source report changed while reading');
  return records;
}

export function assertCurrentCaptureDestination(context: LayoutCurrentContext, path: string): void {
  assertLayoutCurrentRoots(context, context.repoRoot, context.outputRoot);
  absent(layoutCurrentOutputPath(context, path));
}
export async function writeCurrentCaptureFile(
  context: LayoutCurrentContext, path: string, data: string | Uint8Array, encoding?: BufferEncoding,
): Promise<void> {
  const output = layoutCurrentOutputPath(context, path);
  assertCurrentCaptureDestination(context, output);
  await mkdir(dirname(output), { recursive: true });
  assertCurrentCaptureDestination(context, output);
  await writeFile(output, data, { flag: 'wx', ...(encoding ? { encoding } : {}) });
}
export async function captureCurrentScreenshot(
  page: import('playwright').Page, context: LayoutCurrentContext, path: string,
  options: { fullPage?: boolean; timeout?: number } = {},
): Promise<void> {
  assertCurrentCaptureDestination(context, path);
  const bytes = await page.screenshot({ ...options, type: 'png' });
  await writeCurrentCaptureFile(context, path, bytes);
}
