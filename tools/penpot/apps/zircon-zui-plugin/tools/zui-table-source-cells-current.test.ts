import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { mkdtemp, mkdir, readFile, readdir, readlink, rm, rmdir, symlink, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { promisify } from 'node:util';
import test from 'node:test';
import type { CatalogEntry, CatalogManifest } from './zui-layout-catalog';
import { parseZuiDocument } from '../src/bridge/zui-document';
import { bytesSha256 } from './zui-layout-evidence';
import { createLayoutCurrentContext, type LayoutCurrentContext } from './zui-layout-paths';
import { requireApprovedPenpotArtifactDirectory } from './zui-layout-artifact-path';
import { acquireLayoutCatalogLock } from './zui-layout-catalog-lock';
import { commitCurrentTableSourceCells, explicitTableCells, prepareCurrentTableSourceCells, runCurrentTableSourceCells, type PreparedTableSourceCells } from './zui-table-source-cells';

const execute = promisify(execFile);
const source = `[asset]
kind = "component"
version = 2
id = "res://review.zui"
[components.Review]
root = "row"
[nodes.row]
component = "WorkbenchTableRow"
props = { text = "Renderer.Smoke", value_text = "Running   62%   Worker_03", custom = "preserve" }
events = [{ id = "Run", event = "Click", route = "automation.run" }]
`;
const snapshot = async (root: string): Promise<Record<string, string>> => {
  const result: Record<string, string> = {};
  const visit = async (path: string, prefix = ''): Promise<void> => {
    for (const item of await readdir(path, { withFileTypes: true })) {
      const key = prefix + item.name;
      const child = resolve(path, item.name);
      if (item.isSymbolicLink()) result[key] = 'link:' + await readlink(child);
      else if (item.isDirectory()) { result[key + '/'] = 'directory'; await visit(child, key + '/'); }
      else result[key] = bytesSha256(await readFile(child));
    }
  };
  await visit(root);
  return result;
};

test('table source apply separates readonly current catalog and isolated reports', async (t) => {
  const artifactRoot = process.env['ZUI_LAYOUT_TEST_ARTIFACT_ROOT'];
  assert.ok(artifactRoot, 'Root must register ZUI_LAYOUT_TEST_ARTIFACT_ROOT');
  requireApprovedPenpotArtifactDirectory(artifactRoot, 'Table source test fixture');
  const fixture = await mkdtemp(resolve(artifactRoot, 'layout-table-current-'));
  const actualRepo = resolve(dirname(fileURLToPath(import.meta.url)), '../../../../..');
  const envKeys = ['ZUI_LAYOUT_REPO_ROOT', 'ZUI_LAYOUT_OUTPUT_ROOT', 'ZUI_LAYOUT_TABLE_OUTPUT_ROOT', 'ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'] as const;
  const previous = Object.fromEntries(envKeys.map((key) => [key, process.env[key]]));
  for (const key of envKeys) delete process.env[key];
  const makeFixture = async () => {
    const root = await mkdtemp(resolve(fixture, 'case-'));
    const repoRoot = resolve(root, 'repo');
    const catalogRoot = resolve(root, 'catalog');
    const reportRoot = resolve(root, 'report');
    const outside = resolve(root, 'outside');
    for (const path of [resolve(repoRoot, 'zircon_runtime/src'), resolve(repoRoot, 'zircon_editor/assets/ui'), catalogRoot, reportRoot, outside])
      await mkdir(path, { recursive: true });
    // Synthetic parser inputs and repository markers, never a checkout/source-tree copy.
    await writeFile(resolve(repoRoot, 'Cargo.toml'), '# synthetic repository marker\n');
    await writeFile(resolve(repoRoot, 'zircon_runtime/src/lib.rs'), '// synthetic repository marker\n');
    const sourcePath = 'zircon_editor/assets/ui/table.zui';
    const dependencyPath = 'zircon_editor/assets/ui/dependency.zui';
    await writeFile(resolve(repoRoot, sourcePath), source.replaceAll('\n', '\r\n'));
    await writeFile(resolve(repoRoot, dependencyPath), '# synthetic dependency\n');
    const sourceBytes = await readFile(resolve(repoRoot, sourcePath));
    const dependencyBytes = await readFile(resolve(repoRoot, dependencyPath));
    const entry = { sourcePath, sourceSha256: bytesSha256(sourceBytes), sourceClassification: 'test-fixture',
      outputPath: 'table.zui', resultPath: 'result.md', previewPath: 'preview.png',
      dependencyFingerprints: [{ sourcePath: dependencyPath, sha256: bytesSha256(dependencyBytes) }],
      cases: [], status: 'prepared', visualStatus: 'pending' } as CatalogEntry;
    const manifest = { schema: 'dev.zircon.zui.layout-catalog', version: 1, generatedAt: 'fixture-not-capture',
      repoRoot, sourceCount: 1, entries: [entry] } as CatalogManifest;
    const catalogPath = resolve(catalogRoot, 'catalog.json');
    const save = (value: unknown = manifest) => writeFile(catalogPath, JSON.stringify(value));
    await save();
    const contexts = { catalog: createLayoutCurrentContext({ repoRoot, outputRoot: catalogRoot }),
      report: createLayoutCurrentContext({ repoRoot, outputRoot: reportRoot }) };
    const output = (apply = false) => resolve(reportRoot, `evidence/table-source-cells-${apply ? 'applied' : 'proposed'}.json`);
    const backup = resolve(reportRoot, `evidence/before-table-cells/${sourcePath}`);
    const env = { ...process.env, ZUI_LAYOUT_REPO_ROOT: repoRoot, ZUI_LAYOUT_OUTPUT_ROOT: catalogRoot, ZUI_LAYOUT_TABLE_OUTPUT_ROOT: reportRoot };
    return { root, repoRoot, catalogRoot, reportRoot, outside, sourcePath, dependencyPath, sourceBytes, dependencyBytes,
      entry, manifest, catalogPath, save, contexts, output, backup, env };
  };
  type Fixture = Awaited<ReturnType<typeof makeFixture>>;
  const check = (name: string, run: (f: Fixture) => Promise<void>) => t.test(name, async () => {
    const f = await makeFixture();
    try { await run(f); } finally { await rm(f.root, { recursive: true, force: false }); }
  });
  const rejectBeforeWrite = async (f: Fixture, value: unknown, expected: RegExp) => {
    await f.save(value);
    const before = await snapshot(f.root);
    await assert.rejects(runCurrentTableSourceCells(['--apply'], f.contexts), expected);
    assert.deepEqual(await snapshot(f.root), before);
  };
  try {
    await check('proposal writes only isolated report with prepared catalog and unchanged authored bytes', async (f) => {
      const input = await snapshot(f.catalogRoot);
      const result = await runCurrentTableSourceCells([], f.contexts);
      assert.equal(result.status, 'proposed'); assert.equal(result.applied, false);
      assert.equal(result.changedFiles, 1); assert.equal(result.changedRows, 1);
      assert.equal(result.records[0]!.state, 'planned');
      assert.equal(result.inputIdentity.catalogSha256, bytesSha256(await readFile(f.catalogPath)));
      assert.deepEqual(await snapshot(f.catalogRoot), input);
      assert.deepEqual(await readFile(resolve(f.repoRoot, f.sourcePath)), f.sourceBytes);
      assert.deepEqual(await readFile(resolve(f.repoRoot, f.dependencyPath)), f.dependencyBytes);
      assert.deepEqual(JSON.parse(await readFile(f.output(), 'utf8')), result);
      assert.equal(JSON.parse(await readFile(f.catalogPath, 'utf8')).entries[0].status, 'prepared');
      assert.equal(Object.keys(await snapshot(f.reportRoot)).some((path) => path.includes('before-table-cells')), false);
    });
    await check('apply backs up exact CRLF bytes and records actual source change without accepting catalog', async (f) => {
      const input = await snapshot(f.catalogRoot);
      const result = await runCurrentTableSourceCells(['--apply'], f.contexts);
      const updated = await readFile(resolve(f.repoRoot, f.sourcePath), 'utf8');
      assert.equal(result.status, 'applied'); assert.equal(result.applied, true);
      assert.equal(result.records[0]!.state, 'applied');
      assert.equal(result.records[0]!.observedSha256, bytesSha256(updated));
      assert.equal(result.records[0]!.afterSha256, bytesSha256(updated));
      assert.deepEqual(await readFile(f.backup), f.sourceBytes);
      assert.match(updated, /custom = "preserve", options = /); assert.doesNotMatch(updated, /(?<!\r)\n/);
      assert.match(updated, /events = \[\{ id = "Run"/);
      const originalDocument = parseZuiDocument(f.sourceBytes.toString()).document;
      const updatedDocument = parseZuiDocument(updated).document;
      delete updatedDocument.nodes!['row'].props!['options'];
      assert.deepEqual(updatedDocument, originalDocument);
      assert.deepEqual(explicitTableCells(updated).changes, []);
      assert.deepEqual(await snapshot(f.catalogRoot), input);
      assert.deepEqual(await readFile(resolve(f.repoRoot, f.dependencyPath)), f.dependencyBytes);
      assert.deepEqual(JSON.parse(await readFile(f.output(true), 'utf8')), result);
      assert.equal(JSON.parse(await readFile(f.catalogPath, 'utf8')).entries[0].visualStatus, 'pending');
    });
    await check('unknown schema historical role duplicate count and repository identities fail before writes', async (f) => {
      for (const bad of [{ ...f.manifest, historical_only: true }, { ...f.manifest, role: 'package' },
        { ...f.manifest, schema: 'history' }, { ...f.manifest, repoRoot: f.outside }, { ...f.manifest, sourceCount: 0 },
        { ...f.manifest, sourceCount: 2, entries: [f.entry, f.entry] }])
        await rejectBeforeWrite(f, bad, /identity|Duplicate|differs/);
    });
    await check('source and dependency catalog fingerprints are required even before proposal', async (f) => {
      await rejectBeforeWrite(f, { ...f.manifest, entries: [{ ...f.entry, sourceSha256: '0'.repeat(64) }] }, /Source changed/);
      await rejectBeforeWrite(f, { ...f.manifest, entries: [{ ...f.entry, dependencyFingerprints: [{ sourcePath: f.dependencyPath, sha256: '0'.repeat(64) }] }] }, /Dependency changed/);
    });
    await check('later invalid catalog entries cannot allow earlier authored source writes', async (f) => {
      await rejectBeforeWrite(f, { ...f.manifest, sourceCount: 2, entries: [f.entry,
        { ...f.entry, sourcePath: 'missing.zui' }] }, /ENOENT/);
    });
    await check('historical source traversal and unused output evidence paths receive preflight', async (f) => {
      for (const sourcePath of ['../escape.zui', 'docs/_data/layout/old.zui'])
        await rejectBeforeWrite(f, { ...f.manifest, entries: [{ ...f.entry, sourcePath }] }, /escapes|Historical/);
      await rejectBeforeWrite(f, { ...f.manifest, entries: [{ ...f.entry, previewPath: '../outside/preview.png' }] }, /escapes/);
      await rejectBeforeWrite(f, { ...f.manifest, entries: [{ ...f.entry, penpotEvidence: [{ screenshotPath: '../outside/capture.png' }] }] }, /escapes/);
    });
    await check('physical source scope cannot be widened with assets parent traversal', async (f) => {
      const path = resolve(f.repoRoot, 'elsewhere.zui'); await writeFile(path, f.sourceBytes);
      await f.save({ ...f.manifest, entries: [{ ...f.entry, sourcePath: 'zircon_editor/assets/../../elsewhere.zui' }] });
      const result = await runCurrentTableSourceCells(['--apply'], f.contexts);
      assert.equal(result.changedFiles, 0); assert.deepEqual(await readFile(path), f.sourceBytes);
    });
    await check('forged contexts overlapping roots and report override cannot reach writer', async (f) => {
      const before = await snapshot(f.root);
      await assert.rejects(runCurrentTableSourceCells([], { ...f.contexts, catalog: { ...f.contexts.catalog } as LayoutCurrentContext }), /Historical or package/);
      await assert.rejects(runCurrentTableSourceCells([], { catalog: f.contexts.catalog, report: f.contexts.catalog }), /disjoint/);
      await assert.rejects(runCurrentTableSourceCells(['--report-root', f.outside], f.contexts), /cannot be overridden/);
      assert.deepEqual(await snapshot(f.root), before);
      const nested = resolve(f.catalogRoot, 'nested'); await mkdir(nested);
      try { await assert.rejects(runCurrentTableSourceCells([], { catalog: f.contexts.catalog,
        report: createLayoutCurrentContext({ repoRoot: f.repoRoot, outputRoot: nested }) }), /disjoint/); }
      finally { await rmdir(nested); }
      assert.deepEqual(await snapshot(f.root), before);
    });
    await check('source directory aliases inside checkout are rejected without changing alias target', async (f) => {
      const alias = resolve(f.repoRoot, 'zircon_editor/assets/alias');
      await symlink(resolve(f.repoRoot, 'zircon_editor/assets/ui'), alias, 'junction');
      try { await rejectBeforeWrite(f, { ...f.manifest, entries: [{ ...f.entry, sourcePath: 'zircon_editor/assets/alias/table.zui' }] }, /alias/); }
      finally { await rm(alias); }
      assert.deepEqual(await readFile(resolve(f.repoRoot, f.sourcePath)), f.sourceBytes);
    });
    await check('report evidence escape alias is rejected before lock or backup writes', async (f) => {
      const evidence = resolve(f.reportRoot, 'evidence'); await symlink(f.outside, evidence, 'junction');
      const before = await snapshot(f.root);
      try {
        await assert.rejects(runCurrentTableSourceCells(['--apply'], f.contexts), /physically escapes|alias/);
        assert.deepEqual(await snapshot(f.root), before); assert.deepEqual(await readdir(f.outside), []);
      } finally { await rm(evidence); }
    });
    await check('existing report and backup bytes cannot be overwritten', async (f) => {
      await mkdir(dirname(f.output(true)), { recursive: true }); await writeFile(f.output(true), 'owned elsewhere');
      let before = await snapshot(f.root);
      await assert.rejects(runCurrentTableSourceCells(['--apply'], f.contexts), /Refusing existing/);
      assert.deepEqual(await snapshot(f.root), before);
      await rm(f.output(true)); await mkdir(dirname(f.backup), { recursive: true }); await writeFile(f.backup, 'owned backup');
      before = await snapshot(f.root);
      await assert.rejects(runCurrentTableSourceCells(['--apply'], f.contexts), /Refusing existing/);
      assert.deepEqual(await snapshot(f.root), before);
    });
    await check('nondirectory report ancestor is rejected before acquiring a lock or editing source', async (f) => {
      await writeFile(resolve(f.reportRoot, 'evidence'), 'owned file');
      const before = await snapshot(f.root);
      await assert.rejects(runCurrentTableSourceCells(['--apply'], f.contexts), /not a directory|ENOTDIR/);
      assert.deepEqual(await snapshot(f.root), before);
    });
    await check('report lock aliases are rejected without touching external owner bytes', async (f) => {
      await writeFile(resolve(f.outside, 'owner.json'), '{"external":true}');
      const alias = resolve(f.reportRoot, '.zui-layout-catalog.lock');
      await symlink(f.outside, alias, 'junction');
      const before = await snapshot(f.root);
      try {
        await assert.rejects(runCurrentTableSourceCells(['--apply'], f.contexts), /physically escapes|alias/);
        assert.deepEqual(await snapshot(f.root), before);
      } finally { await rm(alias); }
    });
    await check('prepared source drift rejects commit before any report lock or backup', async (f) => {
      const prepared = await prepareCurrentTableSourceCells(['--apply'], f.contexts);
      await writeFile(resolve(f.repoRoot, f.sourcePath), f.sourceBytes.toString() + '# foreign change\n');
      const before = await snapshot(f.root);
      await assert.rejects(commitCurrentTableSourceCells(prepared), /CAS conflict/);
      assert.deepEqual(await snapshot(f.root), before);
    });
    await check('prepared dependency drift rejects commit before source mutation', async (f) => {
      const prepared = await prepareCurrentTableSourceCells(['--apply'], f.contexts);
      await writeFile(resolve(f.repoRoot, f.dependencyPath), '# foreign dependency\n');
      const before = await snapshot(f.root);
      await assert.rejects(commitCurrentTableSourceCells(prepared), /CAS conflict/);
      assert.deepEqual(await snapshot(f.root), before);
    });
    await check('prepared catalog identity drift rejects commit before source mutation', async (f) => {
      const prepared = await prepareCurrentTableSourceCells(['--apply'], f.contexts);
      await f.save({ ...f.manifest, generatedAt: 'changed-after-plan' });
      const before = await snapshot(f.root);
      await assert.rejects(commitCurrentTableSourceCells(prepared), /catalog changed/);
      assert.deepEqual(await snapshot(f.root), before);
    });
    await check('opaque plan rejects forgery and ignores exposed report and caller context mutations', async (f) => {
      await assert.rejects(commitCurrentTableSourceCells({ report: {} } as PreparedTableSourceCells), /Unknown/);
      const prepared = await prepareCurrentTableSourceCells(['--apply'], f.contexts);
      prepared.report.records[0]!.afterSha256 = '0'.repeat(64);
      prepared.report.records.length = 0;
      f.contexts.report = f.contexts.catalog;
      const result = await commitCurrentTableSourceCells(prepared);
      assert.equal(result.records.length, 1); assert.equal(result.records[0]!.state, 'applied');
      assert.equal(result.records[0]!.afterSha256, bytesSha256(await readFile(resolve(f.repoRoot, f.sourcePath))));
      await assert.rejects(commitCurrentTableSourceCells(prepared), /already consumed/);
    });
    await check('report lock contention changes no source and creates no input lock', async (f) => {
      const lock = await acquireLayoutCatalogLock(f.reportRoot, { command: 'table-contention-test', context: f.contexts.report });
      const before = await snapshot(f.root);
      try {
        await assert.rejects(runCurrentTableSourceCells(['--apply'], f.contexts), /already held/);
        assert.deepEqual(await snapshot(f.root), before);
      } finally { await lock.release(); }
      assert.equal((await readdir(f.catalogRoot)).includes('.zui-layout-catalog.lock'), false);
    });
    await check('unresolved five column row remains authored and report explicitly records no changes', async (f) => {
      const bytes = Buffer.from(source.replace('Running   62%   Worker_03', 'Game   128   Valid   Source.ini'));
      await writeFile(resolve(f.repoRoot, f.sourcePath), bytes);
      await f.save({ ...f.manifest, entries: [{ ...f.entry, sourceSha256: bytesSha256(bytes) }] });
      const result = await runCurrentTableSourceCells(['--apply'], f.contexts);
      assert.equal(result.changedFiles, 0); assert.equal(result.unresolvedRows, 1);
      assert.match(result.records[0]!.unresolved[0]!.reason, /5 columns/);
      assert.deepEqual(await readFile(resolve(f.repoRoot, f.sourcePath)), bytes);
      assert.equal(Object.keys(await snapshot(f.reportRoot)).some((path) => path.includes('before-table-cells')), false);
    });
    await check('unknown duplicate missing relative and legacy CLI arguments fail before writes', async (f) => {
      const before = await snapshot(f.root);
      for (const args of [['--wat'], ['--apply', '--apply'], ['--report-root'], ['--catalog', f.catalogPath], ['--output', f.output()]])
        await assert.rejects(runCurrentTableSourceCells(args), /Unknown|Duplicate|Missing/);
      await assert.rejects(runCurrentTableSourceCells(['--report-root', 'relative']), /absolute/);
      assert.deepEqual(await snapshot(f.root), before);
    });
    await check('ambient roots must be explicit absolute and cannot overlap history', async (f) => {
      const before = await snapshot(f.root);
      await assert.rejects(runCurrentTableSourceCells(), /explicit ZUI_LAYOUT_OUTPUT_ROOT/);
      Object.assign(process.env, { ZUI_LAYOUT_REPO_ROOT: f.repoRoot, ZUI_LAYOUT_OUTPUT_ROOT: f.catalogRoot });
      try {
        await assert.rejects(runCurrentTableSourceCells(), /explicit absolute report root/);
        process.env['ZUI_LAYOUT_TABLE_OUTPUT_ROOT'] = f.reportRoot;
        process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = 'relative';
        await assert.rejects(runCurrentTableSourceCells(), /absolute ZUI_LAYOUT_OUTPUT_ROOT/);
        process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = f.catalogRoot;
        process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'] = f.root;
        await assert.rejects(runCurrentTableSourceCells(), /historical archive/);
      } finally { for (const key of envKeys) delete process.env[key]; }
      assert.deepEqual(await snapshot(f.root), before);
    });
    await check('actual CLI apply honors explicit isolated report root and leaves catalog readonly', async (f) => {
      const tsx = process.env['ZUI_LAYOUT_TEST_TSX_CLI']; const tsconfig = process.env['ZUI_LAYOUT_TEST_TSCONFIG'];
      assert.ok(tsx && tsconfig, 'Root must register TSX CLI and validation tsconfig');
      const tool = resolve(actualRepo, 'tools/penpot/apps/zircon-zui-plugin/tools/zui-table-source-cells.ts');
      const input = await snapshot(f.catalogRoot);
      const env: NodeJS.ProcessEnv = { ...f.env }; delete env['ZUI_LAYOUT_TABLE_OUTPUT_ROOT'];
      const result = await execute(process.execPath, [tsx, '--tsconfig', tsconfig, tool, '--apply', '--report-root', f.reportRoot],
        { cwd: actualRepo, env, timeout: 30000 });
      assert.equal(JSON.parse(result.stdout).status, 'applied');
      assert.deepEqual(await snapshot(f.catalogRoot), input);
      assert.deepEqual(await readFile(f.backup), f.sourceBytes);
      assert.match(await readFile(resolve(f.repoRoot, f.sourcePath), 'utf8'), /options = /);
      const before = await snapshot(f.root);
      await assert.rejects(execute(process.execPath, [tsx, '--tsconfig', tsconfig, tool, '--catalog', f.catalogPath],
        { cwd: actualRepo, env: f.env, timeout: 30000 }));
      assert.deepEqual(await snapshot(f.root), before);
    });
    await check('actual native loader import is inert with current roots absent', async (f) => {
      const tsx = process.env['ZUI_LAYOUT_TEST_TSX_CLI']; const tsconfig = process.env['ZUI_LAYOUT_TEST_TSCONFIG'];
      assert.ok(tsx && tsconfig);
      const before = await snapshot(fixture); const env = { ...process.env };
      for (const key of envKeys) delete env[key]; env['TSX_TSCONFIG_PATH'] = tsconfig;
      const tool = resolve(actualRepo, 'tools/penpot/apps/zircon-zui-plugin/tools/zui-table-source-cells.ts');
      const loader = pathToFileURL(resolve(dirname(tsx), 'loader.mjs')).href;
      const result = await execute(process.execPath, ['--import', loader, '--input-type=module', '--eval',
        `await import(${JSON.stringify(pathToFileURL(tool).href)}); console.log('inert')`], { cwd: actualRepo, env, timeout: 30000 });
      assert.equal(result.stdout.trim(), 'inert'); assert.deepEqual(await snapshot(fixture), before);
    });
  } finally {
    for (const key of envKeys) { if (previous[key] === undefined) delete process.env[key]; else process.env[key] = previous[key]; }
    await rm(fixture, { recursive: true, force: false });
  }
});
