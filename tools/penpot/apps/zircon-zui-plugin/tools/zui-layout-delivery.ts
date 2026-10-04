import { resolveLayoutCatalogPath } from './zui-layout-paths';
import assert from 'node:assert/strict';
import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import { dirname, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parse } from 'smol-toml';
import type { CatalogManifest } from './zui-layout-catalog';
import { bytesSha256, containedPath } from './zui-layout-evidence';
import { casesSha256 } from './zui-layout-review-contract';

const catalogRoot = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '../../../../../docs/_data/layout',
);
const catalog: CatalogManifest = JSON.parse(
  await readFile(resolveLayoutCatalogPath(catalogRoot, 'catalog.json'), 'utf8'),
);
const write = !process.argv.includes('--check');
const sourceTree = resolveLayoutCatalogPath(catalogRoot, 'resources/source');
const files = new Map<
  string,
  { sourcePath: string; path: string; sha256: string; assetId?: string }
>();
const slash = (path: string) => path.replaceAll('\\', '/');
const assetRoot = (path: string) =>
  path.includes('/assets/')
    ? path.split('/assets/')[0] + '/assets'
    : path.includes('/ui_zui/')
      ? path.split('/ui_zui/')[0] + '/ui_zui'
      : dirname(path);

async function sourceResource(sourcePath: string, expected: string) {
  const existing = files.get(sourcePath);
  if (existing) {
    assert.equal(
      existing.sha256,
      expected,
      `Conflicting dependency hashes: ${sourcePath}`,
    );
    return existing;
  }
  const bytes = await readFile(containedPath(catalog.repoRoot, sourcePath));
  assert.equal(
    bytesSha256(bytes),
    expected,
    `Source changed after catalog generation: ${sourcePath}`,
  );
  const path = containedPath(sourceTree, sourcePath);
  if (write) {
    await mkdir(dirname(path), { recursive: true });
    await writeFile(path, bytes);
  }
  assert.equal(
    bytesSha256(await readFile(path)),
    expected,
    `Packaged dependency differs: ${sourcePath}`,
  );
  let assetId: string | undefined;
  if (sourcePath.endsWith('.zui')) {
    const document = parse(bytes.toString('utf8'));
    const asset = document['asset'] as Record<string, unknown> | undefined;
    if (typeof asset?.['id'] === 'string') assetId = asset['id'];
  }
  const item = {
    sourcePath,
    path: slash(relative(catalogRoot, resolve(sourceTree, sourcePath))),
    sha256: expected,
    assetId,
  };
  files.set(sourcePath, item);
  return item;
}

const mappedEntries = [];
const nativeFontSources = [
  'zircon_runtime/assets/fonts/editor-ui.font.toml',
  'zircon_runtime/assets/fonts/editor-ui.ttc',
  ...(
    await readdir(
      resolve(
        catalog.repoRoot,
        'zircon_runtime/assets/fonts/editor-ui-sources',
      ),
    )
  ).map((name) => `zircon_runtime/assets/fonts/editor-ui-sources/${name}`),
];
const nativeFonts = [];
for (const sourcePath of nativeFontSources)
  nativeFonts.push(
    await sourceResource(
      sourcePath,
      bytesSha256(await readFile(containedPath(catalog.repoRoot, sourcePath))),
    ),
  );
