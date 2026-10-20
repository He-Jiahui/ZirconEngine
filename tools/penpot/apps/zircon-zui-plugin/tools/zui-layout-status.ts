import { readdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import type { CatalogManifest } from './zui-layout-catalog';
import {
  currentDualReview,
  fileSha256,
  verifyCurrentFiles,
} from './zui-layout-evidence';
import { withLayoutCatalogLock } from './zui-layout-catalog-lock';

import { createLayoutCurrentContext, layoutCurrentOutputPath } from './zui-layout-paths';
import { readCurrentDeliveryCatalog } from './zui-layout-delivery';

export async function runCurrentLayoutStatus(args: string[] = []): Promise<void> {
  if (args.some((arg) => arg !== '--check'))
    throw new Error('Only current status and --check are supported; historical replay requires a fixed read-only inventory');
  const context = createLayoutCurrentContext();
  const root = context.outputRoot;
  const write = !args.includes('--check');
  const preflight = async (): Promise<CatalogManifest> => {
    const catalog = await readCurrentDeliveryCatalog(context);
    layoutCurrentOutputPath(context, 'evidence/penpot-batch-status.json');
    layoutCurrentOutputPath(context, 'evidence/manifest.json');
    if (write) {
      for (const entry of catalog.entries) {
        if (!entry.penpotEvidence?.length && !entry.engineEvidence?.length) continue;
        await verifyCurrentFiles(entry, context.repoRoot, root, entry.review?.status === 'accepted', context, 'strict-capture');
      }
    }
    return catalog;
  };
  await preflight();
async function run(): Promise<void> {
  const catalog = await preflight();
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
        context,
        'strict-capture',
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
      layoutCurrentOutputPath(context, 'evidence/penpot-batch-status.json'),
      `${JSON.stringify(status, null, 2)}\n`,
    );
    const files = [];
    for (const entry of await readdir(layoutCurrentOutputPath(context, 'evidence'), {
      withFileTypes: true,
    })) {
      if (!entry.isFile() || entry.name === 'manifest.json') continue;
      files.push({
        path: entry.name,
        sha256: await fileSha256(layoutCurrentOutputPath(context, `evidence/${entry.name}`)),
      });
    }
    files.sort((a, b) => a.path.localeCompare(b.path));
    await writeFile(
      layoutCurrentOutputPath(context, 'evidence/manifest.json'),
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
    { command: `zui-layout-status ${args.join(' ')}`, context },
    run,
  );
} else {
  await run();
}
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href)
  void runCurrentLayoutStatus(process.argv.slice(2)).catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
