import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, readdir, readlink, rm, symlink, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import test from 'node:test';
import type { CatalogManifest } from './zui-layout-catalog';
import { requireApprovedPenpotArtifactDirectory } from './zui-layout-artifact-path';

test('actual current gallery and native import preserve current source and output roles', async (t) => {
  const artifactRoot = process.env['ZUI_LAYOUT_TEST_ARTIFACT_ROOT'];
  assert.ok(artifactRoot, 'Root must register an existing ZUI_LAYOUT_TEST_ARTIFACT_ROOT');
  requireApprovedPenpotArtifactDirectory(artifactRoot, 'Gallery/native test artifact root');
  const fixture = await mkdtemp(resolve(artifactRoot, 'layout-current-gallery-native-'));
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
    const { bytesSha256 } = await import('./zui-layout-evidence');
    const snapshot = async (): Promise<Record<string, string>> => {
      const files: Record<string, string> = {};
      const visit = async (directory: string, prefix = ''): Promise<void> => {
        for (const item of await readdir(directory, { withFileTypes: true })) {
          const key = prefix + item.name;
          const path = resolve(directory, item.name);
          if (item.isSymbolicLink()) files[key] = 'link:' + await readlink(path);
          else if (item.isDirectory()) { files[key + '/'] = 'directory'; await visit(path, key + '/'); }
          else files[key] = bytesSha256(await readFile(path));
        }
      };
      await visit(outputRoot);
      return files;
    };
    const beforeImports = await snapshot();
    const { runCurrentLayoutCatalog, renderIndex } = await import('./zui-layout-catalog');
    const { runCurrentLayoutGallery } = await import('./zui-layout-gallery');
    const { runCurrentLayoutGalleryCheck } = await import('./zui-layout-gallery-check');
    const { runCurrentLayoutEngineImport, importEngineReport } = await import('./zui-layout-import-engine');
    const { runCurrentLayoutReview } = await import('./zui-layout-review');
    const { createLayoutCurrentContext } = await import('./zui-layout-paths');
    const { caseSha256, requiredNativeRenderer } = await import('./zui-layout-review-contract');
    let launches = 0;
    const launchBrowser = async (): Promise<never> => {
      launches += 1;
      throw new Error('Synthetic browser stop: no browser or capture success');
    };
    const catalogPath = resolve(outputRoot, 'catalog.json');
    const reportPath = resolve(fixture, 'native-report.json');
    const sourcePath = 'tools/penpot/apps/zircon-zui-plugin/src/bridge/roundtrip-fixture.zui';
    const rendererPath = 'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-import-engine.ts';
    const sourceSha = bytesSha256(await readFile(resolve(repoRoot, sourcePath)));
    const rendererSha = bytesSha256(await readFile(resolve(repoRoot, rendererPath)));
    const readManifest = async (): Promise<CatalogManifest> => JSON.parse(await readFile(catalogPath, 'utf8'));
    const saveManifest = async (value: unknown) => writeFile(catalogPath, JSON.stringify(value));
    let generated: CatalogManifest;
    let report: { schema: string; version: number; rendererSha256: string; evidence: Array<{
      sourcePath: string; caseId: string; status: 'failed'; screenshotPath: string; screenshotSha256: string;
      sourceSha256: string; dependencySha256: string | undefined; caseSha256: string; rendererKind: string;
      rendererPath: string; rendererSha256: string; error: string;
    }> };
    const saveReport = async (value: unknown) => writeFile(reportPath, JSON.stringify(value));
    const rejectGallery = async (manifest: unknown, expected: RegExp) => {
      await saveManifest(manifest);
      const before = await snapshot();
      await assert.rejects(runCurrentLayoutGallery(), expected);
      await assert.rejects(runCurrentLayoutGalleryCheck({ launchBrowser }), expected);
      await assert.rejects(runCurrentLayoutEngineImport([catalogPath, reportPath]), expected);
      assert.deepEqual(await snapshot(), before);
      assert.equal(launches, 0);
    };
    const rejectReport = async (value: unknown, expected: RegExp) => {
      await saveReport(value);
      const before = await snapshot();
      await assert.rejects(runCurrentLayoutEngineImport([catalogPath, reportPath]), expected);
      assert.deepEqual(await snapshot(), before);
    };

    await t.test('imports are inert and actual producer generates current non-Workbench source', async () => {
      assert.deepEqual(await snapshot(), beforeImports);
      await runCurrentLayoutCatalog(['--source', sourcePath]);
      generated = await readManifest();
      const entry = generated.entries[0]!;
      assert.equal(entry.status, 'prepared', JSON.stringify(entry.diagnostics));
      assert.equal(entry.sourcePath, sourcePath);
      assert.equal(entry.sourceSha256, sourceSha);
      assert.notEqual(entry.review?.status, 'accepted');
      const reviewCase = entry.cases![0]!;
      report = { schema: 'dev.zircon.zui.native-evidence', version: 1, rendererSha256: rendererSha, evidence: [{
        sourcePath, caseId: reviewCase.id, status: 'failed', screenshotPath: '', screenshotSha256: '',
        sourceSha256: entry.sourceSha256, dependencySha256: entry.dependencySha256,
        caseSha256: caseSha256(reviewCase), rendererKind: requiredNativeRenderer(reviewCase),
        rendererPath, rendererSha256: rendererSha,
        error: 'Synthetic explicit native failure; no renderer, browser, GPU, or capture executed.',
      }] };
      await saveReport(report);
    });

    await t.test('missing explicit output fails gallery check and engine import before lock or browser', async () => {
      const before = await snapshot();
      delete process.env['ZUI_LAYOUT_OUTPUT_ROOT'];
      try {
        await assert.rejects(runCurrentLayoutGallery(), /explicit ZUI_LAYOUT_OUTPUT_ROOT/);
        await assert.rejects(runCurrentLayoutGalleryCheck({ launchBrowser }), /explicit ZUI_LAYOUT_OUTPUT_ROOT/);
        await assert.rejects(runCurrentLayoutEngineImport([catalogPath, reportPath]), /explicit ZUI_LAYOUT_OUTPUT_ROOT/);
      } finally { process.env['ZUI_LAYOUT_OUTPUT_ROOT'] = outputRoot; }
      assert.deepEqual(await snapshot(), before);
      assert.equal(launches, 0);
    });

    await t.test('actual gallery binds exact catalog bytes without rewriting prepared ZUI output', async () => {
      const outputBytes = await readFile(resolve(outputRoot, generated.entries[0]!.outputPath));
      await runCurrentLayoutGallery();
      const html = await readFile(resolve(outputRoot, 'gallery.html'), 'utf8');
      assert.ok(html.includes(`<meta name="zircon-layout-catalog-sha256" content="${bytesSha256(await readFile(catalogPath))}">`));
      assert.equal((html.match(/<article /g) ?? []).length, generated.sourceCount);
      assert.ok(html.includes('pending'));
      assert.equal(html.includes('class="status accepted"'), false);
      assert.deepEqual(await readFile(resolve(outputRoot, generated.entries[0]!.outputPath)), outputBytes);
      assert.equal(bytesSha256(await readFile(resolve(repoRoot, sourcePath))), sourceSha);
    });

    await t.test('historical package and wrong repository identity fail without writes or browser', async () => {
      for (const value of [
        { ...generated, role: 'history' }, { ...generated, role: 'package' },
        { ...generated, historical_only: true }, { ...generated, schema: 'history' },
        { ...generated, sourceCount: generated.sourceCount + 1 },
      ]) await rejectGallery(value, /Invalid current layout catalog identity/);
      await rejectGallery({ ...generated, repoRoot: outside }, /root identity differs/);
      await saveManifest(generated);
    });

    await t.test('traversal and physical output escape fail before browser or lock', async () => {
      const traversal = structuredClone(generated);
      traversal.entries[0]!.outputPath = '../escaped.zui';
      await rejectGallery(traversal, /escapes role root/);
      const escape = resolve(outputRoot, 'escape');
      await symlink(outside, escape, 'junction');
      try {
        const physical = structuredClone(generated);
        physical.entries[0]!.resultPath = 'escape/result.md';
        await rejectGallery(physical, /physically escapes role root/);
        assert.deepEqual(await readdir(outside), []);
      } finally { await rm(escape); }
      await saveManifest(generated);
    });

    await t.test('same-count stale HTML cannot launch the gallery browser', async () => {
      await runCurrentLayoutGallery();
      const changed = { ...generated, generatedAt: generated.generatedAt + '-new-identity' };
      await saveManifest(changed);
      const before = await snapshot();
      await assert.rejects(runCurrentLayoutGalleryCheck({ launchBrowser }), /does not match the current catalog SHA/);
      assert.deepEqual(await snapshot(), before);
      assert.equal(launches, 0);
      await saveManifest(generated);
      await runCurrentLayoutGallery();
    });

    await t.test('valid current gallery reaches the stopped browser boundary without claiming QA success', async () => {
      const before = await snapshot();
      await assert.rejects(runCurrentLayoutGalleryCheck({ launchBrowser }), /Synthetic browser stop/);
      assert.equal(launches, 1);
      const after = await snapshot();
      for (const [path, value] of Object.entries(before)) assert.equal(after[path], value);
      const added = Object.keys(after).filter((path) => !(path in before));
      assert.deepEqual(added, 'evidence/' in before ? [] : ['evidence/']);
      assert.equal((await readdir(outputRoot)).includes('.zui-layout-catalog.lock'), false);
      assert.equal(Object.keys(after).some((path) => path.endsWith('gallery-qa.json')), false);
    });

    await t.test('current engine entry rejects wrong catalog report role and archive identity', async () => {
      const before = await snapshot();
      await assert.rejects(runCurrentLayoutEngineImport([resolve(outside, 'catalog.json'), reportPath]), /current output catalog identity/);
      assert.deepEqual(await snapshot(), before);
      await assert.rejects(runCurrentLayoutEngineImport([catalogPath, resolve(repoRoot, rendererPath)]), /must be a directory below/);
      assert.deepEqual(await snapshot(), before);
      await rejectReport({ ...report, role: 'history' }, /Historical or package native report/);
      await rejectReport({ ...report, role: 'package' }, /Historical or package native report/);
      await rejectReport({ ...report, historical_only: true }, /Historical or package native report/);
      await saveReport(report);
      process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'] = outside;
      const historicalReport = resolve(outside, 'native-report.json');
      await writeFile(historicalReport, JSON.stringify(report));
      try {
        const stable = await snapshot();
        await assert.rejects(runCurrentLayoutEngineImport([catalogPath, historicalReport]), /Historical archive report/);
        assert.deepEqual(await snapshot(), stable);
      } finally { delete process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT']; }
    });

    await t.test('full incoming batch host hashes and pending requirements remain mandatory', async () => {
      const item = report.evidence[0]!;
      await rejectReport({ ...report, evidence: [item, item] }, /Duplicate native evidence/);
      await rejectReport({ ...report, evidence: [item, { ...item, caseId: 'unknown' }] }, /Unknown native evidence/);
      await rejectReport({ ...report, evidence: [{ ...item, sourceSha256: '0'.repeat(64) }] }, /Stale or invalid native evidence/);
      const wrongHost = item.rendererKind === 'zircon-runtime-wgpu-headless' ? 'zircon-editor-retained-host' : 'zircon-runtime-wgpu-headless';
      await rejectReport({ ...report, evidence: [{ ...item, rendererKind: wrongHost }] }, /Wrong native host/);
      await rejectReport({ ...report, evidence: [{ ...item, status: 'pending', pendingReason: '' }] }, /Pending native evidence has no explanation/);
      await rejectReport({ ...report, evidence: [{ ...item, status: 'pending', pendingReason: 'Explicit synthetic pending; no capture' }] }, /Native capture lacks geometry or text/);
      await rejectReport({ ...report, evidence: [{ ...item, screenshotPath: '../escape.png' }] }, /escapes role root/);
      await saveReport(report);
    });

    await t.test('legacy three-argument API still imports explicit failed evidence without current context', async () => {
      const legacy = structuredClone(generated);
      await importEngineReport(legacy, report, outputRoot);
      assert.deepEqual(legacy.entries[0]!.engineEvidence, report.evidence);
      assert.notEqual(legacy.entries[0]!.review?.status, 'accepted');
      assert.ok(renderIndex(legacy).includes('(../../plans/unreal-editor-source-cutover.md)'));
    });

    await t.test('actual current import and review preserve explicit failed evidence and active plan identity', async () => {
      await runCurrentLayoutEngineImport([catalogPath, reportPath]);
      const imported = await readManifest();
      assert.deepEqual(imported.entries[0]!.engineEvidence, report.evidence);
      assert.notEqual(imported.entries[0]!.review?.status, 'accepted');
      const context = createLayoutCurrentContext();
      const planURL = pathToFileURL(resolve(repoRoot, 'docs/plans/unreal-editor-source-cutover.md')).href;
      assert.ok(renderIndex(imported, context).includes(`(${planURL})`));
      assert.ok((await readFile(resolve(outputRoot, 'index.md'), 'utf8')).includes(`(${planURL})`));
      await runCurrentLayoutReview([{ sourcePath, decision: 'needs_revision', observations: 'Explicit synthetic failed native evidence requires actual capture.' }]);
      const reviewed = await readManifest();
      assert.equal(reviewed.entries[0]!.review?.status, 'needs_revision');
      assert.deepEqual(reviewed.entries[0]!.engineEvidence, report.evidence);
      assert.ok((await readFile(resolve(outputRoot, 'index.md'), 'utf8')).includes(`(${planURL})`));
      await runCurrentLayoutGallery();
      assert.equal((await readFile(resolve(outputRoot, 'gallery.html'), 'utf8')).includes('class="status accepted"'), false);
      assert.equal(bytesSha256(await readFile(resolve(repoRoot, sourcePath))), sourceSha);
    });

    await t.test('stale native evidence remains visible as stale and never becomes accepted', async () => {
      const stale = await readManifest();
      stale.entries[0]!.engineEvidence![0]!.rendererSha256 = '0'.repeat(64);
      await saveManifest(stale);
      await runCurrentLayoutGallery();
      const html = await readFile(resolve(outputRoot, 'gallery.html'), 'utf8');
      assert.ok(html.includes('证据已过期，待复拍'));
      assert.equal(html.includes('class="status accepted"'), false);
      assert.equal(launches, 1);
    });
  } finally {
    for (const key of keys) {
      if (previous[key] === undefined) delete process.env[key];
      else process.env[key] = previous[key];
    }
    await rm(fixture, { recursive: true, force: true });
  }
});
