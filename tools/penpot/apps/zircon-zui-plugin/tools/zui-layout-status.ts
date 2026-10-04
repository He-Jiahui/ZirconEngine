import { readFile, readdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { CatalogManifest } from './zui-layout-catalog';
import {
  currentDualReview,
  fileSha256,
  verifyCurrentFiles,
} from './zui-layout-evidence';
import { withLayoutCatalogLock } from './zui-layout-catalog-lock';

const root = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '../../../../../docs/_data/layout',
);
const write = !process.argv.includes('--check');

async function run(): Promise<void> {
  const catalog: CatalogManifest = JSON.parse(
    await readFile(resolve(root, 'catalog.json'), 'utf8'),
  );
  const current = new Set<string>();
  const stale: { sourcePath: string; reason: string }[] = [];
  for (const entry of catalog.entries) {
    if (!entry.penpotEvidence?.length && !entry.engineEvidence?.length)
      continue;
    try {
      await verifyCurrentFiles(
        entry,
        catalog.repoRoot,
        root,
        entry.review?.status === 'accepted',
      );
      current.add(entry.sourcePath);
    } catch (error) {
      stale.push({ sourcePath: entry.sourcePath, reason: String(error) });
    }
  }
  const entries = catalog.entries.filter((entry) =>
    current.has(entry.sourcePath),
  );
  const reviews = entries.filter(currentDualReview);
  const penpotPassed = entries.filter((entry) => {
    const cases = entry.cases;
    return (
      entry.visualStatus === 'passed' &&
      cases !== undefined &&
      cases.length > 0 &&
      cases.every((reviewCase) =>
        entry.penpotEvidence?.some(
          (item) => item.caseId === reviewCase.id && item.status === 'passed',
        ),
      )
    );
  }).length;
  const dualPassed = entries.filter(
    (entry) => entry.visualStatus === 'passed' && currentDualReview(entry),
  ).length;
  const failed = entries.filter(
    (entry) => entry.visualStatus === 'failed' && !currentDualReview(entry),
  ).length;
  const status = {
    schema: 'dev.zircon.zui.layout-penpot-batch-status',
    generatedAt: new Date().toISOString(),
    totalEntries: catalog.sourceCount,
    preparedEntries: catalog.entries.filter(
      (entry) => entry.status === 'prepared',
    ).length,
    penpotPassedEntries: penpotPassed,
    penpotPendingEntries: catalog.sourceCount - penpotPassed - failed,
    // Preserve the existing dual-end acceptance gate separately from the
    // Penpot-only batch count; a screenshot is not a reviewed engine case.
    visualPassedEntries: dualPassed,
    visualFailedEntries: failed,
    visualPendingEntries: catalog.sourceCount - dualPassed - failed,
    reviewedEntries: reviews.length,
    engineAcceptedEntries: reviews.filter(
      (entry) => entry.review?.status === 'accepted',
    ).length,
    penpotCasesPassed: entries.reduce(
      (count, entry) =>
        count +
        (entry.penpotEvidence?.filter((item) => item.status === 'passed')
          .length ?? 0),
      0,
    ),
    penpotCasesPending: entries.reduce(
      (count, entry) =>
        count +
        (entry.penpotEvidence?.filter((item) => item.status === 'pending')
          .length ?? 0),
      0,
    ),
    engineCasesPassed: entries.reduce(
      (count, entry) =>
        count +
        (entry.engineEvidence?.filter((item) => item.status === 'passed')
          .length ?? 0),
      0,
    ),
    failureExamples: entries
      .filter((entry) => entry.visualStatus === 'failed')
      .map((entry) => ({
        sourcePath: entry.sourcePath,
        error: entry.visualError,
      })),
    pendingExamples: entries
      .filter((entry) => entry.visualStatus === 'pending')
      .flatMap((entry) =>
        (entry.penpotEvidence ?? [])
          .filter((item) => item.status === 'pending')
          .map((item) => ({
            sourcePath: entry.sourcePath,
            caseId: item.caseId,
            reason: item.pendingReason,
          })),
      ),
    staleEvidence: stale,
    nativeBlockerEvidence: 'native-validation-blockers-20260921.md',
  };
  if (write) {
    await writeFile(
      resolve(root, 'evidence/penpot-batch-status.json'),
      `${JSON.stringify(status, null, 2)}\n`,
    );
    const files = [];
    for (const entry of await readdir(resolve(root, 'evidence'), {
      withFileTypes: true,
    })) {
      if (!entry.isFile() || entry.name === 'manifest.json') continue;
      files.push({
        path: entry.name,
        sha256: await fileSha256(resolve(root, 'evidence', entry.name)),
      });
    }
    files.sort((a, b) => a.path.localeCompare(b.path));
    await writeFile(
      resolve(root, 'evidence/manifest.json'),
      `${JSON.stringify(
        {
          schema: 'dev.zircon.zui.layout-evidence-manifest',
          generatedAt: new Date().toISOString(),
          files,
        },
        null,
        2,
      )}\n`,
    );
  }
  console.log(
    JSON.stringify({
      ...status,
      staleEvidence: stale.length,
      failureExamples: status.failureExamples.length,
      pendingExamples: status.pendingExamples.length,
      wrote: write,
    }),
  );
}

if (write) {
  await withLayoutCatalogLock(
    root,
    { command: `zui-layout-status ${process.argv.slice(2).join(' ')}` },
    run,
  );
} else {
  await run();
}
