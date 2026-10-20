import { createLayoutCurrentContext, layoutCurrentOutputPath, type LayoutCurrentContext } from './zui-layout-paths';
import { readCurrentDeliveryCatalog, validateCurrentDeliveryPackage } from './zui-layout-delivery';
import { withLayoutCatalogLock } from './zui-layout-catalog-lock';
import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import {
  isStructuralWorkbenchHostDocument,
  type CatalogManifest,
} from './zui-layout-catalog';
import { LayoutDependencies } from './zui-layout-dependencies';
import {
  bytesSha256,
  dependencyFingerprints,
  dependenciesSha256,
  containedPath,
} from './zui-layout-evidence';
import { prepareThemeReviewHost } from './zui-layout-theme-hosts';
import { prepareComponentReviewHost } from './zui-layout-component-hosts';
import { prepareDynamicReviewState } from './zui-layout-dynamic-hosts';
import { embedLayoutImages } from './zui-layout-images';
import { parseZuiLayoutSource } from '../src/bridge/zui-layout-optimizer';
import {
  parseZuiDocument,
  serializeZuiDocument,
} from '../src/bridge/zui-document';
import {
  projectZuiDocument,
  cloneProjectionSnapshot,
} from '../src/bridge/penpot-projection';
import {
  reviewHost,
  reconcileReviewSource,
} from '../src/bridge/zui-review-host';

