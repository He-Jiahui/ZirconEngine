import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { mkdtemp, mkdir, readFile, readdir, readlink, rm, rmdir, symlink, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { promisify } from 'node:util';
import test from 'node:test';
import type { CatalogEntry, CatalogManifest } from './zui-layout-catalog';
import { bytesSha256 } from './zui-layout-evidence';
import { createLayoutCurrentContext, type LayoutCurrentContext } from './zui-layout-paths';
import { requireApprovedPenpotArtifactDirectory } from './zui-layout-artifact-path';
import { acquireLayoutCatalogLock } from './zui-layout-catalog-lock';

const execute = promisify(execFile);
test('style audit separates readonly current input from task report output', async (t) => {
  const artifactRoot = process.env['ZUI_LAYOUT_TEST_ARTIFACT_ROOT'];
  assert.ok(artifactRoot, 'Root must register ZUI_LAYOUT_TEST_ARTIFACT_ROOT');
  requireApprovedPenpotArtifactDirectory(artifactRoot, 'Style audit test fixture');
  const fixture = await mkdtemp(resolve(artifactRoot, 'layout-style-current-'));
  const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../../../../..');
  const catalogRoot = resolve(fixture, 'catalog');
  const reportRoot = resolve(fixture, 'report');
  const outside = resolve(fixture, 'outside');
  for (const root of [catalogRoot, reportRoot, outside]) await mkdir(root);
  const envKeys = ['ZUI_LAYOUT_REPO_ROOT', 'ZUI_LAYOUT_OUTPUT_ROOT', 'ZUI_LAYOUT_STYLE_AUDIT_OUTPUT_ROOT', 'ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'] as const;
  const previous = Object.fromEntries(envKeys.map((key) => [key, process.env[key]]));
  process.env['ZUI_LAYOUT_REPO_ROOT'] = repoRoot;
  process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = catalogRoot;
  process.env['ZUI_LAYOUT_STYLE_AUDIT_OUTPUT_ROOT'] = reportRoot;
  delete process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'];
  const sourcePath = 'tools/penpot/apps/zircon-zui-plugin/src/bridge/roundtrip-fixture.zui';
  const dependencyPath = 'tools/penpot/apps/zircon-zui-plugin/tools/zui-style-contract.ts';
  const sourceBytes = await readFile(resolve(repoRoot, sourcePath));
  const dependencyBytes = await readFile(resolve(repoRoot, dependencyPath));
  // Only a catalog fixture points at actual checkout bytes. No capture or acceptance is fabricated.
  const entry = {
    sourcePath, sourceSha256: bytesSha256(sourceBytes), sourceClassification: 'test-fixture',
    outputPath: 'fixture.zui', resultPath: 'result.md', previewPath: 'preview.png',
    dependencyFingerprints: [{ sourcePath: dependencyPath, sha256: bytesSha256(dependencyBytes) }],
    cases: [], status: 'prepared', visualStatus: 'pending',
  } as CatalogEntry;
  const manifest = { schema: 'dev.zircon.zui.layout-catalog', version: 1, generatedAt: 'fixture-not-capture', repoRoot, sourceCount: 1, entries: [entry] } as CatalogManifest;
  const catalogPath = resolve(catalogRoot, 'catalog.json');
  const reportPath = resolve(reportRoot, 'evidence/style-contract-audit.json');
  const save = (value: unknown = manifest) => writeFile(catalogPath, JSON.stringify(value));
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
  try {
    await save();
    const { runCurrentLayoutStyleAudit, buildStyleAudit } = await import('./zui-layout-style-audit');
    const catalog = createLayoutCurrentContext();
    const report = createLayoutCurrentContext({ repoRoot, outputRoot: reportRoot });
    const contexts = { catalog, report };
    const reject = async (value: unknown, expected: RegExp) => {
      await save(value);
      const inputBefore = await snapshot(catalogRoot);
      const outputBefore = await snapshot(reportRoot);
      await assert.rejects(runCurrentLayoutStyleAudit([], contexts), expected);
      assert.deepEqual(await snapshot(catalogRoot), inputBefore);
      assert.deepEqual(await snapshot(reportRoot), outputBefore);
    };
    await t.test('real current source write and check preserve catalog, dependencies, and prepared status', async () => {
      const inputBefore = await snapshot(catalogRoot);
      const result = await runCurrentLayoutStyleAudit([], contexts);
      assert.equal(result.sourceCount, 1);
      assert.equal(result.entries[0]!.sourceSha256, bytesSha256(sourceBytes));
      assert.equal(result.inputIdentity.catalogSha256, bytesSha256(await readFile(catalogPath)));
      assert.deepEqual(await snapshot(catalogRoot), inputBefore);
      const reportBefore = await snapshot(reportRoot);
      await runCurrentLayoutStyleAudit(['--check'], contexts);
      assert.deepEqual(await snapshot(reportRoot), reportBefore);
      assert.equal(JSON.parse(await readFile(catalogPath, 'utf8')).entries[0].status, 'prepared');
      assert.deepEqual(await readFile(resolve(repoRoot, sourcePath)), sourceBytes);
      assert.deepEqual(await readFile(resolve(repoRoot, dependencyPath)), dependencyBytes);
    });
    await t.test('historical, unknown schema, duplicate, count and repository identities fail before writes', async () => {
      for (const bad of [{ ...manifest, historical_only: true }, { ...manifest, role: 'package' }, { ...manifest, schema: 'history' }, { ...manifest, repoRoot: outside }, { ...manifest, sourceCount: 0 }, { ...manifest, sourceCount: 2, entries: [entry, entry] }])
        await reject(bad, /identity|Duplicate|differs/);
    });
    await t.test('stale actual source and dependency hashes cannot write a report', async () => {
      await reject({ ...manifest, entries: [{ ...entry, sourceSha256: '0'.repeat(64) }] }, /Source changed/);
      await reject({ ...manifest, entries: [{ ...entry, dependencyFingerprints: [{ sourcePath: dependencyPath, sha256: '0'.repeat(64) }] }] }, /Dependency changed/);
    });
    await t.test('source traversal and historical paths fail before any writes', async () => {
      for (const sourcePath of ['../escape.zui', 'docs/_data/layout/old.zui'])
        await reject({ ...manifest, entries: [{ ...entry, sourcePath }] }, /escapes|Historical/);
    });
    await t.test('all unused catalog output paths still receive physical preflight', async () => {
      await reject({ ...manifest, entries: [{ ...entry, previewPath: '../outside/preview.png' }] }, /escapes/);
    });
    await t.test('forged context and producer-report overlaps cannot reach a writer', async () => {
      await save();
      const before = await snapshot(reportRoot);
      await assert.rejects(runCurrentLayoutStyleAudit([], { catalog: { ...catalog } as LayoutCurrentContext, report }), /Historical or package/);
      await assert.rejects(runCurrentLayoutStyleAudit([], { catalog, report: catalog }), /disjoint/);
      const nested = resolve(catalogRoot, 'nested'); await mkdir(nested);
      await assert.rejects(runCurrentLayoutStyleAudit([], { catalog, report: createLayoutCurrentContext({ repoRoot, outputRoot: nested }) }), /disjoint/);
      await rmdir(nested);
      assert.deepEqual(await snapshot(reportRoot), before);
      await assert.rejects(buildStyleAudit(manifest, { ...catalog } as LayoutCurrentContext, '0'.repeat(64)), /Historical or package/);
    });
    await t.test('symlink source aliases outside checkout are rejected without writing', async () => {
      const alias = resolve(catalogRoot, 'escape');
      await symlink(outside, alias, 'junction');
      try {
        await reject({ ...manifest, entries: [{ ...entry, outputPath: 'escape/file.zui' }] }, /physically escapes/);
      } finally { await rm(alias); }
    });
    await t.test('history output roots are rejected without history fallback', async () => {
      process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'] = fixture;
      try { await assert.rejects(runCurrentLayoutStyleAudit(), /historical archive/); }
      finally { delete process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT']; }
    });
    await t.test('unknown duplicate missing and legacy CLI arguments fail before writes', async () => {
      await save();
      const before = await snapshot(reportRoot);
      for (const args of [['--wat'], ['--check', '--check'], ['--report-root'], ['--catalog', catalogPath], ['--output', reportPath]])
        await assert.rejects(runCurrentLayoutStyleAudit(args), /Unknown|Duplicate|Missing/);
      await assert.rejects(runCurrentLayoutStyleAudit(['--report-root', 'relative']), /absolute/);
      assert.deepEqual(await snapshot(reportRoot), before);
    });
    await t.test('relative current catalog environment is rejected before writing', async () => {
      const before = await snapshot(reportRoot);
      process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = 'relative-catalog';
      try { await assert.rejects(runCurrentLayoutStyleAudit(), /absolute ZUI_LAYOUT_OUTPUT_ROOT/); }
      finally { process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = catalogRoot; }
      assert.deepEqual(await snapshot(reportRoot), before);
    });
    await t.test('check rejects catalog and report identity drift without changing either role', async () => {
      await save(); await runCurrentLayoutStyleAudit([], contexts);
      const stored = JSON.parse(await readFile(reportPath, 'utf8'));
      await writeFile(reportPath, JSON.stringify({ ...stored, inputIdentity: { ...stored.inputIdentity, catalogSha256: '0'.repeat(64) } }));
      const before = await snapshot(reportRoot);
      await assert.rejects(runCurrentLayoutStyleAudit(['--check'], contexts), /differs/);
      assert.deepEqual(await snapshot(reportRoot), before);
      await runCurrentLayoutStyleAudit([], contexts);
      await save({ ...manifest, generatedAt: 'different-catalog-same-source' });
      await assert.rejects(runCurrentLayoutStyleAudit(['--check'], contexts), /differs/);
      await save();
    });
    await t.test('check on a missing report creates no evidence directory or lock', async () => {
      await rm(resolve(reportRoot, 'evidence'), { recursive: true });
      const before = await snapshot(reportRoot);
      await assert.rejects(runCurrentLayoutStyleAudit(['--check'], contexts), /ENOENT/);
      assert.deepEqual(await snapshot(reportRoot), before);
    });
    await t.test('report evidence aliases outside its role are rejected before lock creation', async () => {
      const evidence = resolve(reportRoot, 'evidence');
      await symlink(outside, evidence, 'junction');
      const inputBefore = await snapshot(catalogRoot);
      const outputBefore = await snapshot(reportRoot);
      try {
        await assert.rejects(runCurrentLayoutStyleAudit([], contexts), /physically escapes/);
        assert.deepEqual(await snapshot(catalogRoot), inputBefore);
        assert.deepEqual(await snapshot(reportRoot), outputBefore);
        assert.deepEqual(await readdir(outside), []);
      } finally { await rm(evidence); }
    });
    await t.test('report lock contention never creates a producer lock or changes input', async () => {
      const lock = await acquireLayoutCatalogLock(reportRoot, { command: 'style-audit-contention-test', context: report });
      const before = await snapshot(catalogRoot);
      try { await assert.rejects(runCurrentLayoutStyleAudit([], contexts), /already held/); }
      finally { await lock.release(); }
      assert.deepEqual(await snapshot(catalogRoot), before);
      assert.equal((await readdir(catalogRoot)).includes('.zui-layout-catalog.lock'), false);
    });
    await t.test('actual CLI writes and checks only its admitted report root', async () => {
      const tsx = process.env['ZUI_LAYOUT_TEST_TSX_CLI'];
      const tsconfig = process.env['ZUI_LAYOUT_TEST_TSCONFIG'];
      assert.ok(tsx && tsconfig, 'Root must register explicit TSX CLI and validation tsconfig');
      const tool = resolve(repoRoot, 'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-style-audit.ts');
      const before = await snapshot(catalogRoot);
      const call = (args: string[]) => execute(process.execPath, [tsx, '--tsconfig', tsconfig, tool, ...args], { cwd: repoRoot, env: process.env, timeout: 30000 });
      assert.equal(JSON.parse((await call([])).stdout).wrote, true);
      assert.equal(JSON.parse((await call(['--check'])).stdout).wrote, false);
      await assert.rejects(call(['--catalog', catalogPath]));
      assert.deepEqual(await snapshot(catalogRoot), before);
    });
    await t.test('actual import is inert even without configured current roots', async () => {
      const tsx = process.env['ZUI_LAYOUT_TEST_TSX_CLI'];
      const tsconfig = process.env['ZUI_LAYOUT_TEST_TSCONFIG'];
      assert.ok(tsx && tsconfig);
      const before = await snapshot(fixture);
      const env = { ...process.env };
      for (const key of envKeys) delete env[key];
      const modulePath = resolve(repoRoot, 'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-style-audit.ts');
      env['TSX_TSCONFIG_PATH'] = tsconfig;
      const loader = pathToFileURL(resolve(dirname(tsx), 'loader.mjs')).href;
      const imported = await execute(process.execPath, ['--import', loader, '--input-type=module', '--eval', `await import(${JSON.stringify(pathToFileURL(modulePath).href)}); console.log('inert')`], { cwd: repoRoot, env, timeout: 30000 });
      assert.equal(imported.stdout.trim(), 'inert');
      assert.deepEqual(await snapshot(fixture), before);
    });
  } finally {
    for (const key of envKeys) {
      if (previous[key] === undefined) delete process.env[key]; else process.env[key] = previous[key];
    }
    await rm(fixture, { recursive: true, force: false });
  }
});