for (const entry of catalog.entries) {
  const directory = resolveLayoutCatalogPath(catalogRoot, dirname(entry.outputPath));
  assert.equal(
    bytesSha256(await readFile(containedPath(catalogRoot, entry.outputPath))),
    entry.sourceSha256,
    `Delivery mirror differs from source: ${entry.sourcePath}`,
  );
  if (entry.penpotInputPath)
    assert.equal(
      bytesSha256(
        await readFile(containedPath(catalogRoot, entry.penpotInputPath)),
      ),
      entry.penpotInputSha256,
      `Stale Penpot input: ${entry.sourcePath}`,
    );
  for (const reviewCase of entry.cases ?? [])
    if (reviewCase.reviewHost)
      assert.equal(
        bytesSha256(
          await readFile(
            containedPath(catalogRoot, reviewCase.reviewHost.path),
          ),
        ),
        reviewCase.reviewHost.sha256,
        `Stale review host: ${entry.sourcePath}/${reviewCase.id}`,
      );
  const source = await sourceResource(entry.sourcePath, entry.sourceSha256);
  const dependencies = [];
  for (const dependency of entry.dependencyFingerprints ?? [])
    dependencies.push(
      await sourceResource(dependency.sourcePath, dependency.sha256),
    );
  const fonts = new Map<string, { path: string; sha256: string }>();
  for (const evidence of entry.penpotEvidence ?? []) {
    if (
      evidence.sourceSha256 !== entry.sourceSha256 ||
      evidence.dependencySha256 !== entry.dependencySha256
    )
      continue;
    for (const [path, sha256] of evidence.runtimeAssetFingerprints ?? []) {
      if (!path.startsWith('docs/_data/layout/resources/fonts/')) continue;
      assert.equal(
        bytesSha256(await readFile(containedPath(catalog.repoRoot, path))),
        sha256,
        `Font changed: ${path}`,
      );
      fonts.set(path, {
        path: slash(relative(directory, resolve(catalog.repoRoot, path))),
        sha256,
      });
    }
  }
  const local = (resource: typeof source) => ({
    ...resource,
    path: slash(relative(directory, resolveLayoutCatalogPath(catalogRoot, resource.path))),
  });
  const mapping: Record<string, unknown> = {
    schema: 'dev.zircon.zui.layout-resource-map',
    version: 1,
    sourcePath: entry.sourcePath,
    sourceSha256: entry.sourceSha256,
    dependencySha256: entry.dependencySha256,
    source: local(source),
    assetRoot: slash(
      relative(directory, resolve(sourceTree, assetRoot(entry.sourcePath))),
    ),
    dependencies: dependencies.map(local),
    // Evidence can arrive in a different case order after a replay. Keep the
    // packaged resource map byte-stable so a valid replay cannot look stale
    // merely because font fingerprints were observed in another order.
    penpotFonts: [...fonts.values()].sort((left, right) =>
      left.path.localeCompare(right.path),
    ),
    nativeFontBundle: {
      resources: nativeFonts.map(local),
      renderVerification: 'uncaptured',
    },
    penpotInput: entry.penpotInputPath
      ? {
          path: slash(
            relative(directory, resolveLayoutCatalogPath(catalogRoot, entry.penpotInputPath)),
          ),
          sha256: entry.penpotInputSha256,
        }
      : null,
    preparation: entry.status,
  };
  const mapPath = resolve(directory, 'resources.json');
  if (write) {
    await writeFile(mapPath, `${JSON.stringify(mapping, null, 2)}\n`);
    await writeFile(
      resolve(directory, 'review-cases.json'),
      `${JSON.stringify(
        {
          schema: 'dev.zircon.zui.layout-review-cases',
          version: 1,
          sourceSha256: entry.sourceSha256,
          casesSha256: casesSha256(entry.cases ?? []),
          cases: entry.cases,
        },
        null,
        2,
      )}\n`,
    );
  } else {
    assert.deepEqual(
      JSON.parse(await readFile(mapPath, 'utf8')),
      JSON.parse(JSON.stringify(mapping)),
      `Stale resource map: ${entry.sourcePath}`,
    );
    const cases = JSON.parse(
      await readFile(resolve(directory, 'review-cases.json'), 'utf8'),
    );
    assert.equal(cases.sourceSha256, entry.sourceSha256);
    assert.equal(cases.casesSha256, casesSha256(entry.cases ?? []));
    assert.deepEqual(cases.cases, entry.cases);
  }
  mappedEntries.push({
    sourcePath: entry.sourcePath,
    resourceMap: slash(relative(catalogRoot, mapPath)),
  });
}
const manifest = {
  schema: 'dev.zircon.zui.layout-source-package',
  version: 1,
  sourceCount: catalog.sourceCount,
  files: [...files.values()].sort((a, b) =>
    a.sourcePath.localeCompare(b.sourcePath),
  ),
  entries: mappedEntries,
};
if (write)
  await writeFile(
    resolveLayoutCatalogPath(catalogRoot, 'resources/source-manifest.json'),
    `${JSON.stringify(manifest, null, 2)}\n`,
  );
else
  assert.deepEqual(
    JSON.parse(
      await readFile(
        resolveLayoutCatalogPath(catalogRoot, 'resources/source-manifest.json'),
        'utf8',
      ),
    ),
    JSON.parse(JSON.stringify(manifest)),
    'Source package manifest differs from current catalog',
  );
console.log(
  JSON.stringify({
    entries: mappedEntries.length,
    packagedFiles: files.size,
    wrote: write,
  }),
);
