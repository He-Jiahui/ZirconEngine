import assert from 'node:assert/strict';
import { existsSync, lstatSync, realpathSync } from 'node:fs';
import { readFile, writeFile, mkdir, open, type FileHandle } from 'node:fs/promises';
import { dirname, isAbsolute, relative, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { parseZuiDocument, type ZuiDocument } from '../src/bridge/zui-document';
import { tableCells } from '../src/bridge/zui-table-cells';
import { bytesSha256 } from './zui-layout-evidence';
import { readCurrentLayoutOutputCatalog } from './zui-layout-gallery';
import { withLayoutCatalogLock } from './zui-layout-catalog-lock';
import { assertLayoutCurrentRoots, createLayoutCurrentContext, layoutCurrentOutputPath, layoutCurrentSourcePath, type LayoutCurrentContext } from './zui-layout-paths';

export function explicitTableCells(source: string): {
  source: string;
  changes: Array<{ nodeId: string; options: string[] }>;
  unresolved: Array<{ nodeId: string; reason: string }>;
} {
  const document = parseZuiDocument(source).document;
  const expected: ZuiDocument = structuredClone(document);
  const newline = source.includes('\r\n') ? '\r\n' : '\n';
  const lines = source.split(/\r?\n/);
  const changes: Array<{ nodeId: string; options: string[] }> = [];
  const unresolved: Array<{ nodeId: string; reason: string }> = [];
  for (const [nodeId, node] of Object.entries(document.nodes ?? {})) {
    if (node.component !== 'WorkbenchTableRow' || node.props?.['options'] !== undefined) continue;
    const props = node.props ?? {};
    const text = props['text'];
    const detail = props['value_text'];
    if (typeof text !== 'string' || !text.trim()) {
      unresolved.push({ nodeId, reason: 'No authored table label' });
      continue;
    }
    if (detail !== undefined && typeof detail !== 'string') {
      unresolved.push({ nodeId, reason: 'Non-string detail requires a model adapter' });
      continue;
    }
    const options = typeof detail === 'string' && detail.trim()
      ? [text.trim(), ...detail.trim().split(/ {2,}|\t+/)]
      : tableCells(props).map((cell) => cell.value);
    if (options.length > 4 || options.some((value) => !value || /[\r\n]/.test(value))) {
      unresolved.push({ nodeId, reason: `Authored content requires ${options.length} columns or multiline cells` });
      continue;
    }
    // The native archived-row heuristic must still recognize these as cells.
    if (tableCells({ options }).length !== options.length) {
      unresolved.push({ nodeId, reason: 'Native archived-row heuristic rejects declared cell values' });
      continue;
    }
    const start = lines.findIndex((line) => line.trim() === `[nodes.${nodeId}]`);
    const next = lines.findIndex((line, index) => index > start && /^\s*\[/.test(line));
    const end = next < 0 ? lines.length : next;
    const lineIndex = lines.findIndex((line, index) => index > start && index < end && /^\s*props\s*=\s*\{.*\}\s*$/.test(line));
    if (start < 0 || lineIndex < 0) {
      unresolved.push({ nodeId, reason: 'Props are not a single inline table; edit manually' });
      continue;
    }
    const line = lines[lineIndex];
    const close = line.lastIndexOf('}');
    lines[lineIndex] = `${line.slice(0, close).trimEnd()}, options = ${JSON.stringify(options)} ${line.slice(close)}`;
    expected.nodes![nodeId].props!['options'] = options;
    changes.push({ nodeId, options });
  }
  const updated = lines.join(newline);
  assert.deepEqual(parseZuiDocument(updated).document, expected, 'Table migration must only add explicit options');
  return { source: updated, changes, unresolved };
}

export interface TableSourceContexts {
  readonly catalog: LayoutCurrentContext;
  readonly report: LayoutCurrentContext;
}
interface RecordState {
  sourcePath: string;
  beforePath: string;
  beforeSha256: string;
  afterSha256: string;
  changes: Array<{ nodeId: string; options: string[] }>;
  unresolved: Array<{ nodeId: string; reason: string }>;
  state: 'planned' | 'backup-created' | 'write-started' | 'applied' | 'write-failed';
  observedSha256?: string;
}
export interface TableSourceReport {
  schema: 'dev.zircon.zui.table-source-cells';
  version: 1;
  generatedAt: string;
  inputIdentity: { repoRoot: string; catalogRoot: string; reportRoot: string; catalogSha256: string };
  applyRequested: boolean;
  applied: boolean;
  status: 'proposed' | 'applied' | 'failed-before-source-write' | 'partial-source-write';
  changedFiles: number;
  changedRows: number;
  unresolvedRows: number;
  records: RecordState[];
  failure?: string;
}
export interface PreparedTableSourceCells { readonly report: TableSourceReport }
interface Plan {
  contexts: TableSourceContexts;
  report: TableSourceReport;
  output: string;
  hashes: Map<string, string>;
  targets: Array<{ path: string; backup: string; original: Buffer; updated: Buffer; record: RecordState }>;
  consumed: boolean;
}
const plans = new WeakMap<PreparedTableSourceCells, Plan>();
const normalize = (path: string): string => process.platform === 'win32' ? path.toLowerCase() : path;
const contains = (root: string, path: string): boolean => {
  const suffix = relative(root, path);
  return !isAbsolute(suffix) && suffix !== '..' && !/^\.\.[/\\]/.test(suffix);
};
function noAliases(path: string): void {
  let ancestor = resolve(path);
  while (!existsSync(ancestor)) {
    // lstat also rejects dangling links rather than treating them as missing directories.
    try { lstatSync(ancestor); throw new Error(`Unresolved table path alias: ${ancestor}`); }
    catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error; }
    const parent = dirname(ancestor);
    if (parent === ancestor) throw new Error('Table path has no physical parent');
    ancestor = parent;
  }
  if (normalize(realpathSync.native(ancestor)) !== normalize(ancestor))
    throw new Error(`Table path alias is not admitted: ${path}`);
  for (let current = ancestor; ; current = dirname(current)) {
    if (lstatSync(current).isSymbolicLink()) throw new Error(`Table path alias is not admitted: ${path}`);
    if (dirname(current) === current) break;
  }
}
function validateContexts(contexts: TableSourceContexts): void {
  for (const context of [contexts.catalog, contexts.report]) {
    assertLayoutCurrentRoots(context, context.repoRoot, context.outputRoot);
    noAliases(context.repoRoot); noAliases(context.outputRoot);
  }
  assertLayoutCurrentRoots(contexts.catalog, contexts.report.repoRoot, contexts.catalog.outputRoot);
  const input = realpathSync.native(contexts.catalog.outputRoot);
  const output = realpathSync.native(contexts.report.outputRoot);
  if (contains(input, output) || contains(output, input))
    throw new Error('Table report root must be disjoint from readonly catalog input');
  noAliases(layoutCurrentOutputPath(contexts.report, '.zui-layout-catalog.lock'));
  noAliases(layoutCurrentOutputPath(contexts.report, '.zui-layout-catalog.lock/owner.json'));
}
function absentOutput(path: string): void {
  noAliases(path);
  if (existsSync(path)) throw new Error(`Refusing existing table output: ${path}`);
  let parent = dirname(path);
  while (!existsSync(parent)) parent = dirname(parent);
  if (!lstatSync(parent).isDirectory()) throw new Error(`Table output parent is not a directory: ${parent}`);
}
function sourceFile(context: LayoutCurrentContext, sourcePath: string): string {
  const path = layoutCurrentSourcePath(context, sourcePath);
  noAliases(path);
  if (!lstatSync(path).isFile()) throw new Error(`Table source is not a regular file: ${sourcePath}`);
  return path;
}
function identity(contexts: TableSourceContexts, catalogSha256: string): TableSourceReport['inputIdentity'] {
  return { repoRoot: realpathSync.native(contexts.catalog.repoRoot),
    catalogRoot: realpathSync.native(contexts.catalog.outputRoot),
    reportRoot: realpathSync.native(contexts.report.outputRoot), catalogSha256 };
}

/** Plan every input and destination before lock acquisition or any writes. No legacy root overload. */
export async function prepareCurrentTableSourceCells(
  args: readonly string[] = [], supplied?: TableSourceContexts,
): Promise<PreparedTableSourceCells> {
  let apply = false;
  let reportRoot: string | undefined;
  const seen = new Set<string>();
  for (let index = 0; index < args.length; index++) {
    const flag = args[index]!;
    if (seen.has(flag)) throw new Error(`Duplicate table flag: ${flag}`);
    seen.add(flag);
    if (flag === '--apply') apply = true;
    else if (flag === '--report-root') {
      reportRoot = args[++index];
      if (!reportRoot || reportRoot.startsWith('--')) throw new Error('Missing --report-root value');
      if (!isAbsolute(reportRoot)) throw new Error('Table report root must be absolute');
    } else throw new Error(`Unknown table flag: ${flag}`);
  }
  if (supplied && reportRoot !== undefined) throw new Error('Supplied table contexts cannot be overridden');
  if (!supplied) {
    for (const key of ['ZUI_LAYOUT_REPO_ROOT', 'ZUI_LAYOUT_OUTPUT_ROOT']) {
      const value = process.env[key];
      if (value !== undefined && (!value.trim() || !isAbsolute(value)))
        throw new Error(`Table tool requires absolute ${key}`);
    }
    const catalog = createLayoutCurrentContext();
    reportRoot ??= process.env['ZUI_LAYOUT_TABLE_OUTPUT_ROOT'];
    if (!reportRoot?.trim() || !isAbsolute(reportRoot)) throw new Error('Table tool requires explicit absolute report root');
    supplied = { catalog, report: createLayoutCurrentContext({ repoRoot: catalog.repoRoot, outputRoot: reportRoot }) };
  }
  const contexts = Object.freeze({ catalog: supplied.catalog, report: supplied.report });
  validateContexts(contexts);
  const { manifest, catalogSha256 } = await readCurrentLayoutOutputCatalog(contexts.catalog);
  noAliases(layoutCurrentOutputPath(contexts.catalog, 'catalog.json'));
  const output = layoutCurrentOutputPath(contexts.report, `evidence/table-source-cells-${apply ? 'applied' : 'proposed'}.json`);
  absentOutput(output);
  // Resolve even unused catalog output fields before acquiring a report lock.
  const hashes = new Map<string, string>();
  const targets: Plan['targets'] = [];
  const records: RecordState[] = [];
  const physicalSources = new Set<string>();
  const fingerprint = async (sourcePath: string, expected: string, kind: string): Promise<Buffer> => {
    const path = sourceFile(contexts.catalog, sourcePath);
    const bytes = await readFile(path);
    if (bytesSha256(bytes) !== expected) throw new Error(`${kind} changed after catalog generation: ${sourcePath}`);
    if (hashes.has(path) && hashes.get(path) !== expected) throw new Error('Conflicting table source fingerprints');
    hashes.set(path, expected);
    return bytes;
  };
  for (const entry of manifest.entries) {
    const original = await fingerprint(entry.sourcePath, entry.sourceSha256, 'Source');
    const path = sourceFile(contexts.catalog, entry.sourcePath);
    const physical = normalize(realpathSync.native(path));
    if (physicalSources.has(physical)) throw new Error('Duplicate physical table source');
    physicalSources.add(physical);
    for (const dependency of entry.dependencyFingerprints ?? [])
      await fingerprint(dependency.sourcePath, dependency.sha256, 'Dependency');
    if (!relative(contexts.catalog.repoRoot, path).replaceAll('\\', '/').startsWith('zircon_editor/assets/')) continue;
    const result = explicitTableCells(original.toString('utf8'));
    if (!result.changes.length && !result.unresolved.length) continue;
    const beforePath = `evidence/before-table-cells/${entry.sourcePath}`;
    const backup = layoutCurrentOutputPath(contexts.report, beforePath);
    if (apply && result.changes.length) absentOutput(backup);
    const record: RecordState = { sourcePath: entry.sourcePath, beforePath,
      beforeSha256: bytesSha256(original), afterSha256: bytesSha256(result.source),
      changes: result.changes, unresolved: result.unresolved, state: 'planned' };
    records.push(record);
    if (apply && result.changes.length)
      targets.push({ path, backup, original, updated: Buffer.from(result.source), record });
  }
  const report: TableSourceReport = { schema: 'dev.zircon.zui.table-source-cells', version: 1,
    generatedAt: new Date().toISOString(), inputIdentity: identity(contexts, catalogSha256),
    applyRequested: apply, applied: false, status: 'proposed',
    changedFiles: records.filter((record) => record.changes.length).length,
    changedRows: records.reduce((sum, record) => sum + record.changes.length, 0),
    unresolvedRows: records.reduce((sum, record) => sum + record.unresolved.length, 0), records };
  const prepared = Object.freeze({ report: structuredClone(report) });
  plans.set(prepared, { contexts, report, output, hashes, targets, consumed: false });
  return prepared;
}

/** Commit a genuine prepared plan; source CAS and the report lock do not promise a multi-file transaction. */
export async function commitCurrentTableSourceCells(prepared: PreparedTableSourceCells): Promise<TableSourceReport> {
  const plan = plans.get(prepared);
  if (!plan || plan.consumed) throw new Error('Unknown or already consumed table plan');
  const { contexts, report } = plan;
  const verify = async (backupsAbsent = true): Promise<void> => {
    validateContexts(contexts);
    const current = await readCurrentLayoutOutputCatalog(contexts.catalog);
    noAliases(layoutCurrentOutputPath(contexts.catalog, 'catalog.json'));
    if (current.catalogSha256 !== report.inputIdentity.catalogSha256)
      throw new Error('Table catalog changed after preflight');
    for (const [path, expected] of plan.hashes) {
      noAliases(path);
      if (bytesSha256(await readFile(path)) !== expected) throw new Error(`Table source/dependency CAS conflict: ${path}`);
    }
    absentOutput(plan.output);
    if (backupsAbsent) for (const target of plan.targets) absentOutput(target.backup);
  };
  await verify(); // No output directories or lock exist when an input/destination is invalid.
  plan.consumed = true;
  return withLayoutCatalogLock(contexts.report.outputRoot, { command: 'zui-table-source-cells', context: contexts.report }, async () => {
    await verify(); // Fresh catalog/inputs/physical roles after acquiring the actual lock.
    const handles: Array<{ target: Plan['targets'][number]; handle: FileHandle; dev: number; ino: number }> = [];
    try {
      // Open and compare all existing source identities before backups or source writes.
      for (const target of plan.targets) {
        const handle = await open(target.path, 'r+');
        const stat = await handle.stat();
        handles.push({ target, handle, dev: stat.dev, ino: stat.ino });
        const pathStat = lstatSync(target.path);
        if (!stat.isFile() || pathStat.dev !== stat.dev || pathStat.ino !== stat.ino ||
            bytesSha256(await handle.readFile()) !== target.record.beforeSha256)
          throw new Error(`Table source handle CAS conflict: ${target.path}`);
      }
      await verify();
      await mkdir(dirname(plan.output), { recursive: true });
      for (const { target } of handles) {
        absentOutput(target.backup);
        await mkdir(dirname(target.backup), { recursive: true });
        absentOutput(target.backup);
        await writeFile(target.backup, target.original, { flag: 'wx' });
        target.record.state = 'backup-created';
      }
      for (const { target, handle, dev, ino } of handles) {
        validateContexts(contexts);
        const currentCatalog = await readCurrentLayoutOutputCatalog(contexts.catalog);
        if (currentCatalog.catalogSha256 !== report.inputIdentity.catalogSha256) throw new Error('Table catalog changed before source write');
        for (const [path, expected] of plan.hashes) {
          noAliases(path);
          if (bytesSha256(await readFile(path)) !== expected) throw new Error(`Table source/dependency CAS conflict: ${path}`);
        }
        noAliases(target.path);
        const current = lstatSync(target.path);
        if (current.dev !== dev || current.ino !== ino) throw new Error('Table source identity replaced before write');
        target.record.state = 'write-started';
        try {
          let offset = 0;
          while (offset < target.updated.length) {
            const { bytesWritten } = await handle.write(target.updated, offset, target.updated.length - offset, offset);
            if (!bytesWritten) throw new Error('Table source write made no progress');
            offset += bytesWritten;
          }
          await handle.truncate(target.updated.length);
          await handle.sync();
          target.record.observedSha256 = bytesSha256(await readFile(target.path));
          if (target.record.observedSha256 !== target.record.afterSha256) throw new Error('Table source changed during write');
          target.record.state = 'applied';
          plan.hashes.set(target.path, target.record.afterSha256);
        } catch (error) {
          target.record.state = 'write-failed';
          try { target.record.observedSha256 = bytesSha256(await readFile(target.path)); } catch { /* unavailable is not success */ }
          throw error;
        }
      }
      await verify(false); // Recheck all input identities, accepting only our recorded source changes.
      report.applied = report.applyRequested;
      report.status = report.applyRequested ? 'applied' : 'proposed';
    } catch (error) {
      report.applied = false;
      report.status = report.records.some((record) => ['write-started', 'write-failed', 'applied'].includes(record.state))
        ? 'partial-source-write' : 'failed-before-source-write';
      report.failure = error instanceof Error ? error.message : String(error);
      let reportWriteFailure: string | undefined;
      try {
        validateContexts(contexts);
        absentOutput(plan.output);
        await mkdir(dirname(plan.output), { recursive: true });
        absentOutput(plan.output);
        await writeFile(plan.output, `${JSON.stringify(report, null, 2)}\n`, { flag: 'wx' });
      } catch (writeError) {
        reportWriteFailure = writeError instanceof Error ? writeError.message : String(writeError);
      }
      throw Object.assign(new Error(`Table source apply failed: ${report.failure}`, { cause: error }),
        { report: structuredClone(report), reportWriteFailure });
    } finally {
      await Promise.all(handles.map(({ handle }) => handle.close()));
    }
    try {
      validateContexts(contexts);
      absentOutput(plan.output);
      await writeFile(plan.output, `${JSON.stringify(report, null, 2)}\n`, { flag: 'wx' });
    } catch (error) {
      report.applied = false;
      report.status = report.records.some((record) => record.state === 'applied')
        ? 'partial-source-write' : 'failed-before-source-write';
      report.failure = error instanceof Error ? error.message : String(error);
      throw Object.assign(new Error('Table report write failed after source processing', { cause: error }),
        { report: structuredClone(report), reportWriteFailure: report.failure });
    }
    return structuredClone(report);
  });
}

export async function runCurrentTableSourceCells(args: readonly string[] = [], contexts?: TableSourceContexts): Promise<TableSourceReport> {
  return commitCurrentTableSourceCells(await prepareCurrentTableSourceCells(args, contexts));
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  void runCurrentTableSourceCells(process.argv.slice(2)).then((report) => {
    console.log(JSON.stringify({ ...report, records: undefined }));
  }).catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
}
