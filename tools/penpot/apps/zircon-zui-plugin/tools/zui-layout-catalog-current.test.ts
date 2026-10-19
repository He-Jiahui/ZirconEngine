import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, readdir, rm, symlink, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { createLayoutCurrentContext, layoutCurrentOutputPath } from './zui-layout-paths';
import { requireApprovedPenpotArtifactDirectory } from './zui-layout-artifact-path';
import type { CatalogEntry, CatalogManifest } from './zui-layout-catalog';

test('actual current catalog producer separates checkout sources from artifact output', async (t) => {
  const artifactRoot = process.env['ZUI_LAYOUT_TEST_ARTIFACT_ROOT'];
  assert.ok(artifactRoot, 'Root must register an existing ZUI_LAYOUT_TEST_ARTIFACT_ROOT');
  requireApprovedPenpotArtifactDirectory(artifactRoot, 'Catalog test artifact root');
  const fixture = await mkdtemp(resolve(artifactRoot, 'layout-current-catalog-'));
  const outputRoot = resolve(fixture, 'output');
  const outside = resolve(fixture, 'outside');
  await mkdir(outputRoot);
  await mkdir(outside);
  const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../../../../..');
  const keys = ['ZUI_LAYOUT_REPO_ROOT', 'ZUI_LAYOUT_OUTPUT_ROOT', 'ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'] as const;
  const previous = Object.fromEntries(keys.map((key) => [key, process.env[key]]));
  process.env['ZUI_LAYOUT_REPO_ROOT'] = repoRoot;
  process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = outputRoot;
  delete process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'];
  try {
    // Import the actual producer after selecting the same roots its render helpers capture.
    const { runCurrentLayoutCatalog, renderResult, renderIndex } = await import('./zui-layout-catalog');
    const { retainExpandedCaseEvidence } = await import('./zui-layout-catalog-evidence');
    const { readWorkbenchCaseSnapshots, WORKBENCH_WINDOW_SOURCE } = await import('./zui-layout-workbench-cases');
    const { bytesSha256, currentDualReview } = await import('./zui-layout-evidence');
    const { caseSha256, casesSha256 } = await import('./zui-layout-review-contract');
    const { runCurrentLayoutReview } = await import('./zui-layout-review');
    const context = createLayoutCurrentContext();
    const sourcePath = 'tools/penpot/apps/zircon-zui-plugin/src/bridge/roundtrip-fixture.zui';
    const sourceBytes = await readFile(resolve(repoRoot, sourcePath));
    const sourceHash = bytesSha256(sourceBytes);
    const args = ['--source', sourcePath];
    const manifestFile = resolve(outputRoot, 'catalog.json');
    const readManifest = async (): Promise<CatalogManifest> =>
      JSON.parse(await readFile(manifestFile, 'utf8')) as CatalogManifest;
    const save = async (value: unknown) => writeFile(manifestFile, JSON.stringify(value));
    const snapshot = async (root: string): Promise<Record<string, string>> => {
      const result: Record<string, string> = {};
      const visit = async (directory: string, prefix = ''): Promise<void> => {
        for (const item of await readdir(directory, { withFileTypes: true })) {
          const key = prefix + item.name;
          const path = resolve(directory, item.name);
          if (item.isDirectory()) {
            result[key + '/'] = 'directory';
            await visit(path, key + '/');
          } else {
            result[key] = bytesSha256(await readFile(path));
          }
        }
      };
      await visit(root);
      return result;
    };
    let generated: CatalogManifest;
    let entry: CatalogEntry;

    await t.test('real selected-source producer writes prepared files with current source SHA', async () => {
      await runCurrentLayoutCatalog(args);
      generated = await readManifest();
      assert.equal(generated.repoRoot, repoRoot);
      assert.equal(generated.sourceCount, 1);
      entry = generated.entries[0]!;
      assert.equal(entry.sourcePath, sourcePath);
      assert.equal(entry.sourceSha256, sourceHash);
      assert.equal(entry.status, 'prepared', JSON.stringify(entry.diagnostics));
      assert.ok(entry.penpotInputPath);
      assert.ok(entry.cases?.length);
      assert.equal(bytesSha256(await readFile(layoutCurrentOutputPath(context, entry.outputPath))), entry.outputSha256);
      assert.deepEqual(await readFile(layoutCurrentOutputPath(context, entry.outputPath)), sourceBytes);
      assert.equal(bytesSha256(await readFile(layoutCurrentOutputPath(context, entry.penpotInputPath!))), entry.penpotInputSha256);
      assert.ok((await readFile(layoutCurrentOutputPath(context, entry.resultPath), 'utf8')).includes('Layout Review'));
      assert.equal(bytesSha256(await readFile(resolve(repoRoot, sourcePath))), sourceHash);
      assert.notEqual(entry.visualStatus, 'passed');
      assert.equal(entry.review, undefined);
      assert.equal((await readdir(outputRoot)).includes('.zui-layout-catalog.lock'), false);
    });

    await t.test('--check leaves every output file and directory unchanged', async () => {
      const before = await snapshot(outputRoot);
      await runCurrentLayoutCatalog([...args, '--check']);
      assert.deepEqual(await snapshot(outputRoot), before);
      assert.equal(bytesSha256(await readFile(resolve(repoRoot, sourcePath))), sourceHash);
    });

    await t.test('missing explicit output fails before producer output or lock changes', async () => {
      const before = await snapshot(outputRoot);
      delete process.env['ZUI_LAYOUT_OUTPUT_ROOT'];
      try {
        await assert.rejects(runCurrentLayoutCatalog(args), /explicit ZUI_LAYOUT_OUTPUT_ROOT/);
      } finally {
        process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = outputRoot;
      }
      assert.deepEqual(await snapshot(outputRoot), before);
    });

    const assertRejectedWithoutWrites = async (manifest: unknown, expected: RegExp) => {
      await save(manifest);
      const before = await snapshot(outputRoot);
      await assert.rejects(runCurrentLayoutCatalog(['--refresh-records']), expected);
      assert.deepEqual(await snapshot(outputRoot), before);
    };

    await t.test('wrong physical identity and historical/package manifests cannot seed current output', async () => {
      await assertRejectedWithoutWrites({ ...generated, repoRoot: outside }, /root identity differs/);
      await assertRejectedWithoutWrites({ ...generated, historical_only: true }, /Historical or package/);
      await assertRejectedWithoutWrites({ ...generated, role: 'package' }, /Historical or package/);
    });

    await t.test('retained traversal and physical escape fail before the writer lock', async () => {
      await assertRejectedWithoutWrites(
        { ...generated, entries: [{ ...entry, resultPath: '../outside/result.md' }] }, /escapes/,
      );
      await writeFile(resolve(outside, 'external.zui'), sourceBytes);
      const link = resolve(outputRoot, 'escaping');
      await symlink(outside, link, process.platform === 'win32' ? 'junction' : 'dir');
      try {
        await save({ ...generated, entries: [{ ...entry, outputPath: 'escaping/external.zui' }] });
        const before = await readFile(manifestFile, 'utf8');
        await assert.rejects(runCurrentLayoutCatalog(['--refresh-records']), /physically escapes/);
        assert.equal(await readFile(manifestFile, 'utf8'), before);
        assert.equal((await readdir(outputRoot)).includes('.zui-layout-catalog.lock'), false);
        assert.deepEqual(await readFile(resolve(outside, 'external.zui')), sourceBytes);
      } finally {
        await rm(link);
      }
    });

    await t.test('context renderers keep ZUI links in output without legacy extension rewriting', () => {
      const nested = { ...entry, outputPath: 'docs/_data/layout/external.zui', resultPath: 'result.md' };
      assert.ok(renderResult(nested, context).includes('(docs/_data/layout/external.zui)'));
      assert.ok(renderIndex({ ...generated, entries: [nested] }, context).includes('(docs/_data/layout/external.zui)'));
    });

    await t.test('focused regeneration retains unselected records and failed review history', async () => {
      const selected = structuredClone(entry);
      selected.visualStatus = 'failed';
      selected.penpotEvidence = [{
        caseId: selected.cases![0]!.id, status: 'failed',
        error: 'Synthetic failed capture remains failed',
      }] as CatalogEntry['penpotEvidence'];
      const unselected = structuredClone(entry);
      unselected.sourcePath = WORKBENCH_WINDOW_SOURCE;
      unselected.sourceSha256 = bytesSha256(await readFile(resolve(repoRoot, WORKBENCH_WINDOW_SOURCE)));
      unselected.name = 'unselected-current-record';
      unselected.reviewHistory = [];
      await save({ ...generated, sourceCount: 2, entries: [selected, unselected] });

      // A current needs_revision review survives an unchanged producer refresh.
      await runCurrentLayoutReview([{
        sourcePath, decision: 'needs_revision', observations: 'First current failure must remain current',
      }]);
      const first = (await readManifest()).entries.find((item) => item.sourcePath === sourcePath)!;
      assert.ok(first.review);
      const firstReview = structuredClone(first.review);
      assert.equal(firstReview.status, 'needs_revision');
      assert.equal(currentDualReview(first), true);
      assert.deepEqual(first.reviewHistory, []);
      await runCurrentLayoutCatalog(args);
      const current = await readManifest();
      assert.equal(current.sourceCount, 2);
      assert.deepEqual(current.entries.find((item) => item.sourcePath === WORKBENCH_WINDOW_SOURCE), unselected);
      const retained = current.entries.find((item) => item.sourcePath === sourcePath)!;
      assert.deepEqual(retained.review, firstReview);
      assert.deepEqual(retained.reviewHistory, []);
      assert.equal(currentDualReview(retained), true);
      assert.equal(retained.visualStatus, 'failed');
      assert.equal(retained.review?.status === 'accepted', false);
      assert.equal(bytesSha256(await readFile(resolve(repoRoot, sourcePath))), sourceHash);

      // Model a prior case-matrix version using only the owned catalog fixture.
      // No authored source or synthetic passed capture is introduced.
      const canonicalCases = structuredClone(retained.cases!);
      const canonicalCasesHash = casesSha256(canonicalCases);
      const retiredCase = { ...canonicalCases[0]!, id: 'fixture-retired-review-matrix' };
      assert.equal(canonicalCases.some((item) => item.id === retiredCase.id), false);
      retained.cases = [...canonicalCases, retiredCase];
      assert.notEqual(casesSha256(retained.cases), canonicalCasesHash);
      assert.equal(currentDualReview(retained), false);
      await save(current);

      // A second actual review supersedes the first and binds the prior matrix.
      await runCurrentLayoutReview([{
        sourcePath, decision: 'needs_revision', observations: 'Synthetic failure history must survive',
      }]);
      const secondManifest = await readManifest();
      const second = secondManifest.entries.find((item) => item.sourcePath === sourcePath)!;
      assert.ok(second.review);
      const secondReview = structuredClone(second.review);
      assert.equal(secondReview.status, 'needs_revision');
      assert.equal(secondReview.observations, 'Synthetic failure history must survive');
      assert.equal(secondReview.casesSha256, casesSha256(second.cases!));
      assert.notEqual(secondReview.casesSha256, canonicalCasesHash);
      assert.equal(currentDualReview(second), true);
      assert.deepEqual(second.reviewHistory, [firstReview]);
      assert.equal(second.visualStatus, 'failed');
      assert.equal(second.review?.status === 'accepted', false);
      assert.deepEqual(secondManifest.entries.find((item) => item.sourcePath === WORKBENCH_WINDOW_SOURCE), unselected);

      // Regenerating the actual canonical matrix invalidates the second review.
      await runCurrentLayoutCatalog(args);
      const next = await readManifest();
      assert.equal(next.sourceCount, 2);
      assert.deepEqual(next.entries.find((item) => item.sourcePath === WORKBENCH_WINDOW_SOURCE), unselected);
      const refreshed = next.entries.find((item) => item.sourcePath === sourcePath)!;
      assert.deepEqual(refreshed.cases, canonicalCases);
      assert.equal(casesSha256(refreshed.cases!), canonicalCasesHash);
      assert.notEqual(casesSha256(refreshed.cases!), secondReview.casesSha256);
      assert.equal(refreshed.review, undefined);
      assert.equal(currentDualReview(refreshed), false);
      assert.deepEqual(refreshed.reviewHistory, [firstReview, secondReview]);
      assert.ok(refreshed.reviewHistory?.some((item) => item.observations === 'Synthetic failure history must survive'));
      assert.equal(refreshed.reviewHistory?.filter((item) => item.observations === firstReview.observations).length, 1);
      assert.equal(refreshed.reviewHistory?.filter((item) => item.observations === secondReview.observations).length, 1);
      assert.notEqual(refreshed.visualStatus, 'passed');
      assert.equal(refreshed.review?.status === 'accepted', false);
      assert.equal(bytesSha256(await readFile(resolve(repoRoot, sourcePath))), sourceHash);
    });

    await t.test('retention validates all destination collisions and hashes before the first rename', async () => {
      const directory = resolve(outputRoot, 'retention/fixture');
      await mkdir(resolve(directory, 'evidence'), { recursive: true });
      const bytes = Buffer.from('synthetic pending evidence, never a passed capture');
      const hash = bytesSha256(bytes);
      const oldCase = { ...entry.cases![0]!, id: 'fixture-old' };
      const nextCase = { ...oldCase, id: 'fixture-next' };
      const base = {
        ...entry, category: 'retention', name: 'fixture', cases: [oldCase, nextCase],
        penpotPreviewPath: 'retention/fixture/penpot.png', previewPath: 'retention/fixture/preview.png',
      };
      const evidence = {
        caseId: oldCase.id, status: 'pending' as const, pendingReason: 'Synthetic retention fixture',
        sourceSha256: base.sourceSha256, dependencySha256: base.dependencySha256!,
        caseSha256: caseSha256(oldCase), screenshotSha256: hash,
      };
      const previousEntry = {
        ...base,
        penpotEvidence: [{ ...evidence, inputSha256: base.penpotInputSha256, screenshotPath: base.penpotPreviewPath }],
        engineEvidence: [{ ...evidence, inputSha256: base.sourceSha256, screenshotPath: base.previewPath }],
      };
      const nextEntry = { ...base, cases: [nextCase, oldCase] };
      const penpot = resolve(outputRoot, base.penpotPreviewPath);
      const engine = resolve(outputRoot, base.previewPath);
      const penpotArchive = resolve(directory, 'evidence', 'former-primary-penpot-fixture-old-' + hash.slice(0, 12) + '.png');
      const engineArchive = resolve(directory, 'evidence', 'former-primary-engine-fixture-old-' + hash.slice(0, 12) + '.png');
      await writeFile(penpot, bytes);
      await writeFile(engine, bytes);
      await writeFile(engineArchive, 'existing destination must not be overwritten');
      await assert.rejects(
        retainExpandedCaseEvidence(previousEntry, nextEntry, outputRoot, true, context), /archive already exists/,
      );
      assert.deepEqual(await readFile(penpot), bytes);
      assert.deepEqual(await readFile(engine), bytes);
      await assert.rejects(readFile(penpotArchive), { code: 'ENOENT' });
      await rm(engineArchive);
      await writeFile(engine, 'corrupted source evidence');
      await assert.rejects(
        retainExpandedCaseEvidence(previousEntry, nextEntry, outputRoot, true, context), /hash differs/,
      );
      assert.deepEqual(await readFile(penpot), bytes);
      await writeFile(engine, bytes);
      await retainExpandedCaseEvidence(previousEntry, nextEntry, outputRoot, false, context);
      assert.deepEqual(await readFile(penpot), bytes);
      await assert.rejects(readFile(penpotArchive), { code: 'ENOENT' });
      const retained = await retainExpandedCaseEvidence(previousEntry, nextEntry, outputRoot, true, context);
      assert.equal(retained.penpotEvidence![0]!.status, 'pending');
      assert.deepEqual(await readFile(penpotArchive), bytes);
      assert.deepEqual(await readFile(engineArchive), bytes);
      await assert.rejects(readFile(penpot), { code: 'ENOENT' });
      await assert.rejects(readFile(engine), { code: 'ENOENT' });
    });

    await t.test('missing current Workbench snapshot fails at output with no historical fallback', async () => {
      const archive = resolve(fixture, 'historical-negative-input');
      const oldDirectory = resolve(archive, 'docs/_data/layout');
      await mkdir(oldDirectory, { recursive: true });
      await writeFile(resolve(oldDirectory, 'workbench-product-state.json'), 'sentinel: never consume this historical input');
      process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'] = archive;
      try {
        await assert.rejects(readWorkbenchCaseSnapshots(repoRoot, context), (error: unknown) => {
          const failure = error as NodeJS.ErrnoException;
          assert.equal(failure.code, 'ENOENT');
          assert.equal(failure.path, resolve(outputRoot, 'workbench-product-state.json'));
          return true;
        });
      } finally {
        delete process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'];
      }
      assert.equal(bytesSha256(await readFile(resolve(repoRoot, sourcePath))), sourceHash);
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
