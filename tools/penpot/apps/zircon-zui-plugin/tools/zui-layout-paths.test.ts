import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, readdir, rm, symlink, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import {
  assertLayoutCurrentRoots,
  createLayoutCurrentContext,
  layoutCurrentOutputPath,
  layoutCurrentSourcePath,
  layoutSourceRoot,
  resolveLayoutCatalogPath,
  type LayoutCurrentContext,
} from './zui-layout-paths';
import { acquireLayoutCatalogLock } from './zui-layout-catalog-lock';
import { requireApprovedPenpotArtifactDirectory } from './zui-layout-artifact-path';
import type { CatalogEntry, CatalogManifest } from './zui-layout-catalog';

test('current review uses explicit source/output roles and preserves legacy callers', async (t) => {
  const temporaryRoot = process.env['ZUI_LAYOUT_TEST_ARTIFACT_ROOT'];
  assert.ok(temporaryRoot, 'Root must register an existing ZUI_LAYOUT_TEST_ARTIFACT_ROOT');
  requireApprovedPenpotArtifactDirectory(temporaryRoot, 'Layout test TEMP');
  const fixture = await mkdtemp(resolve(temporaryRoot, 'layout-current-review-'));
  const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../../../../..');
  const outputRoot = resolve(fixture, 'output');
  const outside = resolve(fixture, 'outside');
  await mkdir(outputRoot);
  await mkdir(outside);
  const keys = ['ZUI_LAYOUT_REPO_ROOT', 'ZUI_LAYOUT_OUTPUT_ROOT', 'ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'] as const;
  const previous = Object.fromEntries(keys.map((key) => [key, process.env[key]]));
  process.env['ZUI_LAYOUT_REPO_ROOT'] = repoRoot;
  process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = outputRoot;
  delete process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'];
  try {
    // The renderer captures these same roots at import. Never switch them concurrently.
    const { runCurrentLayoutReview } = await import('./zui-layout-review');
    const { bytesSha256, dependenciesSha256, verifyCurrentFiles } = await import('./zui-layout-evidence');
    const context = createLayoutCurrentContext();
    const sourcePath = 'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-paths.ts';
    const sourceSha256 = bytesSha256(await readFile(resolve(repoRoot, sourcePath)));
    const outputBytes = 'synthetic prepared source; no passed capture\n';
    await writeFile(resolve(outputRoot, 'external.zui'), outputBytes);
    const entry: CatalogEntry = {
      sourcePath, sourceSha256,
      outputPath: 'external.zui', outputSha256: bytesSha256(outputBytes),
      resultPath: 'result.md', previewPath: 'preview.png',
      category: 'test', name: 'current-role', sourceClassification: 'test-fixture',
      assetRoot: 'tools/penpot', componentDependencies: [], resourceDependencies: [],
      sourceKind: 'fixture', sourceVersion: 1, outputKind: 'fixture', outputVersion: 1,
      sourceFormat: 'v2', status: 'failed', nodeCount: 0, projectedShapeCount: 0,
      prefabNodeCount: 0, prefabCoverage: null, prefabRoleCounts: {}, diagnostics: [],
      changes: [], visualStatus: 'failed', dependencyFingerprints: [],
      dependencySha256: dependenciesSha256([]), cases: [],
      penpotEvidence: [{ caseId: 'failed-fixture', status: 'failed', error: 'Synthetic capture failure' }],
    } as CatalogEntry;
    const manifest: CatalogManifest = {
      schema: 'dev.zircon.zui.layout-catalog', version: 1, generatedAt: 'fixture',
      repoRoot, sourceCount: 1, entries: [entry],
    };
    const input = { sourcePath, decision: 'needs_revision' as const, observations: 'Synthetic failed capture remains unaccepted.' };
    const save = async (value: unknown = manifest) =>
      writeFile(resolve(outputRoot, 'catalog.json'), JSON.stringify(value));
    const assertRejectedWithoutWrites = async (value: unknown, expected: RegExp) => {
      await save(value);
      const before = await readFile(resolve(outputRoot, 'catalog.json'), 'utf8');
      const names = await readdir(outputRoot);
      await assert.rejects(runCurrentLayoutReview([input]), expected);
      assert.equal(await readFile(resolve(outputRoot, 'catalog.json'), 'utf8'), before);
      assert.deepEqual(await readdir(outputRoot), names);
    };

    await t.test('root discovery does not need docs/_data and external ZUI is never rewritten', () => {
      assertLayoutCurrentRoots(context, repoRoot, outputRoot);
      assert.equal(layoutCurrentOutputPath(context, 'external.zui'), resolve(outputRoot, 'external.zui'));
      assert.equal(layoutCurrentSourcePath(context, sourcePath), resolve(repoRoot, sourcePath));
      assert.throws(() => layoutCurrentSourcePath(context, resolve(outside, 'external.zui')), /escapes/);
      assert.throws(() => layoutCurrentSourcePath(context, 'docs/_data/layout/external.zui'), /Historical/);
      assert.throws(() => layoutCurrentOutputPath(context, '../outside/external.zui'), /escapes/);
      assert.throws(() => createLayoutCurrentContext({ repoRoot: outside, outputRoot }), /source markers/);
      assert.throws(() => createLayoutCurrentContext({ repoRoot, outputRoot: repoRoot }), /cargo-targets|overlap/);
    });

    await t.test('historical and forged package contexts are rejected', () => {
      process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'] = fixture;
      try {
        assert.throws(() => createLayoutCurrentContext(), /historical archive/);
      } finally {
        delete process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'];
      }
      assert.throws(() => assertLayoutCurrentRoots(
        { ...context, role: 'package' } as unknown as LayoutCurrentContext, repoRoot, outputRoot,
      ), /Historical or package/);
    });

    await t.test('legacy exports retain their existing mapping', () => {
      const legacy = resolve(fixture, 'docs/_data/layout/example');
      const source = resolve(fixture, 'docs/ui/zui/example');
      assert.equal(layoutSourceRoot(legacy), source);
      assert.equal(resolveLayoutCatalogPath(legacy, 'external.zui'), resolve(source, 'external.zui'));
      assert.equal(resolveLayoutCatalogPath(legacy, 'catalog.json'), resolve(legacy, 'catalog.json'));
    });

    await t.test('missing explicit output fails before lock/read/write', async () => {
      await save();
      const names = await readdir(outputRoot);
      delete process.env['ZUI_LAYOUT_OUTPUT_ROOT'];
      try {
        await assert.rejects(runCurrentLayoutReview([input]), /explicit ZUI_LAYOUT_OUTPUT_ROOT/);
      } finally {
        process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = outputRoot;
      }
      assert.deepEqual(await readdir(outputRoot), names);
      process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = outside;
      try {
        await assert.rejects(runCurrentLayoutReview([input]), /environment changed after renderer initialization/);
      } finally {
        process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = outputRoot;
      }
      assert.deepEqual(await readdir(outside), []);
    });

    await t.test('wrong manifest identity is rejected before creating a lock', async () => {
      await assertRejectedWithoutWrites({ ...manifest, repoRoot: outside }, /root identity differs/);
    });

    await t.test('historical passed and package manifests cannot become current reviews', async () => {
      await assertRejectedWithoutWrites({ ...manifest, historical_only: true }, /Historical or package/);
      await assertRejectedWithoutWrites({ ...manifest, role: 'package' }, /Historical or package/);
    });

    await t.test('traversal is rejected before creating a lock', async () => {
      await assertRejectedWithoutWrites(
        { ...manifest, entries: [{ ...entry, resultPath: '../outside/result.md' }] }, /escapes/,
      );
    });

    await t.test('legacy renderer rewrite is rejected before any current review writes', async () => {
      await assertRejectedWithoutWrites(
        { ...manifest, entries: [{ ...entry, outputPath: 'docs/_data/layout/external.zui' }] },
        /Legacy renderer would rewrite/,
      );
    });

    await t.test('physical output escape is rejected before creating a lock', async () => {
      const target = resolve(outside, 'external.zui');
      await writeFile(target, outputBytes);
      const link = resolve(outputRoot, 'escaping');
      // Directory junctions do not require Windows symbolic-link privileges.
      await symlink(outside, link, process.platform === 'win32' ? 'junction' : 'dir');
      try {
        await assertRejectedWithoutWrites(
          { ...manifest, entries: [{ ...entry, outputPath: 'escaping/external.zui' }] }, /physically escapes/,
        );
      } finally {
        await rm(link);
      }
      assert.equal(await readFile(target, 'utf8'), outputBytes);
    });

    await t.test('real review validates current files and records only needs_revision', async () => {
      await save();
      await runCurrentLayoutReview([input]);
      const recorded = JSON.parse(await readFile(resolve(outputRoot, 'catalog.json'), 'utf8')) as CatalogManifest;
      assert.equal(recorded.entries[0]!.review?.status, 'needs_revision');
      assert.equal(recorded.entries[0]!.review?.sourceSha256, sourceSha256);
      assert.match(await readFile(resolve(outputRoot, 'result.md'), 'utf8'), /Synthetic capture failure/);
      assert.ok((await readdir(outputRoot)).includes('index.md'));
      assert.equal((await readdir(outputRoot)).includes('.zui-layout-catalog.lock'), false);
      await assert.rejects(
        verifyCurrentFiles(entry, outside, outputRoot, false, context), /root identity differs/,
      );
      await assert.rejects(
        verifyCurrentFiles({ ...entry, outputSha256: '0'.repeat(64) }, repoRoot, outputRoot, false, context),
        /File changed after validation/,
      );
    });

    await t.test('whole batch validation prevents partial review writes', async () => {
      const secondSource = 'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-artifact-path.ts';
      const bad = { ...entry, sourcePath: secondSource, sourceSha256: '0'.repeat(64), resultPath: 'second-result.md' };
      const secondInput = { ...input, sourcePath: secondSource };
      await save({ ...manifest, sourceCount: 2, entries: [entry, bad] });
      const before = await readFile(resolve(outputRoot, 'catalog.json'), 'utf8');
      await assert.rejects(runCurrentLayoutReview([input, secondInput]), /File changed after validation/);
      assert.equal(await readFile(resolve(outputRoot, 'catalog.json'), 'utf8'), before);
      await assert.rejects(runCurrentLayoutReview([{ ...input, decision: 'accepted' }]), /Missing dual-renderer/);
      assert.equal(await readFile(resolve(outputRoot, 'catalog.json'), 'utf8'), before);
    });

    await t.test('current lock preserves contention and token ownership', async () => {
      const lock = await acquireLayoutCatalogLock(outputRoot, { command: 'fixture', token: 'original', context });
      await assert.rejects(acquireLayoutCatalogLock(outputRoot, { command: 'contender', context }), /already held/);
      const receipt = resolve(lock.path, 'owner.json');
      await writeFile(receipt, JSON.stringify({ ...lock.owner, token: 'foreign' }));
      await assert.rejects(lock.release(), /no longer belongs/);
      assert.equal(JSON.parse(await readFile(receipt, 'utf8')).token, 'foreign');
      await writeFile(receipt, JSON.stringify(lock.owner));
      await lock.release();
      await lock.release();
    });

    await t.test('context-free lock callers retain their original behavior', async () => {
      const legacyRoot = resolve(fixture, 'legacy-created-by-lock');
      const lock = await acquireLayoutCatalogLock(legacyRoot, { command: 'legacy' });
      assert.equal(lock.path, resolve(legacyRoot, '.zui-layout-catalog.lock'));
      await lock.release();
    });
  } finally {
    for (const key of keys) {
      const value = previous[key];
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
    await rm(fixture, { recursive: true, force: true });
  }
});
