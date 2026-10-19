import { createLayoutCurrentContext, assertLayoutCurrentRoots, layoutCurrentSourcePath, layoutCurrentOutputPath, type LayoutCurrentContext } from './zui-layout-paths';
import { withLayoutCatalogLock } from './zui-layout-catalog-lock';
import assert from 'node:assert/strict';
import { mkdir, readFile, readdir, writeFile, lstat } from 'node:fs/promises';
import { dirname, relative, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { parse } from 'smol-toml';
import type { CatalogManifest } from './zui-layout-catalog';
import { bytesSha256, containedPath } from './zui-layout-evidence';
import { casesSha256 } from './zui-layout-review-contract';

async function deliver(catalog: CatalogManifest, context: LayoutCurrentContext, write: boolean) {
const catalogRoot = context.outputRoot;
const sourceTree = layoutCurrentOutputPath(context, 'resources/source');
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
  const bytes = await readFile(layoutCurrentSourcePath(context, sourcePath));
  assert.equal(
    bytesSha256(bytes),
    expected,
    `Source changed after catalog generation: ${sourcePath}`,
  );
  const path = layoutCurrentOutputPath(context, relative(catalogRoot, containedPath(sourceTree, sourcePath)));
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
      layoutCurrentSourcePath(context, 'zircon_runtime/assets/fonts/editor-ui-sources'),
    )
  ).map((name) => `zircon_runtime/assets/fonts/editor-ui-sources/${name}`),
];
const nativeFonts = [];
for (const sourcePath of nativeFontSources)
  nativeFonts.push(
    await sourceResource(
      sourcePath,
      bytesSha256(await readFile(layoutCurrentSourcePath(context, sourcePath))),
    ),
  );
