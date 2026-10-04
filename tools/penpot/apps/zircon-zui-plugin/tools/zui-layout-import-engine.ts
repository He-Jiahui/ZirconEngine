import { resolveLayoutCatalogPath } from './zui-layout-paths';
import { copyFile, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import {
  type CatalogManifest,
  renderIndex,
  renderResult,
} from './zui-layout-catalog';
import {
  caseSha256,
  requiredNativeRenderer,
  validateLayoutReviewCase,
  type LayoutRenderEvidence,
} from './zui-layout-review-contract';
import { containedPath, verifyCurrentFiles } from './zui-layout-evidence';
import { withLayoutCatalogLock } from './zui-layout-catalog-lock';

interface NativeReport {
  schema: 'dev.zircon.zui.native-evidence';
  version: 1;
  rendererSha256: string;
  evidence: Array<LayoutRenderEvidence & { sourcePath: string }>;
}

/**
 * Pick the deterministic engine frame used as the catalogue's primary
 * preview.  The case matrix deliberately contains several states and
 * viewport sizes; the primary must remain a real product frame, never a
 * synthetic crop or a Penpot projection.  Prefer the default 1x case at the
 * host's largest canonical viewport and fall back to the first passed default
 * case when a partial capture is imported.
 */
function primaryEngineEvidence(
  entry: CatalogManifest['entries'][number],
): LayoutRenderEvidence | undefined {
  const candidates = (entry.engineEvidence ?? []).filter(
    (item) => item.status === 'passed' && item.screenshotPath,
  );
  if (!candidates.length) return undefined;
  const score = (item: LayoutRenderEvidence): number => {
    const reviewCase = entry.cases?.find(
      (candidate) => candidate.id === item.caseId,
    );
    if (!reviewCase) return -1;
    let value = 0;
    if (reviewCase.state === 'default') value += 1000;
    if (reviewCase.dpi === 1) value += 100;
    value += (reviewCase.viewport.width * reviewCase.viewport.height) / 1000;
    return value;
  };
  return [...candidates].sort((a, b) => score(b) - score(a))[0];
}

// Validate the complete incoming batch before replacing any capture records.
export async function importEngineReport(
  manifest: CatalogManifest,
  value: unknown,
  catalogRoot: string,
): Promise<void> {
  const report = value as NativeReport;
  if (
    report?.schema !== 'dev.zircon.zui.native-evidence' ||
    report.version !== 1 ||
    !Array.isArray(report.evidence) ||
    !report.evidence.length ||
    !/^[0-9a-f]{64}$/.test(report.rendererSha256)
  )
    throw new Error('Invalid native evidence report');
  const identities = new Set<string>();
  for (const item of report.evidence) {
    const identity = JSON.stringify([item.sourcePath, item.caseId]);
    if (identities.has(identity))
      throw new Error(`Duplicate native evidence: ${identity}`);
    identities.add(identity);
    const entry = manifest.entries.find(
      (entry) => entry.sourcePath === item.sourcePath,
    );
    const reviewCase = entry?.cases?.find(
      (reviewCase) => reviewCase.id === item.caseId,
    );
    if (!entry || !reviewCase)
      throw new Error(`Unknown native evidence: ${identity}`);
    try {
      validateLayoutReviewCase(reviewCase);
    } catch (error) {
      throw new Error(
        `Invalid catalog review case ${identity}: ${String(error)}`,
        {
          cause: error,
        },
      );
    }
    if (
      !['passed', 'pending', 'failed'].includes(item.status) ||
      item.rendererSha256 !== report.rendererSha256 ||
      !['zircon-runtime-wgpu-headless', 'zircon-editor-retained-host'].includes(
        item.rendererKind ?? '',
      ) ||
      !item.rendererPath ||
      item.sourceSha256 !== entry.sourceSha256 ||
      item.dependencySha256 !== entry.dependencySha256 ||
      item.caseSha256 !== caseSha256(reviewCase)
    )
      throw new Error(`Stale or invalid native evidence: ${identity}`);
    if (item.rendererKind !== requiredNativeRenderer(reviewCase))
      throw new Error(
        `Wrong native host for ${identity}: expected ${requiredNativeRenderer(reviewCase)}, received ${item.rendererKind}`,
      );
    if (item.status === 'failed' && !item.error?.trim())
      throw new Error(`Native failure has no explanation: ${identity}`);
    if (item.status === 'pending' && !item.pendingReason?.trim())
      throw new Error(
        `Pending native evidence has no explanation: ${identity}`,
      );
    if (
      (item.status === 'passed' || item.status === 'pending') &&
      (!item.geometryPath ||
        !item.geometrySha256 ||
        !item.textPath ||
        !item.textSha256)
    )
      throw new Error(`Native capture lacks geometry or text: ${identity}`);
    await verifyCurrentFiles(
      { ...entry, penpotEvidence: [], engineEvidence: [item] },
      manifest.repoRoot,
      catalogRoot,
      false,
    );
  }
  for (const item of report.evidence) {
    const entry = manifest.entries.find(
      (entry) => entry.sourcePath === item.sourcePath,
    )!;
    entry.engineEvidence = [
      ...(entry.engineEvidence ?? []).filter(
        (previous) => previous.caseId !== item.caseId,
      ),
      structuredClone(item),
    ];
  }

  // Keep the required per-entry preview artifact synchronized with the
  // imported native evidence.  This is intentionally a byte-for-byte copy of
  // a captured engine frame; no resizing, compositing, or filename-based
  // content substitution is allowed at this boundary.
  for (const entry of manifest.entries) {
    const primary = primaryEngineEvidence(entry);
    if (!primary) continue;
    const source = containedPath(catalogRoot, primary.screenshotPath);
    const destination = containedPath(catalogRoot, entry.previewPath);
    if (source !== destination) await copyFile(source, destination);
  }
}

async function main() {
  const [catalogPath, reportPath] = process.argv.slice(2);
  if (!catalogPath || !reportPath)
    throw new Error('Expected catalog.json and engine-report.json paths');
  const catalogRoot = dirname(resolve(catalogPath));
  await withLayoutCatalogLock(
    catalogRoot,
    {
      command: `zui-layout-import-engine ${process.argv.slice(2).join(' ')}`,
    },
    async () => {
      const manifest = JSON.parse(
        await readFile(resolve(catalogPath), 'utf8'),
      ) as CatalogManifest;
      await importEngineReport(
        manifest,
        JSON.parse(await readFile(resolve(reportPath), 'utf8')),
        catalogRoot,
      );
      await writeFile(
        resolve(catalogPath),
        `${JSON.stringify(manifest, null, 2)}\n`,
      );
      for (const entry of manifest.entries)
        await writeFile(
          containedPath(catalogRoot, entry.resultPath),
          renderResult(entry),
        );
      await writeFile(resolveLayoutCatalogPath(catalogRoot, 'index.md'), renderIndex(manifest));
    },
  );
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(resolve(process.argv[1])).href
)
  void main().catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
