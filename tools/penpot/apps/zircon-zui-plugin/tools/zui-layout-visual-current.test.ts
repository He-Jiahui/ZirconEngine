import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import { realpathSync } from 'node:fs';
import { mkdtemp, mkdir, readFile, readdir, readlink, rm, symlink, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import type { Browser, Page, Response } from 'playwright';
import type { CatalogEntry, CatalogManifest } from './zui-layout-catalog';
import {
  createLayoutCurrentContext, layoutCurrentOutputPath,
} from './zui-layout-paths';
import { requireApprovedPenpotArtifactDirectory } from './zui-layout-artifact-path';

test('actual current visual entry and capture helpers keep source and output roles', async (t) => {
  const artifactRoot = process.env['ZUI_LAYOUT_TEST_ARTIFACT_ROOT'];
  assert.ok(artifactRoot, 'Root must register an existing ZUI_LAYOUT_TEST_ARTIFACT_ROOT');
  requireApprovedPenpotArtifactDirectory(artifactRoot, 'Visual test artifact root');
  const fixture = await mkdtemp(resolve(artifactRoot, 'layout-current-visual-'));
  const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../../../../..');
  const outputRoot = resolve(fixture, 'output');
  const outside = resolve(fixture, 'outside');
  const dist = resolve(fixture, 'plugin-dist');
  await mkdir(outputRoot);
  await mkdir(outside);
  await mkdir(resolve(dist, 'assets'), { recursive: true });
  // These bytes exercise bundle-identity routing only; they are never rendered or accepted.
  const bundle = Buffer.from('synthetic stopped-browser bundle; no renderer success\n');
  const bundlePath = resolve(fixture, 'plugin.js');
  await writeFile(bundlePath, bundle);
  await writeFile(resolve(dist, 'assets/plugin.js'), bundle);
  const keys = [
    'ZUI_LAYOUT_REPO_ROOT', 'ZUI_LAYOUT_OUTPUT_ROOT', 'ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT',
    'ZUI_PLUGIN_DIST_ROOT', 'ZUI_PLUGIN_BUNDLE_PATH',
  ] as const;
  const previous = Object.fromEntries(keys.map((key) => [key, process.env[key]]));
  process.env['ZUI_LAYOUT_REPO_ROOT'] = repoRoot;
  process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = outputRoot;
  process.env['ZUI_PLUGIN_DIST_ROOT'] = dist;
  process.env['ZUI_PLUGIN_BUNDLE_PATH'] = bundlePath;
  delete process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'];
  try {
    const { runCurrentLayoutCatalog } = await import('./zui-layout-catalog');
    const { bytesSha256, verifyCurrentFiles } = await import('./zui-layout-evidence');
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
    const beforeImport = await snapshot(outputRoot);
    const { runCurrentVisualValidation } = await import('./zui-layout-visual-validation');
    const { capturePenpotCases } = await import('./zui-layout-penpot-cases');
    const { openPenpotSession, recordPenpotSessionFailure } = await import('./zui-layout-penpot-session');
    const { collectFontResources } = await import('./zui-layout-font-resources');
    const { chromium } = await import('playwright');
    const stoppedBrowser = 'Synthetic browser boundary stop; no actual capture';
    let browserLaunches = 0;
    t.mock.method(chromium, 'launch', async () => {
      browserLaunches += 1;
      throw new Error(stoppedBrowser);
    });
    const context = createLayoutCurrentContext();
    const sourcePath = 'tools/penpot/apps/zircon-zui-plugin/src/bridge/roundtrip-fixture.zui';
    const sourceHash = bytesSha256(await readFile(resolve(repoRoot, sourcePath)));
    const args = ['--source', sourcePath];
    const manifestFile = resolve(outputRoot, 'catalog.json');
    const readManifest = async (): Promise<CatalogManifest> =>
      JSON.parse(await readFile(manifestFile, 'utf8')) as CatalogManifest;
    const save = async (value: unknown) => writeFile(manifestFile, JSON.stringify(value));
    let generated: CatalogManifest;
    let entry: CatalogEntry;
    const rejectWithoutWrites = async (value: unknown, expected: RegExp) => {
      await save(value);
      const before = await snapshot(outputRoot);
      const launches = browserLaunches;
      await assert.rejects(runCurrentVisualValidation(args), expected);
      assert.deepEqual(await snapshot(outputRoot), before);
      assert.equal(browserLaunches, launches);
    };

    await t.test('import is inert and actual producer supplies current checkout evidence', async () => {
      assert.deepEqual(await snapshot(outputRoot), beforeImport);
      assert.equal(browserLaunches, 0);
      await runCurrentLayoutCatalog(args);
      generated = await readManifest();
      entry = generated.entries[0]!;
      assert.equal(entry.status, 'prepared', JSON.stringify(entry.diagnostics));
      assert.equal(entry.sourcePath, sourcePath);
      assert.equal(entry.sourceSha256, sourceHash);
      assert.equal(entry.review?.status === 'accepted', false);
    });

    await t.test('actual entry rejects missing output before read lock or browser', async () => {
      const before = await snapshot(outputRoot);
      delete process.env['ZUI_LAYOUT_OUTPUT_ROOT'];
      try {
        await assert.rejects(runCurrentVisualValidation(args), /explicit ZUI_LAYOUT_OUTPUT_ROOT/);
      } finally {
        process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = outputRoot;
      }
      assert.deepEqual(await snapshot(outputRoot), before);
      assert.equal(browserLaunches, 0);
    });

    await t.test('historical package and mismatched manifests cannot seed current capture', async () => {
      for (const invalid of [
        { ...generated, historical_only: true },
        { ...generated, role: 'history' },
        { ...generated, role: 'package' },
        { ...generated, schema: 'historical-catalog' },
      ]) await rejectWithoutWrites(invalid, /Historical or package catalog/);
      await rejectWithoutWrites({ ...generated, repoRoot: outside }, /root identity differs/);
      const stale = structuredClone(generated);
      stale.entries[0]!.sourceSha256 = '0'.repeat(64);
      await rejectWithoutWrites(stale, /Catalog source changed since catalog generation/);
      await save(generated);
    });

    await t.test('traversal and physical escape fail before lock browser or output', async () => {
      const traversal = structuredClone(generated);
      traversal.entries[0]!.outputPath = '../escaped.zui';
      await rejectWithoutWrites(traversal, /escapes role root/);
      const derived = structuredClone(generated);
      derived.entries[0]!.category = '../../outside';
      await rejectWithoutWrites(derived, /escapes role root/);
      const escape = resolve(outputRoot, 'escape');
      await symlink(outside, escape, 'junction');
      try {
        const physical = structuredClone(generated);
        physical.entries[0]!.outputPath = 'escape/external.zui';
        await rejectWithoutWrites(physical, /physically escapes role root/);
        assert.deepEqual(await readdir(outside), []);
      } finally {
        await rm(escape);
      }
      await save(generated);
    });

    await t.test('actual check reaches browser boundary with unchanged current output and source', async () => {
      await save(generated);
      const before = await snapshot(outputRoot);
      await assert.rejects(runCurrentVisualValidation([...args, '--check']), new RegExp(stoppedBrowser));
      assert.equal(browserLaunches, 1);
      assert.deepEqual(await snapshot(outputRoot), before);
      assert.equal(bytesSha256(await readFile(resolve(repoRoot, sourcePath))), sourceHash);
      const current = await readManifest();
      assert.equal(current.entries[0]!.review?.status === 'accepted', false);
    });

    await t.test('actual write entry uses and releases current writer lock on browser failure', async () => {
      await save(generated);
      await assert.rejects(runCurrentVisualValidation(args), new RegExp(stoppedBrowser));
      assert.equal(browserLaunches, 2);
      assert.equal((await readdir(outputRoot)).includes('.zui-layout-catalog.lock'), false);
      const current = await readManifest();
      assert.equal(current.entries[0]!.review?.status === 'accepted', false);
      assert.equal(bytesSha256(await readFile(resolve(repoRoot, sourcePath))), sourceHash);
    });

    await t.test('cases enforce current containment before touching the browser', async () => {
      let pageTouches = 0;
      const options = {
        page: { context: () => { pageTouches += 1; throw new Error('Synthetic cases boundary stop'); } } as unknown as Page,
        pluginFrame: {} as never, statusPanel: {} as never, pluginModal: {} as never,
        previewBoardId: 'unrendered-fixture', entry: structuredClone(entry),
        writeEvidence: true, catalogRoot: outputRoot, currentContext: context,
        pluginBundleSha256: bytesSha256(bundle), captureProgramFingerprints: [],
        captureFontResources: async () => [],
        validateImage: async () => 'Synthetic unrendered fixture',
      };
      await assert.rejects(capturePenpotCases({ ...options, catalogRoot: outside }), /root identity differs/);
      const escaped = structuredClone(entry);
      escaped.penpotInputPath = '../escaped.zui';
      await assert.rejects(capturePenpotCases({ ...options, entry: escaped }), /escapes role root/);
      assert.equal(pageTouches, 0);
      await assert.rejects(capturePenpotCases(options), /Synthetic cases boundary stop/);
      assert.equal(pageTouches, 1);
    });

    await t.test('session current check failure writes nothing while legacy diagnostics remain', async () => {
      let screenshots = 0;
      let closes = 0;
      const makeBrowser = () => {
        const page = Object.assign(new EventEmitter(), {
          exposeFunction: async () => { throw new Error('Synthetic session startup failure'); },
          screenshot: async ({ path }: { path: string }) => {
            screenshots += 1;
            await writeFile(path, 'synthetic startup diagnostic; no render claim');
          },
          title: async () => '',
          locator: () => ({ innerText: async () => '' }),
          evaluate: async () => ({ readyState: 'synthetic-failure', scripts: [], bodyHtml: '' }),
          url: () => 'https://invalid.example/synthetic-failure',
        });
        return {
          newContext: async () => ({
            newPage: async () => page,
            close: async () => { closes += 1; },
          }),
        } as unknown as Browser;
      };
      const options = {
        repoRoot, catalogRoot: outputRoot, currentContext: context, writeEvidence: false,
        penpotBaseUrl: 'https://invalid.example', pluginBundleSha256: bytesSha256(bundle),
        pageErrors: [] as string[], unexpectedRpcs: new Set<string>(),
      };
      const before = await snapshot(outputRoot);
      await assert.rejects(openPenpotSession(makeBrowser(), { ...options, catalogRoot: outside }), /root identity differs/);
      assert.equal(closes, 0);
      await assert.rejects(openPenpotSession(makeBrowser(), options), /Synthetic session startup failure/);
      assert.equal(screenshots, 0);
      assert.equal(closes, 1);
      assert.deepEqual(await snapshot(outputRoot), before);
      // The optional-context migration preserves the existing context-free contract.
      const legacyRoot = resolve(fixture, 'legacy-session');
      await mkdir(legacyRoot);
      await assert.rejects(openPenpotSession(makeBrowser(), {
        ...options, currentContext: undefined, catalogRoot: legacyRoot,
      }), /Synthetic session startup failure/);
      assert.equal(screenshots, 1);
      assert.equal(closes, 2);
      assert.deepEqual((await readdir(legacyRoot)).sort(), [
        'penpot-session-startup-failure.json', 'penpot-session-startup-failure.png',
      ]);
      await assert.rejects(recordPenpotSessionFailure({} as never, outside, new Error('unreached'), context), /escapes role root/);
    });

    await t.test('font bytes URL and SHA remain output-contained and independently verified', async () => {
      const page = new EventEmitter();
      const collector = collectFontResources(page as unknown as Page, repoRoot, outputRoot, true, context);
      const bytes = Buffer.from('wOF2synthetic byte-routing fixture; not font-render acceptance');
      const url = 'https://invalid.example/fonts/synthetic.woff2';
      const response = {
        ok: () => true, status: () => 200, url: () => url, body: async () => bytes,
        request: () => ({ resourceType: () => 'font' }),
      } as unknown as Response;
      try {
        page.emit('response', response);
        const records = await collector.snapshotRecords();
        const path = realpathSync.native(layoutCurrentOutputPath(context, 'resources/fonts/' + bytesSha256(bytes) + '.woff2'));
        assert.deepEqual(records, [{ url, path, sha256: bytesSha256(bytes) }]);
        assert.deepEqual(await readFile(path), bytes);
        assert.deepEqual(JSON.parse(await readFile(resolve(outputRoot, 'resources/fonts/manifest.json'), 'utf8')), records);
        assert.deepEqual(await collector.snapshot(), [[path, bytesSha256(bytes)]]);
        const failed = structuredClone(entry);
        failed.penpotEvidence = [{
          caseId: entry.cases![0]!.id, status: 'failed', error: 'Synthetic uncaptured case',
          runtimeAssetFingerprints: [[path, bytesSha256(bytes)]],
        }];
        await verifyCurrentFiles(failed, repoRoot, outputRoot, false, context);
        await writeFile(path, 'tampered synthetic bytes');
        await assert.rejects(verifyCurrentFiles(failed, repoRoot, outputRoot, false, context), /File changed after validation/);
        await writeFile(path, bytes);
        assert.equal(failed.review?.status === 'accepted', false);
      } finally {
        collector.stop();
      }
      assert.equal(page.listenerCount('response'), 0);
      assert.equal(bytesSha256(await readFile(resolve(repoRoot, sourcePath))), sourceHash);
    });

    await t.test('font failures mismatched roots and physical escape never produce valid records', async () => {
      const page = new EventEmitter();
      assert.throws(() => collectFontResources(page as unknown as Page, repoRoot, outside, true, context), /root identity differs/);
      assert.equal(page.listenerCount('response'), 0);
      const collector = collectFontResources(page as unknown as Page, repoRoot, outputRoot, true, context);
      const before = await snapshot(outputRoot);
      try {
        page.emit('response', {
          ok: () => false, status: () => 503, url: () => 'https://invalid.example/failed-font',
          request: () => ({ resourceType: () => 'font' }),
        } as unknown as Response);
        await assert.rejects(collector.snapshotRecords(), /Font response 503/);
        assert.deepEqual(await snapshot(outputRoot), before);
      } finally {
        collector.stop();
      }
      const escapeOutput = resolve(fixture, 'font-escape-output');
      await mkdir(escapeOutput);
      const escape = resolve(escapeOutput, 'resources');
      await symlink(outside, escape, 'junction');
      try {
        const escapeContext = createLayoutCurrentContext({ repoRoot, outputRoot: escapeOutput });
        assert.throws(() => collectFontResources(page as unknown as Page, repoRoot, escapeOutput, true, escapeContext), /physically escapes role root/);
        assert.equal(page.listenerCount('response'), 0);
        assert.deepEqual(await readdir(outside), []);
      } finally {
        await rm(escape);
      }
    });
  } finally {
    t.mock.restoreAll();
    for (const key of keys) {
      if (previous[key] === undefined) delete process.env[key];
      else process.env[key] = previous[key];
    }
    await rm(fixture, { recursive: true, force: true });
  }
});
