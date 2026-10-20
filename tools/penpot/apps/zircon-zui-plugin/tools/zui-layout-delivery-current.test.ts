import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, readdir, readlink, rm, symlink, writeFile } from 'node:fs/promises';
import { dirname, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { requireApprovedPenpotArtifactDirectory } from './zui-layout-artifact-path';
import type { CatalogManifest } from './zui-layout-catalog';

test('actual current delivery producer and package replay preserve source and review contracts', async (t) => {
  const artifactRoot = process.env['ZUI_LAYOUT_TEST_ARTIFACT_ROOT'];
  assert.ok(artifactRoot, 'Root must register an existing ZUI_LAYOUT_TEST_ARTIFACT_ROOT');
  requireApprovedPenpotArtifactDirectory(artifactRoot, 'Delivery test artifact root');
  const fixture = await mkdtemp(resolve(artifactRoot, 'layout-current-delivery-'));
  const outputRoot = resolve(fixture, 'output');
  const outside = resolve(fixture, 'outside');
  await mkdir(outputRoot);
  await mkdir(outside);
  const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../../../../..');
  const keys = ['ZUI_LAYOUT_REPO_ROOT', 'ZUI_LAYOUT_OUTPUT_ROOT', 'ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'] as const;
  const previous = Object.fromEntries(keys.map((key) => [key, process.env[key]]));
  const { runCurrentLayoutStatus } = await import('./zui-layout-status');
  const { runCurrentLayoutContactSheet } = await import('./zui-layout-contact-sheet');
  const { runCurrentLayoutTextAudit } = await import('./zui-layout-text-audit');
  process.env['ZUI_LAYOUT_REPO_ROOT'] = repoRoot;
  process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = outputRoot;
  delete process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'];
  try {
    const { bytesSha256 } = await import('./zui-layout-evidence');
    const { caseSha256, requiredNativeRenderer } = await import('./zui-layout-review-contract');
    const snapshot = async (): Promise<Record<string, string>> => {
      const files: Record<string, string> = {};
      const visit = async (directory: string, prefix = ''): Promise<void> => {
        for (const item of await readdir(directory, { withFileTypes: true })) {
          const path = resolve(directory, item.name);
          const key = prefix + item.name;
          if (item.isSymbolicLink()) files[key] = 'link:' + await readlink(path);
          else if (item.isDirectory()) {
            files[key + '/'] = 'directory';
            await visit(path, key + '/');
          } else files[key] = bytesSha256(await readFile(path));
        }
      };
      await visit(outputRoot);
      return files;
    };
    const beforeImport = await snapshot();
    const { runCurrentLayoutDelivery } = await import('./zui-layout-delivery');
    const { runCurrentLayoutDeliveryReplay } = await import('./zui-layout-delivery-replay');
    const { runCurrentLayoutCatalog } = await import('./zui-layout-catalog');
    const catalogPath = resolve(outputRoot, 'catalog.json');
    const packagePath = resolve(outputRoot, 'resources/source-manifest.json');
    const sourcePath = 'tools/penpot/apps/zircon-zui-plugin/src/bridge/roundtrip-fixture.zui';
    const source = await readFile(resolve(repoRoot, sourcePath));
    const save = async (value: unknown) => writeFile(catalogPath, JSON.stringify(value));
    const readManifest = async (): Promise<CatalogManifest> => JSON.parse(await readFile(catalogPath, 'utf8'));
    let manifest: CatalogManifest;
    let packageBytes: Buffer;

    await t.test('imports are inert and actual selected-source producer prepares the current input', async () => {
      assert.deepEqual(await snapshot(), beforeImport);
      await runCurrentLayoutCatalog(['--source', sourcePath]);
      manifest = await readManifest();
      assert.equal(manifest.sourceCount, 1);
      assert.equal(manifest.repoRoot, repoRoot);
      assert.equal(manifest.entries[0]!.sourceSha256, bytesSha256(source));
      assert.equal(manifest.entries[0]!.status, 'prepared', JSON.stringify(manifest.entries[0]!.diagnostics));
    });

    await t.test('actual delivery, immutable check and package-only replay retain uncaptured state', async () => {
      await runCurrentLayoutDelivery([]);
      const entry = manifest.entries[0]!;
      const packaged = resolve(outputRoot, 'resources/source', sourcePath);
      assert.deepEqual(await readFile(packaged), source);
      assert.deepEqual(await readFile(resolve(outputRoot, entry.outputPath)), source);
      assert.notEqual(packaged, resolve(repoRoot, sourcePath));
      packageBytes = await readFile(packagePath);
      const before = await snapshot();
      await runCurrentLayoutDelivery(['--check']);
      assert.deepEqual(await snapshot(), before);
      await runCurrentLayoutDeliveryReplay([]);
      const report = JSON.parse(await readFile(resolve(outputRoot, 'evidence/delivery-replay.json'), 'utf8'));
      assert.equal(report.sourceCount, 1);
      assert.equal(report.identicalProjectionCount, 1);
      assert.equal(report.nativeRenderStatus, 'uncaptured');
      assert.equal(report.records[0].runtimeSourceRoundtrip, 'byte-identical');
      assert.deepEqual(await readManifest(), manifest);
      assert.notEqual(entry.visualStatus, 'passed');
      assert.equal(entry.review, undefined);
      assert.deepEqual(await readFile(resolve(repoRoot, sourcePath)), source);
      assert.equal((await readdir(outputRoot)).includes('.zui-layout-catalog.lock'), false);
    });

    await t.test('missing explicit output rejects both actual entrypoints without output changes', async () => {
      const before = await snapshot();
      delete process.env['ZUI_LAYOUT_OUTPUT_ROOT'];
      try {
        await assert.rejects(runCurrentLayoutDelivery([]), /explicit ZUI_LAYOUT_OUTPUT_ROOT/);
        await assert.rejects(runCurrentLayoutDeliveryReplay([]), /explicit ZUI_LAYOUT_OUTPUT_ROOT/);
      } finally { process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = outputRoot; }
      assert.deepEqual(await snapshot(), before);
    });

    await t.test('historical, package, count and wrong-root catalog identities reject before writes', async () => {
      for (const [candidate, pattern] of [
        [{ ...manifest, historical_only: true }, /Historical or package/],
        [{ ...manifest, role: 'package' }, /Historical or package/],
        [{ ...manifest, schema: 'dev.zircon.zui.layout-source-package' }, /Historical or package/],
        [{ ...manifest, sourceCount: 2 }, /Historical or package/],
        [{ ...manifest, repoRoot: outside }, /root identity differs/],
      ] as Array<[unknown, RegExp]>) {
        await save(candidate);
        const before = await snapshot();
        await assert.rejects(runCurrentLayoutDelivery([]), pattern);
        await assert.rejects(runCurrentLayoutDeliveryReplay([]), pattern);
        assert.deepEqual(await snapshot(), before);
      }
      await save(manifest);
    });

    await t.test('source identity and mirror, Penpot-input and review-host hashes stay mandatory', async () => {
      const original = manifest.entries[0]!;
      await save({ ...manifest, entries: [{ ...original, sourceSha256: '0'.repeat(64) }] });
      const before = await snapshot();
      await assert.rejects(runCurrentLayoutDelivery([]), /Source changed after catalog generation/);
      assert.deepEqual(await snapshot(), before);
      await save(manifest);
      for (const [path, expected] of [
        [original.outputPath, /Delivery mirror differs/],
        [original.penpotInputPath!, /Stale Penpot input/],
      ] as Array<[string, RegExp]>) {
        const file = resolve(outputRoot, path);
        const bytes = await readFile(file);
        await writeFile(file, 'Synthetic corruption; no capture claim.');
        const changed = await snapshot();
        await assert.rejects(runCurrentLayoutDelivery([]), expected);
        assert.deepEqual(await snapshot(), changed);
        await writeFile(file, bytes);
      }
      const candidate = structuredClone(manifest);
      candidate.entries[0]!.cases![0]!.reviewHost = {
        ...candidate.entries[0]!.cases![0]!.reviewHost!,
        path: original.penpotInputPath!, sha256: '0'.repeat(64),
      };
      await save(candidate);
      const changed = await snapshot();
      await assert.rejects(runCurrentLayoutDelivery([]), /Stale review host/);
      assert.deepEqual(await snapshot(), changed);
      await save(manifest);
    });

    await t.test('actual output font bytes enter sorted resource maps without upgrading failed reviews', async () => {
      // Real checkout font bytes, not an invented renderer/capture or workspace copy.
      const bytes = await readFile(resolve(repoRoot, 'zircon_runtime/assets/fonts/editor-ui.ttc'));
      const font = resolve(outputRoot, 'resources/fonts', bytesSha256(bytes) + '.ttc');
      await mkdir(dirname(font), { recursive: true });
      await writeFile(font, bytes);
      const candidate = structuredClone(manifest);
      const entry = candidate.entries[0]!;
      entry.visualStatus = 'failed';
      entry.reviewHistory = [{
        status: 'needs_revision', observations: 'Synthetic failure history must survive delivery.',
        screenshotSha256: '', outputSha256: entry.outputSha256, reviewedAt: '2026-10-18T00:00:00.000Z',
      }];
      entry.penpotEvidence = [{
        caseId: entry.cases![0]!.id, status: 'failed',
        screenshotPath: '', screenshotSha256: '', caseSha256: caseSha256(entry.cases![0]!),
        sourceSha256: entry.sourceSha256, dependencySha256: entry.dependencySha256!,
        rendererKind: 'penpot-official-frontend', error: 'Synthetic capture never started.',
        runtimeAssetFingerprints: [[font, bytesSha256(bytes)]],
      }];
      await save(candidate);
      await runCurrentLayoutDelivery([]);
      const map = JSON.parse(await readFile(resolve(outputRoot, dirname(entry.outputPath), 'resources.json'), 'utf8'));
      assert.deepEqual(map.penpotFonts, [{
        path: relative(resolve(outputRoot, dirname(entry.outputPath)), font).replaceAll('\\', '/'),
        sha256: bytesSha256(bytes),
      }]);
      assert.equal(map.nativeFontBundle.renderVerification, 'uncaptured');
      await runCurrentLayoutDelivery(['--check']);
      await runCurrentLayoutDeliveryReplay([]);
      assert.deepEqual(await readManifest(), candidate);
      assert.equal((await readManifest()).entries[0]!.review?.status === 'accepted', false);
      await writeFile(font, 'changed');
      const changed = await snapshot();
      await assert.rejects(runCurrentLayoutDelivery([]), /Font changed/);
      assert.deepEqual(await snapshot(), changed);
      await writeFile(font, bytes);
      await save(manifest);
      await runCurrentLayoutDelivery([]);
    });

    await t.test('external and historical fonts cannot be silently packaged as current output', async () => {
      const font = resolve(outside, 'external.ttf');
      await writeFile(font, 'Synthetic external font');
      for (const path of [font, 'docs/_data/layout/resources/fonts/old.ttf']) {
        const candidate = structuredClone(manifest);
        const entry = candidate.entries[0]!;
        entry.penpotEvidence = [{
          caseId: entry.cases![0]!.id, status: 'failed',
          screenshotPath: '', screenshotSha256: '', caseSha256: caseSha256(entry.cases![0]!), error: 'Synthetic failure.',
          sourceSha256: entry.sourceSha256, dependencySha256: entry.dependencySha256!,
          runtimeAssetFingerprints: [[path, '0'.repeat(64)]],
        }];
        await save(candidate);
        const before = await snapshot();
        await assert.rejects(runCurrentLayoutDelivery([]), /escapes|Historical font/);
        assert.deepEqual(await snapshot(), before);
      }
      await save(manifest);
    });

    await t.test('package manifest schema, duplicates, missing inventory and SHA corruption fail without fallback', async () => {
      const valid = JSON.parse(packageBytes.toString('utf8'));
      for (const value of [
        { ...valid, schema: 'historical' },
        { ...valid, files: [...valid.files, valid.files[0]] },
        { ...valid, files: valid.files.slice(1) },
        { ...valid, files: valid.files.map((item: { sha256: string }, index: number) => index ? item : { ...item, sha256: '0'.repeat(64) }) },
      ]) {
        await writeFile(packagePath, JSON.stringify(value));
        const before = await snapshot();
        await assert.rejects(runCurrentLayoutDeliveryReplay([]), /package|Packaged/i);
        assert.deepEqual(await snapshot(), before);
      }
      await writeFile(packagePath, packageBytes);
      const packaged = resolve(outputRoot, 'resources/source', sourcePath);
      await writeFile(packaged, 'Synthetic invalid packaged source');
      const before = await snapshot();
      await assert.rejects(runCurrentLayoutDeliveryReplay([]), /Packaged dependency differs/);
      assert.deepEqual(await snapshot(), before);
      assert.deepEqual(await readFile(resolve(repoRoot, sourcePath)), source);
      await writeFile(packaged, source);
      const extra = resolve(outputRoot, 'resources/source/unlisted.zui');
      await writeFile(extra, 'Synthetic unlisted input');
      const withExtra = await snapshot();
      await assert.rejects(runCurrentLayoutDeliveryReplay([]), /inventory differs/);
      assert.deepEqual(await snapshot(), withExtra);
      await rm(extra);
    });

    await t.test('source and output traversal plus package junctions reject before lock or writes', async () => {
      for (const patch of [{ sourcePath: '../outside/input.zui' }, { resultPath: '../outside/result.md' }]) {
        await save({ ...manifest, entries: [{ ...manifest.entries[0]!, ...patch }] });
        const before = await snapshot();
        await assert.rejects(runCurrentLayoutDelivery([]), /escapes/);
        assert.deepEqual(await snapshot(), before);
      }
      await save(manifest);
      const valid = JSON.parse(packageBytes.toString('utf8'));
      await writeFile(packagePath, JSON.stringify({ ...valid, files: [{ ...valid.files[0], sourcePath: '../escape.zui' }] }));
      const before = await snapshot();
      await assert.rejects(runCurrentLayoutDeliveryReplay([]), /escapes/);
      assert.deepEqual(await snapshot(), before);
      await writeFile(packagePath, packageBytes);
      const link = resolve(outputRoot, 'resources/source/escaping');
      await symlink(outside, link, process.platform === 'win32' ? 'junction' : 'dir');
      try {
        const linked = await snapshot();
        await assert.rejects(runCurrentLayoutDelivery([]), /physically escapes|symbolic link/);
        await assert.rejects(runCurrentLayoutDeliveryReplay([]), /physically escapes|symbolic link/);
        assert.deepEqual(await snapshot(), linked);
      } finally { await rm(link); }
    });

    await t.test('unsupported diagnostics and prepared fingerprints remain exact replay gates', async () => {
      const candidate = structuredClone(manifest);
      candidate.entries[0]!.status = 'failed';
      candidate.entries[0]!.diagnostics.push({
        severity: 'error', code: 'preparation-failed', message: 'Synthetic unsupported contract must match exactly.',
      });
      await save(candidate);
      await assert.rejects(runCurrentLayoutDeliveryReplay([]), /Delivery changed the unsupported contract/);
      assert.deepEqual(await readManifest(), candidate);
      await save(manifest);
      await runCurrentLayoutDelivery([]);
      await runCurrentLayoutDelivery(['--check']);
      assert.deepEqual(await readFile(resolve(repoRoot, sourcePath)), source);
    });
    await t.test('current consumer imports are inert and missing roots reject before writes', async () => {
      const before = await snapshot();
      delete process.env['ZUI_LAYOUT_OUTPUT_ROOT'];
      try {
        await assert.rejects(runCurrentLayoutStatus(['--check']), /explicit ZUI_LAYOUT_OUTPUT_ROOT/);
        await assert.rejects(runCurrentLayoutContactSheet(['0', '1']), /explicit ZUI_LAYOUT_OUTPUT_ROOT/);
        await assert.rejects(runCurrentLayoutTextAudit([]), /explicit ZUI_LAYOUT_OUTPUT_ROOT/);
      } finally {
        process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = outputRoot;
      }
      assert.deepEqual(await snapshot(), before);
    });

    await t.test('current consumers reject historical identities and escaping evidence before output', async () => {
      for (const candidate of [
        { ...manifest, historical_only: true },
        { ...manifest, role: 'package' },
        { ...manifest, repoRoot: outside },
        { ...manifest, entries: [{ ...manifest.entries[0]!, sourceSha256: '0'.repeat(64) }] },
        { ...manifest, entries: [{ ...manifest.entries[0]!, dependencyFingerprints: [{ sourcePath, sha256: '0'.repeat(64) }] }] },
        { ...manifest, entries: [{ ...manifest.entries[0]!, previewPath: '../escape.png' }] },
      ]) {
        await save(candidate);
        const before = await snapshot();
        await assert.rejects(runCurrentLayoutStatus(['--check']));
        await assert.rejects(runCurrentLayoutContactSheet(['0', '1']));
        await assert.rejects(runCurrentLayoutTextAudit([]));
        assert.deepEqual(await snapshot(), before);
      }
      await save(manifest);
      const before = await snapshot();
      await assert.rejects(runCurrentLayoutContactSheet(['0', '0']), /positive count/);
      await assert.rejects(runCurrentLayoutContactSheet(['NaN', '1']), /integer start\/count/);
      await assert.rejects(runCurrentLayoutStatus(['--history']), /historical replay/);
      await assert.rejects(runCurrentLayoutTextAudit(['--history']), /historical replay/);
      assert.deepEqual(await snapshot(), before);
    });

    await t.test('current consumer evidence and output junction rejection leaves all output unchanged', async () => {
      await save({ ...manifest, entries: [{
        ...manifest.entries[0]!,
        penpotEvidence: [{ sourceSha256: '0'.repeat(64), dependencySha256: '0'.repeat(64) }],
      }] });
      let before = await snapshot();
      await assert.rejects(runCurrentLayoutStatus([]));
      await assert.rejects(runCurrentLayoutContactSheet(['0', '1']));
      await assert.rejects(runCurrentLayoutTextAudit([]));
      assert.deepEqual(await snapshot(), before);
      await save(manifest);
      const link = resolve(outputRoot, 'consumer-escaping');
      await symlink(outside, link, process.platform === 'win32' ? 'junction' : 'dir');
      try {
        await save({ ...manifest, entries: [{ ...manifest.entries[0]!, previewPath: 'consumer-escaping/preview.png' }] });
        before = await snapshot();
        await assert.rejects(runCurrentLayoutStatus(['--check']), /physically escapes/);
        await assert.rejects(runCurrentLayoutContactSheet(['0', '1']), /physically escapes/);
        await assert.rejects(runCurrentLayoutTextAudit([]), /physically escapes/);
        assert.deepEqual(await snapshot(), before);
      } finally {
        await save(manifest);
        await rm(link);
      }
    });

    await t.test('unaccepted captures require complete fingerprints geometry and current case source before writes', async () => {
      const original = manifest.entries[0]!;
      const reviewCase = original.cases![0]!;
      const rendererPath = 'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-import-engine.ts';
      const rendererSha256 = bytesSha256(await readFile(resolve(repoRoot, rendererPath)));
      // Deliberately incomplete declarations exercise rejection only; no capture success is claimed.
      const capture = {
        caseId: reviewCase.id, status: 'pending' as const, pendingReason: 'Incomplete negative-control evidence.',
        sourceSha256: original.sourceSha256, dependencySha256: original.dependencySha256!,
        caseSha256: caseSha256(reviewCase), reviewHostSha256: reviewCase.reviewHost?.sha256,
        rendererKind: requiredNativeRenderer(reviewCase), rendererPath, rendererSha256,
        captureProgramFingerprints: [[rendererPath, rendererSha256]] as Array<[string, string]>,
        screenshotPath: original.previewPath, screenshotSha256: '0'.repeat(64),
        geometryPath: 'evidence/not-captured.geometry.json', geometrySha256: '0'.repeat(64),
        textPath: 'evidence/not-captured.text.json', textSha256: '0'.repeat(64),
      };
      for (const patch of [
        { captureProgramFingerprints: undefined },
        { geometryPath: undefined, geometrySha256: undefined },
        { caseId: 'unregistered-case' },
        { caseSha256: '0'.repeat(64) },
        { sourceSha256: '0'.repeat(64) },
        { dependencySha256: '0'.repeat(64) },
      ]) {
        await save({ ...manifest, entries: [{ ...original, engineEvidence: [{ ...capture, ...patch }] }] });
        const before = await snapshot();
        await assert.rejects(runCurrentLayoutStatus([]), /capture|geometry/i);
        await assert.rejects(runCurrentLayoutContactSheet(['0', '1']), /capture|geometry/i);
        await assert.rejects(runCurrentLayoutTextAudit([]), /capture|geometry/i);
        assert.deepEqual(await snapshot(), before);
      }
      await save(manifest);
    });

    await t.test('actual current status check and uncaptured text audit retain pending acceptance', async () => {
      const before = await snapshot();
      await runCurrentLayoutStatus(['--check']);
      assert.deepEqual(await snapshot(), before);
      await runCurrentLayoutTextAudit([]);
      const summary = JSON.parse(await readFile(resolve(outputRoot, 'evidence/text-structure-summary.json'), 'utf8'));
      assert.equal(summary.acceptanceStatus, 'unaccepted');
      assert.equal(summary.measuredCases, 0);
      assert.deepEqual(await readManifest(), manifest);
      assert.equal((await readdir(outputRoot)).includes('.zui-layout-catalog.lock'), false);
      assert.deepEqual(await readFile(resolve(repoRoot, sourcePath)), source);
    });

    await t.test('diagnostic pending native measurements audit after import and current root switching without acceptance', async () => {
      const diagnosticRoot = resolve(fixture, 'diagnostic-current-output');
      await mkdir(diagnosticRoot);
      const originalSnapshot = await snapshot();
      process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = diagnosticRoot;
      try {
        await runCurrentLayoutCatalog(['--source', sourcePath]);
        await runCurrentLayoutDelivery([]);
        const catalogFile = resolve(diagnosticRoot, 'catalog.json');
        const diagnostic: CatalogManifest = JSON.parse(await readFile(catalogFile, 'utf8'));
        const entry = diagnostic.entries[0]!;
        const reviewCase = entry.cases![0]!;
        const rendererPath = 'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-import-engine.ts';
        const rendererSha256 = bytesSha256(await readFile(resolve(repoRoot, rendererPath)));
        const prefix = `${dirname(entry.outputPath)}/evidence/diagnostic-native`;
        await mkdir(resolve(diagnosticRoot, prefix), { recursive: true });
        // Diagnostic file-integrity/measurement input only: no native renderer or browser ran,
        // this single-pixel placeholder and measured node are never visual acceptance evidence.
        const screenshot = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+a4i8AAAAASUVORK5CYII=', 'base64');
        const bounds = { x: 4, y: 4, width: 120, height: 24 };
        const geometry = JSON.stringify({ case: reviewCase, caseSha256: caseSha256(reviewCase), layout: { semanticNodes: [{
          nodeId: 'diagnostic-measurement', parentNodeId: null, component: 'Text', visible: true,
          bounds, text: 'Diagnostic pending',
        }] } });
        const text = JSON.stringify({ nodes: [{ nodeId: 'diagnostic-measurement', text: 'Diagnostic pending',
          layout: { lines: [{ frame: { x: 4, y: 4, width: 100, height: 16 } }] },
        }] });
        const screenshotPath = `${prefix}/placeholder.png`;
        const geometryPath = `${prefix}/geometry.json`;
        const textPath = `${prefix}/text.json`;
        await writeFile(resolve(diagnosticRoot, screenshotPath), screenshot);
        await writeFile(resolve(diagnosticRoot, geometryPath), geometry);
        await writeFile(resolve(diagnosticRoot, textPath), text);
        entry.visualStatus = 'pending';
        entry.engineEvidence = [{
          caseId: reviewCase.id, status: 'pending',
          pendingReason: 'Diagnostic pending measurement fixture; no renderer execution or visual acceptance.',
          sourceSha256: entry.sourceSha256, dependencySha256: entry.dependencySha256!,
          caseSha256: caseSha256(reviewCase), reviewHostSha256: reviewCase.reviewHost?.sha256,
          rendererKind: requiredNativeRenderer(reviewCase), rendererPath, rendererSha256,
          captureProgramFingerprints: [[rendererPath, rendererSha256]],
          screenshotPath, screenshotSha256: bytesSha256(screenshot),
          geometryPath, geometrySha256: bytesSha256(geometry), textPath, textSha256: bytesSha256(text),
        }];
        await writeFile(catalogFile, JSON.stringify(diagnostic));
        const catalogBytes = await readFile(catalogFile);
        await runCurrentLayoutStatus(['--check']);
        await runCurrentLayoutTextAudit([]);
        const summary = JSON.parse(await readFile(resolve(diagnosticRoot, 'evidence/text-structure-summary.json'), 'utf8'));
        assert.equal(summary.acceptanceStatus, 'unaccepted');
        assert.equal(summary.measuredCases, 1);
        assert.equal(summary.entries.length, 1);
        const report = JSON.parse(await readFile(resolve(diagnosticRoot, summary.entries[0].report), 'utf8'));
        assert.equal(report.status, 'requires-visual-review');
        assert.equal(report.cases.length, 1);
        assert.equal(report.cases[0].renderer, 'engine');
        assert.deepEqual(report.cases[0].findings, []);
        assert.equal(report.cases[0].geometrySha256, bytesSha256(geometry));
        assert.equal(report.cases[0].textSha256, bytesSha256(text));
        assert.deepEqual(await readFile(catalogFile), catalogBytes);
        assert.equal(entry.review?.status === 'accepted', false);
        assert.deepEqual(await snapshot(), originalSnapshot);
      } finally {
        process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = outputRoot;
      }
      await runCurrentLayoutTextAudit([]);
      const originalSummary = JSON.parse(await readFile(resolve(outputRoot, 'evidence/text-structure-summary.json'), 'utf8'));
      assert.equal(originalSummary.measuredCases, 0);
      assert.deepEqual(await readManifest(), manifest);
      assert.deepEqual(await readFile(resolve(repoRoot, sourcePath)), source);
    });

    await t.test('explicit failed attempts remain auditable without requiring unproduced capture files', async () => {
      const candidate = structuredClone(manifest);
      const entry = candidate.entries[0]!;
      const reviewCase = entry.cases![0]!;
      entry.visualStatus = 'failed';
      entry.engineEvidence = [{ caseId: reviewCase.id, status: 'failed',
        sourceSha256: entry.sourceSha256, dependencySha256: entry.dependencySha256!,
        caseSha256: caseSha256(reviewCase), screenshotPath: '', screenshotSha256: '',
        error: 'Explicit diagnostic failed attempt; capture never started.',
      }];
      await save(candidate);
      try {
        await runCurrentLayoutStatus([]);
        await runCurrentLayoutTextAudit([]);
        const summary = JSON.parse(await readFile(resolve(outputRoot, 'evidence/text-structure-summary.json'), 'utf8'));
        assert.equal(summary.measuredCases, 0);
        assert.equal(summary.acceptanceStatus, 'unaccepted');
        assert.deepEqual(await readManifest(), candidate);
      } finally { await save(manifest); }
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