async function replay(catalog: CatalogManifest, context: LayoutCurrentContext): Promise<void> {
const packageRoot = layoutCurrentOutputPath(context, 'resources/source');
const dependencies = new LayoutDependencies();
await dependencies.load(
  packageRoot,
  [...new Set(catalog.entries.flatMap((entry) => [
    entry.sourcePath,
    ...(entry.dependencyFingerprints ?? []).map((item) => item.sourcePath)
      .filter((path) => path.endsWith('.zui')),
  ]))],
);
const records = [];
for (const entry of catalog.entries) {
  const source = await readFile(
    containedPath(packageRoot, entry.sourcePath),
    'utf8',
  );
  assert.equal(
    bytesSha256(source),
    entry.sourceSha256,
    `Packaged source changed: ${entry.sourcePath}`,
  );
  let prepared = '';
  let failure: string | undefined;
  try {
    const input =
      entry.sourceFormat === 'v2'
        ? parseZuiDocument(source).document
        : parseZuiLayoutSource(source, entry.sourcePath).document;
    // Match catalog preparation: these retained host contracts deliberately
    // keep runtime imports unexpanded in the editable design projection.
    const structuralHost = isStructuralWorkbenchHostDocument(input);
    if (!structuralHost)
      dependencies.embed(
        input,
        entry.sourcePath,
        entry.cases?.[0]?.themeSourcePath,
      );
    // Catalog generation applies deterministic design-only state to retained
    // hosts before building their projection. Replay must use the same input
    // so it verifies the delivery artifact rather than a collapsed runtime
    // default branch.
    prepareDynamicReviewState(input, entry.sourcePath);
    const consumer = structuralHost
      ? null
      : ((await prepareThemeReviewHost(packageRoot, entry.sourcePath, input)) ??
        (await prepareComponentReviewHost(
          packageRoot,
          entry.sourcePath,
          input,
          dependencies,
          entry.cases?.[0]?.themeSourcePath,
        )));
    if (consumer) {
      input['penpot_review_host'] = consumer.projection;
      input['penpot_dependency_sources'] = [
        ...new Set([
          ...((input['penpot_dependency_sources'] as string[]) ?? []),
          ...consumer.consumers.map((item) => item.sourcePath),
          ...((consumer.projection['penpot_dependency_sources'] as string[]) ??
            []),
        ]),
      ];
      // A design projection can have a Penpot consumer even when the case's
      // native host is intentionally incompatible (for example a WoC theme
      // projected for editing).  Catalog generation only publishes a
      // fingerprinted reviewHost for compatible contracts; compare the
      // generated consumer against that fingerprint when it is present and
      // leave the design-only projection unasserted here.
      for (const reviewCase of entry.cases ?? []) {
        if (!reviewCase.reviewHost) continue;
        assert.equal(
          bytesSha256(consumer.source),
          reviewCase.reviewHost.sha256,
          `Packaged host differs: ${entry.sourcePath}/${reviewCase.id}`,
        );
      }
    }
    await embedLayoutImages(input, entry.sourcePath, packageRoot);
    const fingerprints = await dependencyFingerprints(
      packageRoot,
      entry.sourcePath,
      input,
    );
    assert.equal(
      dependenciesSha256(fingerprints),
      entry.dependencySha256,
      `Packaged dependencies differ: ${entry.sourcePath}`,
    );
    projectZuiDocument(reviewHost(input) ?? input);
    input['penpot_original_source'] = source;
    const exported = reconcileReviewSource(
      input,
      cloneProjectionSnapshot(projectZuiDocument(reviewHost(input) ?? input)),
    );
    assert.equal(
      exported.source,
      source,
      `Delivery export changed runtime source: ${entry.sourcePath}`,
    );
    prepared = serializeZuiDocument(input);
  } catch (error) {
    failure = error instanceof Error ? error.message : String(error);
  }
  if (entry.status === 'prepared') {
    assert.equal(
      failure,
      undefined,
      `Delivery re-import failed: ${entry.sourcePath}`,
    );
    if (bytesSha256(prepared) !== entry.penpotInputSha256)
      await writeFile(
        layoutCurrentOutputPath(context, 'evidence/delivery-mismatch.zui'),
        prepared,
      );
    assert.equal(
      bytesSha256(prepared),
      entry.penpotInputSha256,
      `Delivery projection changed: ${entry.sourcePath}`,
    );
  } else {
    const original = entry.diagnostics.find(
      (item) => item.code === 'preparation-failed',
    )?.message;
    assert.ok(
      original,
      `Missing original preparation failure: ${entry.sourcePath}`,
    );
    assert.equal(
      failure,
      original,
      `Delivery changed the unsupported contract: ${entry.sourcePath}`,
    );
  }
  records.push({
    sourcePath: entry.sourcePath,
    sourceSha256: entry.sourceSha256,
    dependencySha256: entry.dependencySha256,
    penpotInputSha256: entry.penpotInputSha256,
    status:
      entry.status === 'prepared'
        ? 'identical-projection'
        : 'identical-unsupported-contract',
    runtimeSourceRoundtrip:
      entry.status === 'prepared' ? 'byte-identical' : 'not-run',
    error: failure,
  });
}
const report = {
  schema: 'dev.zircon.zui.layout-delivery-replay',
  version: 1,
  generatedAt: new Date().toISOString(),
  sourceCount: catalog.sourceCount,
  identicalProjectionCount: records.filter(
    (item) => item.status === 'identical-projection',
  ).length,
  unsupportedContractCount: records.filter(
    (item) => item.status === 'identical-unsupported-contract',
  ).length,
  nativeRenderStatus: 'uncaptured',
  records,
};
await writeFile(
  layoutCurrentOutputPath(context, 'evidence/delivery-replay.json'),
  `${JSON.stringify(report, null, 2)}\n`,
);
console.log(
  JSON.stringify({
    entries: records.length,
    identicalProjections: report.identicalProjectionCount,
    preservedUnsupportedContracts: report.unsupportedContractCount,
  }),
);

}

export async function runCurrentLayoutDeliveryReplay(_args: string[] = []): Promise<void> {
  const context = createLayoutCurrentContext();
  const before = await readCurrentDeliveryCatalog(context, false);
  await validateCurrentDeliveryPackage(context, before);
  await withLayoutCatalogLock(context.outputRoot, { command: 'zui-layout-delivery-replay', context }, async () => {
    const catalog = await readCurrentDeliveryCatalog(context, false);
    await validateCurrentDeliveryPackage(context, catalog);
    await mkdir(dirname(layoutCurrentOutputPath(context, 'evidence/delivery-replay.json')), { recursive: true });
    await replay(catalog, context);
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href)
  void runCurrentLayoutDeliveryReplay(process.argv.slice(2)).catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
