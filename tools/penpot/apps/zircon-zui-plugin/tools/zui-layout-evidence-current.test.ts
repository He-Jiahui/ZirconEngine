import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, readdir, readlink, rm, symlink, writeFile } from 'node:fs/promises';
import { dirname, isAbsolute, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import type { CatalogEntry, CatalogManifest } from './zui-layout-catalog';
import { createLayoutCurrentContext } from './zui-layout-paths';
import { requireApprovedPenpotArtifactDirectory } from './zui-layout-artifact-path';

test('current capture program verifies exact checkout sources and one approved bundle', async (t) => {
  const artifactRoot = process.env['ZUI_LAYOUT_TEST_ARTIFACT_ROOT'];
  assert.ok(artifactRoot, 'Root must register an existing ZUI_LAYOUT_TEST_ARTIFACT_ROOT');
  requireApprovedPenpotArtifactDirectory(artifactRoot, 'Capture program test artifact root');
  const fixture = await mkdtemp(resolve(artifactRoot, 'layout-current-program-'));
  const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../../../../..');
  const outputRoot = resolve(fixture, 'output');
  const stagingRoot = resolve(fixture, 'staging');
  const archiveRoot = resolve(fixture, 'synthetic-history');
  await mkdir(outputRoot);
  await mkdir(stagingRoot);
  await mkdir(archiveRoot);
  // Controlled bytes exercise artifact routing only. They are not a built product,
  // renderer output, screenshot, passed capture, or acceptance evidence.
  const bundle = Buffer.from('synthetic stopped renderer fixture; never passed or accepted\n');
  const bundlePath = resolve(stagingRoot, 'plugin.js');
  await writeFile(bundlePath, bundle);
  const keys = ['ZUI_LAYOUT_REPO_ROOT', 'ZUI_LAYOUT_OUTPUT_ROOT', 'ZUI_PLUGIN_BUNDLE_PATH', 'ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'] as const;
  const previous = Object.fromEntries(keys.map((key) => [key, process.env[key]]));
  process.env['ZUI_LAYOUT_REPO_ROOT'] = repoRoot;
  process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = outputRoot;
  process.env['ZUI_PLUGIN_BUNDLE_PATH'] = bundlePath;
  delete process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'];
  try {
    const { runCurrentLayoutCatalog } = await import('./zui-layout-catalog');
    const { runCurrentLayoutReview } = await import('./zui-layout-review');
    const { bytesSha256, currentDualReview, verifyCurrentFiles } = await import('./zui-layout-evidence');
    const { captureProgramFingerprints } = await import('./zui-layout-capture-provenance');
    const { caseSha256 } = await import('./zui-layout-review-contract');
    const context = createLayoutCurrentContext();
    const sourcePath = 'tools/penpot/apps/zircon-zui-plugin/src/bridge/roundtrip-fixture.zui';
    const sourceHash = bytesSha256(await readFile(resolve(repoRoot, sourcePath)));
    const manifestFile = resolve(outputRoot, 'catalog.json');
    const readManifest = async (): Promise<CatalogManifest> =>
      JSON.parse(await readFile(manifestFile, 'utf8')) as CatalogManifest;
    const save = async (value: unknown) => writeFile(manifestFile, JSON.stringify(value));
    const snapshot = async (root: string): Promise<Record<string, string>> => {
      const files: Record<string, string> = {};
      const visit = async (directory: string, prefix = ''): Promise<void> => {
        for (const item of await readdir(directory, { withFileTypes: true })) {
          const key = prefix + item.name;
          const path = resolve(directory, item.name);
          if (item.isSymbolicLink()) files[key] = 'link:' + await readlink(path);
          else if (item.isDirectory()) {
            files[key + '/'] = 'directory';
            await visit(path, key + '/');
          } else files[key] = bytesSha256(await readFile(path));
        }
      };
      await visit(root);
      return files;
    };
    let manifest: CatalogManifest;
    let entry: CatalogEntry;
    let program: Array<[string, string]>;
    const review = {
      sourcePath, decision: 'needs_revision' as const,
      observations: 'Synthetic failed renderer fixture; no actual capture or acceptance claim.',
    };
    const rejectWithoutWrites = async (candidate: CatalogEntry, expected: RegExp) => {
      await save({ ...manifest, entries: [candidate] });
      const before = await snapshot(outputRoot);
      await assert.rejects(runCurrentLayoutReview([review]), expected);
      assert.deepEqual(await snapshot(outputRoot), before);
      const current = (await readManifest()).entries[0]!;
      assert.equal(current.review?.status === 'accepted', false);
    };
    const fresh = () => structuredClone(entry);

    await t.test('actual producer and inventory use unchanged real checkout files', async () => {
      await runCurrentLayoutCatalog(['--source', sourcePath]);
      manifest = await readManifest();
      entry = structuredClone(manifest.entries[0]!);
      assert.equal(entry.status, 'prepared', JSON.stringify(entry.diagnostics));
      assert.equal(entry.sourceSha256, sourceHash);
      program = await captureProgramFingerprints(repoRoot);
      assert.equal(new Set(program.map(([path]) => path)).size, program.length);
      assert.deepEqual(program.filter(([path]) => isAbsolute(path)), [[bundlePath, bytesSha256(bundle)]]);
      for (const [path, hash] of program.filter(([path]) => !isAbsolute(path)))
        assert.equal(bytesSha256(await readFile(resolve(repoRoot, path))), hash);
      entry.visualStatus = 'failed';
      entry.reviewHistory = [];
      delete entry.review;
      entry.penpotEvidence = [{
        caseId: entry.cases![0]!.id, status: 'failed',
        sourceSha256: entry.sourceSha256, inputSha256: entry.penpotInputSha256,
        dependencySha256: entry.dependencySha256,
        caseSha256: caseSha256(entry.cases![0]!),
        rendererKind: 'penpot-official-frontend',
        rendererPath: bundlePath, rendererSha256: bytesSha256(bundle),
        captureProgramFingerprints: structuredClone(program),
        error: 'Synthetic renderer never started; complete program identity only.',
      }];
      delete entry.engineEvidence;
      assert.equal(entry.review?.status === 'accepted', false);
    });

    await t.test('actual needs_revision review verifies full program and retains source order independence', async () => {
      const candidate = fresh();
      candidate.penpotEvidence![0]!.captureProgramFingerprints!.reverse();
      await save({ ...manifest, entries: [candidate] });
      await runCurrentLayoutReview([review]);
      const current = (await readManifest()).entries[0]!;
      assert.equal(current.review?.status, 'needs_revision');
      assert.equal(current.review?.observations, review.observations);
      assert.deepEqual(current.reviewHistory, []);
      assert.equal(currentDualReview(current), true);
      assert.deepEqual(current.penpotEvidence![0]!.captureProgramFingerprints, [...program].reverse());
      assert.equal(current.penpotEvidence![0]!.status, 'failed');
      assert.equal(current.visualStatus, 'failed');
      assert.equal(current.review?.status === 'accepted', false);
      assert.equal(bytesSha256(await readFile(resolve(repoRoot, sourcePath))), sourceHash);
    });

    await t.test('missing extra duplicate substituted and stale source identities cannot be reviewed', async () => {
      const mutations: Array<(items: Array<[string, string]>) => void> = [
        (items) => { items.shift(); },
        (items) => { items.push([sourcePath, sourceHash]); },
        (items) => { items.push([...items[0]!] as [string, string]); },
        (items) => { items[0] = [sourcePath, sourceHash]; },
        (items) => { items[0] = [items[0]![0], '0'.repeat(64)]; },
        (items) => { items[0] = ['../unregistered-program.ts', items[0]![1]]; },
      ];
      for (const mutate of mutations) {
        const candidate = fresh();
        mutate(candidate.penpotEvidence![0]!.captureProgramFingerprints!);
        await rejectWithoutWrites(candidate, /Current Penpot capture program inventory differs|Layout path escapes role root/);
      }
    });

    await t.test('same-byte alternate bundle and arbitrary absolute path are not equivalent identities', async () => {
      const alternate = resolve(stagingRoot, 'other-plugin.js');
      await writeFile(alternate, bundle);
      for (const path of [alternate, 'C:/unregistered-capture-program/plugin.js']) {
        const candidate = fresh();
        candidate.penpotEvidence![0]!.captureProgramFingerprints =
          program.map(([key, hash]) => [key === bundlePath ? path : key, hash]);
        await rejectWithoutWrites(candidate, /Current Penpot capture program inventory differs/);
      }
    });

    await t.test('equal-length bundle mutation rejects old SHA without catalog or review writes', async () => {
      const changed = Buffer.from(bundle);
      changed[0] = changed[0]! ^ 1;
      assert.equal(changed.length, bundle.length);
      assert.notEqual(bytesSha256(changed), bytesSha256(bundle));
      await writeFile(bundlePath, changed);
      try {
        await rejectWithoutWrites(fresh(), /Current Penpot capture program inventory differs/);
      } finally {
        await writeFile(bundlePath, bundle);
      }
    });

    await t.test('renderer path and SHA must bind the same unique approved program bundle', async () => {
      const wrongPath = fresh();
      wrongPath.penpotEvidence![0]!.rendererPath = resolve(stagingRoot, 'other-plugin.js');
      await rejectWithoutWrites(wrongPath, /renderer differs from the approved capture bundle/);
      const wrongHash = fresh();
      wrongHash.penpotEvidence![0]!.rendererSha256 = '0'.repeat(64);
      await rejectWithoutWrites(wrongHash, /renderer differs from the approved capture bundle/);
    });

    await t.test('configured archive bundle and physical archive alias cannot seed current review', async () => {
      const archiveBundle = resolve(archiveRoot, 'plugin.js');
      await writeFile(archiveBundle, bundle);
      const alias = resolve(fixture, 'history-alias');
      await symlink(archiveRoot, alias, 'junction');
      process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'] = archiveRoot;
      try {
        for (const path of [archiveBundle, resolve(alias, 'plugin.js')]) {
          process.env['ZUI_PLUGIN_BUNDLE_PATH'] = path;
          const candidate = fresh();
          const capture = candidate.penpotEvidence![0]!;
          capture.rendererPath = path;
          capture.captureProgramFingerprints = program.map(([key, hash]) => [key === bundlePath ? path : key, hash]);
          await rejectWithoutWrites(candidate, /must not come from historical archive/);
        }
      } finally {
        process.env['ZUI_PLUGIN_BUNDLE_PATH'] = bundlePath;
        delete process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'];
        await rm(alias);
      }
    });

    await t.test('lexically approved bundle alias escaping physical cargo storage is rejected', async () => {
      const alias = resolve(fixture, 'source-alias');
      await symlink(repoRoot, alias, 'junction');
      try {
        const aliasPath = resolve(alias, 'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-paths.ts');
        process.env['ZUI_PLUGIN_BUNDLE_PATH'] = aliasPath;
        const candidate = fresh();
        const capture = candidate.penpotEvidence![0]!;
        capture.rendererPath = aliasPath;
        capture.rendererSha256 = bytesSha256(await readFile(resolve(repoRoot, 'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-paths.ts')));
        capture.captureProgramFingerprints = program.map(([key, hash]) =>
          key === bundlePath ? [aliasPath, capture.rendererSha256!] : [key, hash]);
        await rejectWithoutWrites(candidate, /outside its approved physical artifact root/);
      } finally {
        process.env['ZUI_PLUGIN_BUNDLE_PATH'] = bundlePath;
        await rm(alias);
      }
      assert.equal(bytesSha256(await readFile(resolve(repoRoot, sourcePath))), sourceHash);
    });

    await t.test('engine records do not gain an arbitrary absolute program bypass', async () => {
      const candidate = fresh();
      candidate.engineEvidence = structuredClone(candidate.penpotEvidence);
      candidate.penpotEvidence = [];
      await rejectWithoutWrites(candidate, /Unrecognized absolute current capture program artifact/);
    });

    await t.test('context-free verifier keeps its existing relative program contract', async () => {
      const legacy = fresh();
      legacy.penpotEvidence![0]!.captureProgramFingerprints = [structuredClone(program[0]!)];
      await verifyCurrentFiles(legacy, repoRoot, outputRoot, false);
      assert.equal(legacy.penpotEvidence![0]!.status, 'failed');
      assert.equal(legacy.review?.status === 'accepted', false);
      await assert.rejects(verifyCurrentFiles(legacy, repoRoot, outputRoot, false, context),
        /Current Penpot capture program inventory differs/);
      await save({ ...manifest, entries: [fresh()] });
      const before = await snapshot(outputRoot);
      for (const role of ['history', 'package']) {
        await save({ ...manifest, role, entries: [fresh()] });
        const rejected = await snapshot(outputRoot);
        await assert.rejects(runCurrentLayoutReview([review]), /Historical or package catalog/);
        assert.deepEqual(await snapshot(outputRoot), rejected);
      }
      await save({ ...manifest, entries: [fresh()] });
      assert.deepEqual(await snapshot(outputRoot), before);
    });
  } finally {
    for (const key of keys) {
      if (previous[key] === undefined) delete process.env[key];
      else process.env[key] = previous[key];
    }
    await rm(fixture, { recursive: true, force: true });
  }
});