for (const entry of catalog.entries) {
  const directory = layoutCurrentOutputPath(context, dirname(entry.outputPath));
  assert.equal(
    bytesSha256(await readFile(layoutCurrentOutputPath(context, entry.outputPath))),
    entry.sourceSha256,
    `Delivery mirror differs from source: ${entry.sourcePath}`,
  );
  if (entry.penpotInputPath)
    assert.equal(
      bytesSha256(
        await readFile(layoutCurrentOutputPath(context, entry.penpotInputPath)),
      ),
      entry.penpotInputSha256,
      `Stale Penpot input: ${entry.sourcePath}`,
    );
  for (const reviewCase of entry.cases ?? [])
    if (reviewCase.reviewHost)
      assert.equal(
        bytesSha256(
          await readFile(
            layoutCurrentOutputPath(context, reviewCase.reviewHost.path),
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
      const font = currentDeliveryFont(context, path);
      if (!font) continue;
      assert.equal(
        bytesSha256(await readFile(font)),
        sha256,
        `Font changed: ${path}`,
      );
      fonts.set(path, {
        path: slash(relative(directory, font)),
        sha256,
      });
    }
  }
  const local = (resource: typeof source) => ({
    ...resource,
    path: slash(relative(directory, layoutCurrentOutputPath(context, resource.path))),
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
            relative(directory, layoutCurrentOutputPath(context, entry.penpotInputPath)),
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
    layoutCurrentOutputPath(context, 'resources/source-manifest.json'),
    `${JSON.stringify(manifest, null, 2)}\n`,
  );
else
  assert.deepEqual(
    JSON.parse(
      await readFile(
        layoutCurrentOutputPath(context, 'resources/source-manifest.json'),
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

}

const slashPath = (path: string): string => path.replaceAll('\\', '/');

function currentDeliveryFont(context: LayoutCurrentContext, path: string): string | undefined {
  // Current capture records absolute output font paths; ordinary runtime source
  // assets remain part of the source closure, never historical catalog fonts.
  if (!/^(?:[A-Za-z]:[/\\]|\/)/.test(path)) {
    if (/^docs\/_data(?:\/|$)/i.test(slashPath(path)))
      throw new Error('Historical font cannot seed current delivery');
    layoutCurrentSourcePath(context, path);
    return undefined;
  }
  const font = layoutCurrentOutputPath(context, path);
  containedPath(layoutCurrentOutputPath(context, 'resources/fonts'), font);
  return font;
}

/** The replay variant validates source identities without reading checkout bytes. */
export async function readCurrentDeliveryCatalog(
  context: LayoutCurrentContext,
  verifySources = true,
): Promise<CatalogManifest> {
  const catalog = JSON.parse(await readFile(layoutCurrentOutputPath(context, 'catalog.json'), 'utf8')) as CatalogManifest & { role?: string; historical_only?: boolean };
  if (catalog.schema !== 'dev.zircon.zui.layout-catalog' || catalog.version !== 1 ||
      catalog.historical_only || (catalog.role && catalog.role !== 'current') ||
      !Array.isArray(catalog.entries) || catalog.sourceCount !== catalog.entries.length)
    throw new Error('Historical or package catalog cannot seed current delivery');
  assertLayoutCurrentRoots(context, catalog.repoRoot, context.outputRoot);
  const sources = new Set<string>();
  const outputs = new Set<string>();
  for (const entry of catalog.entries) {
    if (sources.has(entry.sourcePath)) throw new Error('Duplicate delivery source');
    sources.add(entry.sourcePath);
    for (const [path, hash] of [
      [entry.sourcePath, entry.sourceSha256],
      ...(entry.dependencyFingerprints ?? []).map((item) => [item.sourcePath, item.sha256]),
    ] as Array<[string, string]>) {
      const source = layoutCurrentSourcePath(context, path);
      layoutCurrentOutputPath(context, `resources/source/${path}`);
      if (verifySources) assert.equal(bytesSha256(await readFile(source)), hash,
        `Source changed after catalog generation: ${path}`);
    }
    for (const reviewCase of entry.cases ?? [])
      if (reviewCase.themeSourcePath) layoutCurrentSourcePath(context, reviewCase.themeSourcePath);
    for (const path of [entry.outputPath, entry.resultPath, entry.previewPath,
      entry.penpotInputPath, entry.penpotPreviewPath,
      ...(entry.cases ?? []).map((item) => item.reviewHost?.path),
      ...[...(entry.penpotEvidence ?? []), ...(entry.engineEvidence ?? [])]
        .flatMap((item) => [item.screenshotPath, item.geometryPath, item.textPath]),
    ]) if (path) layoutCurrentOutputPath(context, path);
    for (const path of [entry.outputPath, entry.resultPath, entry.previewPath]) {
      const destination = layoutCurrentOutputPath(context, path);
      if (outputs.has(destination)) throw new Error('Duplicate delivery output');
      outputs.add(destination);
    }
    for (const path of ['resources.json', 'review-cases.json'])
      layoutCurrentOutputPath(context, `${dirname(entry.outputPath)}/${path}`);
    assert.equal(bytesSha256(await readFile(layoutCurrentOutputPath(context, entry.outputPath))),
      entry.sourceSha256, `Delivery mirror differs from source: ${entry.sourcePath}`);
    if (entry.penpotInputPath)
      assert.equal(bytesSha256(await readFile(layoutCurrentOutputPath(context, entry.penpotInputPath))),
        entry.penpotInputSha256, `Stale Penpot input: ${entry.sourcePath}`);
    for (const reviewCase of entry.cases ?? [])
      if (reviewCase.reviewHost)
        assert.equal(bytesSha256(await readFile(layoutCurrentOutputPath(context, reviewCase.reviewHost.path))),
          reviewCase.reviewHost.sha256, `Stale review host: ${entry.sourcePath}/${reviewCase.id}`);
    for (const evidence of entry.penpotEvidence ?? []) {
      if (evidence.sourceSha256 !== entry.sourceSha256 || evidence.dependencySha256 !== entry.dependencySha256) continue;
      for (const [path, hash] of evidence.runtimeAssetFingerprints ?? []) {
        const font = currentDeliveryFont(context, path);
        if (font) assert.equal(bytesSha256(await readFile(font)), hash, `Font changed: ${path}`);
      }
    }
  }
  for (const path of ['resources/source', 'resources/source-manifest.json',
    'evidence/delivery-replay.json', 'evidence/delivery-mismatch.zui']) layoutCurrentOutputPath(context, path);
  try { await packageFiles(context); } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
  }
  if (verifySources) {
    const fonts = 'zircon_runtime/assets/fonts/';
    for (const path of ['editor-ui.font.toml', 'editor-ui.ttc',
      ...(await readdir(layoutCurrentSourcePath(context, fonts + 'editor-ui-sources')))
        .map((name) => 'editor-ui-sources/' + name)]) {
      layoutCurrentOutputPath(context, 'resources/source/' + fonts + path);
      await readFile(layoutCurrentSourcePath(context, fonts + path));
    }
  }
  return catalog;
}

interface DeliveryPackage {
  schema: string;
  version: number;
  sourceCount: number;
  files: Array<{ sourcePath: string; path: string; sha256: string }>;
  entries: Array<{ sourcePath: string; resourceMap: string }>;
}

async function packageFiles(context: LayoutCurrentContext): Promise<Set<string>> {
  const root = layoutCurrentOutputPath(context, 'resources/source');
  const actual = new Set<string>();
  const visit = async (directory: string): Promise<void> => {
    const stat = await lstat(directory);
    if (stat.isSymbolicLink()) throw new Error('Source package contains a symbolic link');
    for (const item of await readdir(directory, { withFileTypes: true })) {
      const path = layoutCurrentOutputPath(context, relative(context.outputRoot, resolve(directory, item.name)));
      if (item.isSymbolicLink()) throw new Error('Source package contains a symbolic link');
      if (item.isDirectory()) await visit(path);
      else {
        assert.ok(item.isFile(), 'Source package contains a non-file');
        actual.add(slashPath(relative(root, path)));
      }
    }
  };
  await visit(root);
  return actual;
}

/** Validate the actual package tree before any parser can consume it. */
export async function validateCurrentDeliveryPackage(context: LayoutCurrentContext, catalog: CatalogManifest): Promise<void> {
  const root = layoutCurrentOutputPath(context, 'resources/source');
  const manifest = JSON.parse(await readFile(layoutCurrentOutputPath(context, 'resources/source-manifest.json'), 'utf8')) as DeliveryPackage;
  if (manifest.schema !== 'dev.zircon.zui.layout-source-package' || manifest.version !== 1 ||
      manifest.sourceCount !== catalog.sourceCount || !Array.isArray(manifest.files) || !Array.isArray(manifest.entries))
    throw new Error('Invalid source package manifest');
  const actual = await packageFiles(context);
  const required = new Map<string, string>();
  for (const entry of catalog.entries)
    for (const [path, hash] of [[entry.sourcePath, entry.sourceSha256],
      ...(entry.dependencyFingerprints ?? []).map((item) => [item.sourcePath, item.sha256])] as Array<[string, string]>) {
      if (required.has(path)) assert.equal(required.get(path), hash, `Conflicting dependency hashes: ${path}`);
      required.set(path, hash);
    }
  const declared = new Map<string, string>();
  for (const file of manifest.files) {
    assert.ok(typeof file.sourcePath === 'string' && typeof file.path === 'string' && /^[a-f0-9]{64}$/.test(file.sha256), 'Invalid package file identity');
    const path = containedPath(root, file.sourcePath);
    layoutCurrentSourcePath(context, file.sourcePath);
    assert.ok(required.has(file.sourcePath) ||
      /^zircon_runtime\/assets\/fonts\/(?:editor-ui\.(?:font\.toml|ttc)|editor-ui-sources\/[^/]+)$/.test(file.sourcePath),
      'Unrelated source is outside the delivery closure');
    assert.equal(slashPath(relative(root, path)), file.sourcePath, 'Noncanonical package source path');
    assert.equal(file.path, 'resources/source/' + file.sourcePath, 'Package file role differs');
    assert.ok(!declared.has(file.sourcePath), 'Duplicate package source');
    declared.set(file.sourcePath, file.sha256);
    assert.equal(bytesSha256(await readFile(layoutCurrentOutputPath(context, file.path))), file.sha256,
      `Packaged dependency differs: ${file.sourcePath}`);
  }
  assert.deepEqual([...actual].sort(), [...declared.keys()].sort(), 'Source package file inventory differs');
  assert.deepEqual(manifest.entries, catalog.entries.map((entry) => ({
    sourcePath: entry.sourcePath,
    resourceMap: slashPath(relative(context.outputRoot, resolve(context.outputRoot, dirname(entry.outputPath), 'resources.json'))),
  })), 'Source package entries differ from current catalog');
  for (const [path, hash] of required)
    assert.equal(declared.get(path), hash, `Package source identity differs: ${path}`);
  for (const path of ['zircon_runtime/assets/fonts/editor-ui.font.toml', 'zircon_runtime/assets/fonts/editor-ui.ttc'])
    assert.ok(declared.has(path), `Missing native font package resource: ${path}`);
}

export async function runCurrentLayoutDelivery(args: string[] = []): Promise<void> {
  const context = createLayoutCurrentContext();
  await readCurrentDeliveryCatalog(context);
  // Check performs no lock-directory mutation, matching the catalog check contract.
  if (args.includes('--check')) {
    await deliver(await readCurrentDeliveryCatalog(context), context, false);
    await validateCurrentDeliveryPackage(context, await readCurrentDeliveryCatalog(context));
    return;
  }
  await withLayoutCatalogLock(context.outputRoot, { command: 'zui-layout-delivery', context }, async () => {
    await deliver(await readCurrentDeliveryCatalog(context), context, true);
    await validateCurrentDeliveryPackage(context, await readCurrentDeliveryCatalog(context));
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href)
  void runCurrentLayoutDelivery(process.argv.slice(2)).catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
