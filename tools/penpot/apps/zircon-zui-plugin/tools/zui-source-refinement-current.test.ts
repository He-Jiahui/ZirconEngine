import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { mkdtemp, mkdir, readFile, readdir, readlink, rm, symlink, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { promisify } from 'node:util';
import test from 'node:test';
import type { CatalogEntry, CatalogManifest } from './zui-layout-catalog';
import { bytesSha256 } from './zui-layout-evidence';
import { createLayoutCurrentContext } from './zui-layout-paths';
import { requireApprovedPenpotArtifactDirectory } from './zui-layout-artifact-path';
import { acquireLayoutCatalogLock } from './zui-layout-catalog-lock';
import { inline, runCurrentMaterialSourceRefinement } from './zui-material-source-refinement';
import { runCurrentMaterialSampleRefinement } from './zui-material-sample-refinement';
import { runCurrentStyleSourceRefinement } from './zui-style-source-refinement';
import { runCurrentWorkbenchSourceRefinement } from './zui-workbench-source-refinement';
import { parseZuiDocument } from '../src/bridge/zui-document';
import { parseZuiLayoutSource } from '../src/bridge/zui-layout-optimizer';
import { LayoutDependencies } from './zui-layout-dependencies';
import {
  assertCurrentLayoutSnapshot, commitCurrentSourceRefinement, prepareCurrentLayoutSnapshot,
  prepareCurrentSourceRefinement, readCurrentSourceReport, runCurrentSourceRefinement,
  writeCurrentCaptureFile, type SourceRefinementSpec, type PreparedCurrentSourceRefinement,
} from './zui-source-refinement-current';

const execute = promisify(execFile);
const prefix = 'zircon_editor/assets/ui/editor/';
const paths = [
  'components/showcase/showcase_collections_section.zui',
  'components/showcase/showcase_input_section.zui',
  'components/showcase/showcase_selection_section.zui',
  'components/workbench/composites/animation/workbench_transport_controls.zui',
  'components/workbench/composites/animation/workbench_sample_weights.zui',
  'components/workbench/floating/workbench_command_palette.zui',
].map((path) => prefix + path);
const generic: SourceRefinementSpec = {
  id: 'style-source-normalization', select: () => true,
  transform: (source) => ({ source: source + '# changed\n', changes: ['synthetic suffix'] }),
};
const tree = async (root: string): Promise<Record<string, string>> => {
  const result: Record<string, string> = {};
  const visit = async (root: string, prefix = ''): Promise<void> => {
    for (const entry of await readdir(root, { withFileTypes: true })) {
      const key = prefix + entry.name, path = resolve(root, entry.name);
      if (entry.isSymbolicLink()) result[key] = 'link:' + await readlink(path);
      else if (entry.isDirectory()) { result[key + '/'] = 'directory'; await visit(path, key + '/'); }
      else result[key] = bytesSha256(await readFile(path));
    }
  };
  await visit(root); return result;
};
function document(nodes: Record<string, Record<string, unknown>>): string {
  return '[asset]\nkind = "view"\nid = "synthetic"\nversion = 2\n[root]\nnode = "root"\n[imports]\nwidgets = []\nstyles = []\n' +
    Object.entries(nodes).map(([id, values]) => '[nodes.' + id + ']\n' +
      Object.entries(values).map(([key, value]) => key + ' = ' + inline(value)).join('\n')).join('\n') + '\n';
}
const node = (component: string, extra: Record<string, unknown> = {}) => ({
  component, props: { text: 'Checkboxes', custom: 'preserve' },
  layout: { height: { preferred: 34 }, container: { kind: 'VerticalBox', gap: 4 } }, ...extra,
});
function materialSource(): string {
  const nodes: Record<string, Record<string, unknown>> = {
    root: node('VerticalBox', { children: ['title', 'meta', 'sample', 'state_strip'].map((node) => ({ node })) }),
    title: node('Text'), sample: node('Space'), meta: node('VerticalBox', {
      children: [0, 1, 2, 3].map((i) => ({ node: 'meta_' + i })),
    }), state_strip: node('VerticalBox', { children: Array.from({ length: 8 }, (_, i) => ({ node: 'state_' + i })) }),
  };
  for (let i = 0; i < 4; i++) nodes['meta_' + i] = node('Text');
  for (let i = 0; i < 8; i++) nodes['state_' + i] = node('Text');
  return document(nodes);
}
function sampleSource(): string {
  return document({ root: node('VerticalBox'), title: node('WorkbenchLabel'),
    sample: node('HorizontalBox', { children: [{ node: 'first' }, { node: 'second' }] }),
    first: node('Checkbox', { props: { checked: true, custom: 'preserve' },
      events: [{ id: 'Toggle', event: 'Click', route: 'checkbox.toggle' }], bindings: { enabled: 'model.enabled' } }),
    second: node('Checkbox', { props: { checked: false, disabled: true } }) });
}
const styleSource = document({ root: node('VerticalBox', { children: [{ node: 'command' }] }),
  command: node('WorkbenchButton', { props: { text: 'Run', font_size: 11, gap: 6, corner_radius: 10, custom: 'preserve' },
    events: [{ id: 'Run', event: 'Click', route: 'command.run' }], bindings: { enabled: 'model.enabled' } }) });
const workbenchSource = document({ root: node('Space'), fixture_section_root: node('VerticalGroup') })
  .replace('kind = "view"', 'kind = "component"')
  .replace('[root]\nnode = "root"\n', '[components.Fixture]\nroot = "fixture_section_root"\n');

test('remaining source consumers use readonly current inputs and separate explicit reports', async (t) => {
  const configured = process.env['ZUI_LAYOUT_TEST_ARTIFACT_ROOT'];
  assert.ok(configured, 'Root must register the future ZUI_LAYOUT_TEST_ARTIFACT_ROOT');
  requireApprovedPenpotArtifactDirectory(configured, 'Source refinement fixture');
  const fixture = await mkdtemp(resolve(configured, 'layout-source-current-'));
  const make = async (sources: Record<string, string> = { [prefix + 'inputs/synthetic.zui']: styleSource }) => {
    const root = await mkdtemp(resolve(fixture, 'case-')), repoRoot = resolve(root, 'repo');
    const catalogRoot = resolve(root, 'catalog'), reportRoot = resolve(root, 'report');
    await mkdir(resolve(repoRoot, 'zircon_runtime/src'), { recursive: true });
    for (const path of [catalogRoot, reportRoot]) await mkdir(path);
    await writeFile(resolve(repoRoot, 'Cargo.toml'), '# synthetic marker\n');
    await writeFile(resolve(repoRoot, 'zircon_runtime/src/lib.rs'), '// synthetic marker\n');
    const entries: CatalogEntry[] = [];
    for (const [sourcePath, source] of Object.entries(sources)) {
      const path = resolve(repoRoot, sourcePath); await mkdir(dirname(path), { recursive: true });
      await writeFile(path, source.replaceAll('\n', '\r\n'));
      entries.push({ sourcePath, sourceSha256: bytesSha256(await readFile(path)),
        sourceClassification: 'test-fixture', outputPath: 'out/' + entries.length + '.zui',
        previewPath: 'preview/' + entries.length + '.png', resultPath: 'results/' + entries.length + '.md',
        dependencyFingerprints: [], cases: [], status: 'prepared', visualStatus: 'pending' } as CatalogEntry);
    }
    const manifest = { schema: 'dev.zircon.zui.layout-catalog', version: 1, generatedAt: 'synthetic-not-capture',
      repoRoot, sourceCount: entries.length, entries } as CatalogManifest;
    const catalogPath = resolve(catalogRoot, 'catalog.json');
    const save = (value: unknown = manifest) => writeFile(catalogPath, JSON.stringify(value));
    await save();
    const contexts = { catalog: createLayoutCurrentContext({ repoRoot, outputRoot: catalogRoot }),
      report: createLayoutCurrentContext({ repoRoot, outputRoot: reportRoot }) };
    return { root, repoRoot, catalogRoot, reportRoot, catalogPath, manifest, contexts, save };
  };
  type Fixture = Awaited<ReturnType<typeof make>>;
  const check = (name: string, run: (f: Fixture) => Promise<void>) => t.test(name, async () => {
    const f = await make(); try { await run(f); } finally { await rm(f.root, { recursive: true, force: false }); }
  });
  try {
    await check('proposal writes only explicit report, not authored sources catalog or backups', async (f) => {
      const catalog = await tree(f.catalogRoot), source = await tree(f.repoRoot);
      const report = await runCurrentSourceRefinement(generic, [], f.contexts);
      assert.equal(report.status, 'proposed'); assert.equal(report.applied, false); assert.equal(report.changedFiles, 1);
      assert.deepEqual(await tree(f.repoRoot), source); assert.deepEqual(await tree(f.catalogRoot), catalog);
      assert.deepEqual(Object.keys(await tree(f.reportRoot)), ['evidence/', 'evidence/style-source-normalization-proposed.json']);
    });
    await t.test('four real transformer entries preserve original contracts and exact backup bytes', async () => {
      const variants = [
        { run: runCurrentMaterialSourceRefinement, sources: { [prefix + 'material.zui']: materialSource() } },
        { run: runCurrentMaterialSampleRefinement, sources: { [prefix + 'inputs/material.zui']: sampleSource() } },
        { run: runCurrentStyleSourceRefinement, sources: { [prefix + 'inputs/style.zui']: styleSource } },
        { run: runCurrentWorkbenchSourceRefinement, sources: Object.fromEntries(paths.map((path) => [path, workbenchSource])) },
      ];
      for (const variant of variants) {
        const f = await make(variant.sources);
        try {
          const catalog = await tree(f.catalogRoot), before = await tree(f.repoRoot);
          const report = await variant.run(['--apply'], f.contexts);
          assert.equal(report.status, 'applied'); assert.equal(report.changedFiles, Object.keys(variant.sources).length);
          for (const record of report.records) {
            const updated = await readFile(resolve(f.repoRoot, record.sourcePath));
            const backup = await readFile(resolve(f.reportRoot, record.backupPath as string));
            assert.equal(bytesSha256(backup), before[record.sourcePath]);
            assert.equal(bytesSha256(updated), record.afterSha256); assert.equal(record.state, 'applied');
            assert.doesNotMatch(updated.toString(), /(?<!\r)\n/);
            const oldNodes = parseZuiDocument(backup.toString()).document.nodes!;
            const newNodes = parseZuiDocument(updated.toString()).document.nodes!;
            for (const [id, oldNode] of Object.entries(oldNodes)) {
              assert.ok(newNodes[id], 'all original nodes remain');
              for (const key of ['events', 'bindings', 'children'] as const)
                assert.deepEqual(newNodes[id][key], oldNode[key], id + '.' + key);
              if (oldNode.props?.['custom']) assert.equal(newNodes[id].props?.['custom'], oldNode.props['custom']);
            }
          }
          assert.deepEqual(await tree(f.catalogRoot), catalog);
        } finally { await rm(f.root, { recursive: true, force: false }); }
      }
    });
    await check('later invalid input prevents all earlier source or backup writes', async (f) => {
      f.manifest.entries.push({ ...f.manifest.entries[0]!, sourcePath: prefix + 'missing.zui' });
      f.manifest.sourceCount++; await f.save();
      const before = await tree(f.root);
      await assert.rejects(runCurrentSourceRefinement(generic, ['--apply'], f.contexts), /ENOENT/);
      assert.deepEqual(await tree(f.root), before);
    });
    await check('historical schemas roles duplicate sources and traversal reject without lock creation', async (f) => {
      for (const bad of [{ ...f.manifest, historical_only: true }, { ...f.manifest, role: 'package' },
        { ...f.manifest, schema: 'history' }, { ...f.manifest, sourceCount: 2, entries: [f.manifest.entries[0], f.manifest.entries[0]] },
        { ...f.manifest, entries: [{ ...f.manifest.entries[0], sourcePath: 'docs/_data/layout/old.zui' }] },
        { ...f.manifest, entries: [{ ...f.manifest.entries[0], sourcePath: '../escape.zui' }] }]) {
        await f.save(bad); const before = await tree(f.root);
        await assert.rejects(runCurrentSourceRefinement(generic, ['--apply'], f.contexts));
        assert.deepEqual(await tree(f.root), before);
      }
    });
    await check('all source and dependency changes after preparation fail before backup', async (f) => {
      const sourcePath = f.manifest.entries[0]!.sourcePath;
      const dependencyPath = prefix + 'dependency.zui';
      await writeFile(resolve(f.repoRoot, dependencyPath), '# dependency\n');
      f.manifest.entries[0]!.dependencyFingerprints = [{ sourcePath: dependencyPath,
        sha256: bytesSha256(await readFile(resolve(f.repoRoot, dependencyPath))) }]; await f.save();
      for (const path of [sourcePath, dependencyPath]) {
        const prepared = await prepareCurrentSourceRefinement(generic, ['--apply'], f.contexts);
        const file = resolve(f.repoRoot, path), bytes = await readFile(file);
        await writeFile(file, Buffer.concat([bytes, Buffer.from('# changed\n')]));
        const before = await tree(f.root);
        await assert.rejects(commitCurrentSourceRefinement(prepared), /changed after preflight/);
        assert.deepEqual(await tree(f.root), before); await writeFile(file, bytes);
      }
    });
    await check('loader icon bytes and absent optional fixture are part of capture preflight', async (f) => {
      const sourcePath = f.manifest.entries[0]!.sourcePath, sourceFile = resolve(f.repoRoot, sourcePath);
      const iconSource = styleSource.replace(/props = ([^\n]+)/, (line) => line.replace('{', '{ icon = "res://icons/synthetic.svg",'));
      await writeFile(sourceFile, iconSource);
      f.manifest.entries[0]!.sourceSha256 = bytesSha256(await readFile(sourceFile)); await f.save();
      const icon = resolve(f.repoRoot, 'zircon_editor/assets/icons/synthetic.svg');
      await mkdir(dirname(icon), { recursive: true }); await writeFile(icon, '<svg/>');
      const snapshot = await prepareCurrentLayoutSnapshot(f.contexts.catalog, [], true);
      await assertCurrentLayoutSnapshot(snapshot);
      await writeFile(icon, '<svg changed="yes"/>');
      await assert.rejects(assertCurrentLayoutSnapshot(snapshot), /changed after preflight/);
      const second = await prepareCurrentLayoutSnapshot(f.contexts.catalog, [], true);
      const optional = resolve(f.repoRoot, 'zircon_runtime/src/dynamic_api/session/tests/runtime_ui_surface.rs');
      await mkdir(dirname(optional), { recursive: true }); await writeFile(optional, '// appeared fixture');
      await assert.rejects(assertCurrentLayoutSnapshot(second), /appeared after preflight/);
    });
    await t.test('legacy migrated root component and nested icons pin loader bytes and presence', async () => {
      const sourcePath = prefix + 'inputs/legacy.zui';
      const iconNode = '{ type = "Icon", props = { icon = "res://icons/legacy.svg" } }';
      const variants = [
        '[asset]\nkind = "layout"\nid = "legacy-root"\nversion = 1\n[root]\ntype = "Icon"\n[root.props]\nicon = "res://icons/legacy.svg"\n',
        '[asset]\nkind = "widget"\nid = "legacy-component"\nversion = 1\n[components.Legacy]\nroot = ' + iconNode + '\n',
        '[asset]\nkind = "layout"\nid = "legacy-child"\nversion = 1\n[root]\ntype = "VerticalBox"\nchildren = [{ node = ' + iconNode + ' }]\n',
      ];
      for (const source of variants) {
        const f = await make({ [sourcePath]: source });
        try {
          const icon = resolve(f.repoRoot, 'zircon_editor/assets/icons/legacy.svg');
          const svg = '<svg xmlns="http://www.w3.org/2000/svg" data-fixture="legacy"/>';
          await mkdir(dirname(icon), { recursive: true }); await writeFile(icon, svg);
          const original = await readFile(resolve(f.repoRoot, sourcePath), 'utf8');
          const parsed = parseZuiLayoutSource(original, sourcePath);
          assert.equal(parsed.sourceFormat, 'legacy');
          assert.ok(Object.values(parsed.document.nodes ?? {}).some((node) => node.props?.['icon'] === 'res://icons/legacy.svg'));
          const snapshot = await prepareCurrentLayoutSnapshot(f.contexts.catalog, [], true);
          const dependencies = new LayoutDependencies(); await dependencies.load(f.repoRoot, [sourcePath]);
          dependencies.embed(parsed.document, sourcePath);
          assert.ok(Object.values(parsed.document.nodes ?? {}).some((node) => node['penpot_icon_svg'] === svg), 'actual loader must read the migrated icon');
          await assertCurrentLayoutSnapshot(snapshot);
          await writeFile(icon, '<svg data-fixture="changed"/>');
          await assert.rejects(assertCurrentLayoutSnapshot(snapshot), /changed after preflight/);
          await rm(icon);
          const absentSnapshot = await prepareCurrentLayoutSnapshot(f.contexts.catalog, [], true);
          const absentParsed = parseZuiLayoutSource(original, sourcePath);
          const absentDependencies = new LayoutDependencies(); await absentDependencies.load(f.repoRoot, [sourcePath]);
          absentDependencies.embed(absentParsed.document, sourcePath);
          assert.ok(Object.values(absentParsed.document.nodes ?? {}).every((node) => node['penpot_icon_svg'] === undefined));
          await assertCurrentLayoutSnapshot(absentSnapshot);
          const before = await tree(f.catalogRoot);
          await writeFile(icon, svg);
          await assert.rejects(assertCurrentLayoutSnapshot(absentSnapshot), /appeared after preflight/);
          assert.deepEqual(await tree(f.catalogRoot), before);
          assert.equal(await readFile(resolve(f.repoRoot, sourcePath), 'utf8'), original);
          assert.deepEqual(await tree(f.reportRoot), {});
        } finally { await rm(f.root, { recursive: true, force: false }); }
      }
    });
    await check('catalog mutation and exposed snapshot mutation cannot bypass CAS', async (f) => {
      const prepared = await prepareCurrentSourceRefinement(generic, ['--apply'], f.contexts);
      await f.save({ ...f.manifest, generatedAt: 'changed' }); const before = await tree(f.root);
      await assert.rejects(commitCurrentSourceRefinement(prepared), /catalog changed/);
      assert.deepEqual(await tree(f.root), before); await f.save();
      const snapshot = await prepareCurrentLayoutSnapshot(f.contexts.catalog);
      snapshot.manifest.entries[0]!.sourcePath = 'forged.zui';
      await assert.rejects(assertCurrentLayoutSnapshot(snapshot), /mutated current layout snapshot/);
    });
    await check('forged plan forged context report overlap and override are rejected', async (f) => {
      const before = await tree(f.root);
      await assert.rejects(commitCurrentSourceRefinement({ report: {} } as PreparedCurrentSourceRefinement), /Unknown/);
      await assert.rejects(runCurrentSourceRefinement(generic, [], { catalog: { ...f.contexts.catalog }, report: f.contexts.report }), /Historical or package/);
      await assert.rejects(runCurrentSourceRefinement(generic, [], { catalog: f.contexts.catalog, report: f.contexts.catalog }), /disjoint/);
      await assert.rejects(runCurrentSourceRefinement(generic, ['--report-root', f.reportRoot], f.contexts), /cannot be overridden/);
      assert.deepEqual(await tree(f.root), before);
    });
    await check('held coordination lock and existing backup cannot be overwritten', async (f) => {
      const lock = await acquireLayoutCatalogLock(f.reportRoot, { command: 'negative', context: f.contexts.report });
      try {
        const before = await tree(f.root);
        await assert.rejects(runCurrentSourceRefinement(generic, ['--apply'], f.contexts), /lock/i);
        assert.deepEqual(await tree(f.root), before);
      } finally { await lock.release(); }
      const backup = resolve(f.reportRoot, 'backups/style-source-normalization/' + f.manifest.entries[0]!.sourcePath);
      await mkdir(dirname(backup), { recursive: true }); await writeFile(backup, 'sentinel');
      const before = await tree(f.root);
      await assert.rejects(runCurrentSourceRefinement(generic, ['--apply'], f.contexts), /existing/);
      assert.deepEqual(await tree(f.root), before);
    });
    await check('source alias and report alias are rejected without changing physical targets', async (f) => {
      const alias = resolve(f.repoRoot, 'alias');
      await symlink(resolve(f.repoRoot, 'zircon_editor'), alias, process.platform === 'win32' ? 'junction' : 'dir');
      try {
        await f.save({ ...f.manifest, entries: [{ ...f.manifest.entries[0],
          sourcePath: 'alias/assets/ui/editor/inputs/synthetic.zui' }] });
        const before = await tree(f.root);
        await assert.rejects(runCurrentSourceRefinement(generic, ['--apply'], f.contexts), /alias/i);
        assert.deepEqual(await tree(f.root), before);
      } finally { await rm(alias); }
      await f.save();
      const destination = resolve(f.root, 'outside'); await mkdir(destination);
      const evidence = resolve(f.reportRoot, 'evidence');
      await symlink(destination, evidence, process.platform === 'win32' ? 'junction' : 'dir');
      try {
        const before = await tree(f.root);
        await assert.rejects(runCurrentSourceRefinement(generic, ['--apply'], f.contexts), { message: 'Layout path physically escapes role root' });
        assert.deepEqual(await tree(f.root), before);
      } finally { await rm(evidence); }
    });
    await check('one-shot prepared plan and exposed report cannot mutate private batch', async (f) => {
      const prepared = await prepareCurrentSourceRefinement(generic, ['--apply'], f.contexts);
      prepared.report.records.length = 0; prepared.report.applyRequested = false;
      const result = await commitCurrentSourceRefinement(prepared);
      assert.equal(result.applied, true); assert.equal(result.records.length, 1);
      await assert.rejects(commitCurrentSourceRefinement(prepared), /consumed/);
    });
    await check('explicit report reader preserves table object changes and material detail', async (f) => {
      const sourcePath = f.manifest.entries[0]!.sourcePath, hash = f.manifest.entries[0]!.sourceSha256;
      const record = { sourcePath, beforeSha256: hash, afterSha256: hash, changes: [{ nodeId: 'row', options: ['a', 'b'] }], unresolved: ['retain'] };
      await mkdir(resolve(f.reportRoot, 'evidence'));
      const path = resolve(f.reportRoot, 'evidence/table-source-cells-applied.json');
      await writeFile(path, JSON.stringify({ applied: true, records: [record] }));
      assert.deepEqual([...(await readCurrentSourceReport(f.contexts.catalog, 'table-source-cells-applied.json', f.reportRoot)).values()], [record]);
      const material = { ...record, changes: ['retain detail'], titleHeight: 56, unknownDetail: { retained: true } };
      await writeFile(resolve(f.reportRoot, 'evidence/material-source-applied.json'), JSON.stringify({ applied: true, records: [material] }));
      assert.deepEqual([...(await readCurrentSourceReport(f.contexts.catalog, 'material-source-applied.json', f.reportRoot)).values()], [material]);
      await writeFile(path, JSON.stringify({ applied: true, records: [{ ...record, afterSha256: '0'.repeat(64) }] }));
      assert.equal((await readCurrentSourceReport(f.contexts.catalog, 'table-source-cells-applied.json', f.reportRoot)).size, 0);
      await writeFile(path, '{malformed'); await assert.rejects(readCurrentSourceReport(f.contexts.catalog, 'table-source-cells-applied.json', f.reportRoot));
      await assert.rejects(readCurrentSourceReport(f.contexts.catalog, '../escape.json', f.reportRoot), /filename/);
      await assert.rejects(readCurrentSourceReport(f.contexts.catalog, 'table-source-cells-applied.json', f.catalogRoot), /disjoint/);
    });
    await check('capture output writer rejects overwrite and path escape', async (f) => {
      const path = resolve(f.reportRoot, 'capture/result.json');
      await writeCurrentCaptureFile(f.contexts.report, path, 'first');
      await assert.rejects(writeCurrentCaptureFile(f.contexts.report, path, 'second'), /existing/);
      await assert.rejects(writeCurrentCaptureFile(f.contexts.report, '../escape.json', 'bad'), /escapes/);
      assert.equal(await readFile(path, 'utf8'), 'first');
    });
    await check('actual CLI imports tolerate missing output and invalid URL without IO or browser', async (f) => {
      const here = dirname(fileURLToPath(import.meta.url));
      const tsx = process.env['ZUI_LAYOUT_TEST_TSX_CLI'];
      const tsconfig = process.env['ZUI_LAYOUT_TEST_TSCONFIG'];
      assert.ok(tsx && tsconfig, 'Root must register TSX CLI and validation tsconfig');
      const files = ['zui-material-source-refinement.ts', 'zui-material-sample-refinement.ts',
        'zui-style-source-refinement.ts', 'zui-workbench-source-refinement.ts',
        'penpot-native-components-browser-contract.ts', 'penpot-browser-canvas-contract.ts',
        'penpot-workbench-export-apply-contract.ts', 'zui-layout-catalog.ts'];
      const env: NodeJS.ProcessEnv = { ...process.env, TSX_TSCONFIG_PATH: tsconfig, PENPOT_BASE_URL: 'invalid url', ZUI_LAYOUT_SOURCE_REPORT_ROOT: '' };
      delete env['ZUI_LAYOUT_OUTPUT_ROOT']; delete env['ZUI_PLUGIN_DIST_ROOT'];
      const before = await tree(f.root);
      await execute(process.execPath, ['--import', pathToFileURL(resolve(dirname(tsx), 'loader.mjs')).href, '--input-type=module', '--eval',
        'for (const url of ' + JSON.stringify(files.map((name) => pathToFileURL(resolve(here, name)).href)) + ') await import(url);'],
      { cwd: resolve(here, '..'), env, timeout: 30000 });
      assert.deepEqual(await tree(f.root), before);
    });
    await check('actual style CLI executes explicit apply while malformed CLI never writes', async (f) => {
      const here = dirname(fileURLToPath(import.meta.url));
      const tsx = process.env['ZUI_LAYOUT_TEST_TSX_CLI'];
      const tsconfig = process.env['ZUI_LAYOUT_TEST_TSCONFIG'];
      assert.ok(tsx && tsconfig, 'Root must register TSX CLI and validation tsconfig');
      const env = { ...process.env, ZUI_LAYOUT_REPO_ROOT: f.repoRoot,
        ZUI_LAYOUT_OUTPUT_ROOT: f.catalogRoot, ZUI_LAYOUT_SOURCE_REPORT_ROOT: f.reportRoot };
      for (const name of ['material-source', 'material-sample', 'style-source', 'workbench-source']) {
        const before = await tree(f.root);
        await assert.rejects(execute(process.execPath, [tsx, '--tsconfig', tsconfig, resolve(here, 'zui-' + name + '-refinement.ts'), '--unknown'], { cwd: resolve(here, '..'), env, timeout: 30000 }));
        assert.deepEqual(await tree(f.root), before);
      }
      await execute(process.execPath, [tsx, '--tsconfig', tsconfig, resolve(here, 'zui-style-source-refinement.ts'), '--apply'], { cwd: resolve(here, '..'), env, timeout: 30000 });
      const report = JSON.parse(await readFile(resolve(f.reportRoot, 'evidence/style-source-normalization-applied.json'), 'utf8'));
      assert.equal(report.applied, true); assert.equal(report.changedFiles, 1);
    });
  } finally { await rm(fixture, { recursive: true, force: false }); }
});
