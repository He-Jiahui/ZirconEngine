import { resolveLayoutCatalogPath } from './zui-layout-paths';
import { createHash } from 'node:crypto';
import { readFile, rename } from 'node:fs/promises';
import { isAbsolute, relative, resolve } from 'node:path';
import type { CatalogEntry } from './zui-layout-catalog';
import { caseSha256 } from './zui-layout-review-contract';

export type CatalogEvidenceSnapshot = Pick<
  CatalogEntry,
  | 'category'
  | 'name'
  | 'sourceSha256'
  | 'outputSha256'
  | 'dependencySha256'
  | 'penpotInputSha256'
  | 'previewPath'
  | 'penpotPreviewPath'
  | 'cases'
  | 'penpotEvidence'
  | 'engineEvidence'
>;

export async function retainExpandedCaseEvidence(
  previous: CatalogEvidenceSnapshot,
  next: CatalogEvidenceSnapshot,
  outputRoot: string,
  shouldWrite: boolean,
): Promise<Pick<CatalogEntry, 'penpotEvidence' | 'engineEvidence'>> {
  const stable =
    previous.category === next.category &&
    previous.name === next.name &&
    previous.sourceSha256 === next.sourceSha256 &&
    previous.outputSha256 === next.outputSha256 &&
    previous.dependencySha256 === next.dependencySha256 &&
    previous.penpotInputSha256 === next.penpotInputSha256 &&
    previous.previewPath === next.previewPath &&
    previous.penpotPreviewPath === next.penpotPreviewPath;
  if (!stable || !previous.cases || !next.cases) return {};

  const directory = resolveLayoutCatalogPath(outputRoot, next.category, next.name);
  const nextCases = new Map(next.cases.map((item) => [item.id, item]));
  const previousCases = new Map(previous.cases.map((item) => [item.id, item]));
  const pendingMoves: Array<[string, string]> = [];
  const retained: Pick<CatalogEntry, 'penpotEvidence' | 'engineEvidence'> = {};

  for (const [field, renderer, primaryPath] of [
    ['penpotEvidence', 'penpot', previous.penpotPreviewPath],
    ['engineEvidence', 'engine', previous.previewPath],
  ] as const) {
    const items: NonNullable<CatalogEntry[typeof field]> = [];
    for (const evidence of previous[field] ?? []) {
      const reviewCase = nextCases.get(evidence.caseId);
      const oldCase = previousCases.get(evidence.caseId);
      if (
        !reviewCase ||
        !oldCase ||
        evidence.status === 'failed' ||
        evidence.sourceSha256 !== next.sourceSha256 ||
        evidence.dependencySha256 !== next.dependencySha256 ||
        evidence.caseSha256 !== caseSha256(oldCase) ||
        evidence.caseSha256 !== caseSha256(reviewCase) ||
        (reviewCase.reviewHost &&
          evidence.reviewHostSha256 !== reviewCase.reviewHost.sha256) ||
        (renderer === 'penpot' &&
          evidence.inputSha256 !== next.penpotInputSha256) ||
        (renderer === 'engine' &&
          evidence.inputSha256 !== undefined &&
          evidence.inputSha256 !== next.sourceSha256)
      )
        continue;

      // Old primary pixels must never occupy the newly selected primary slot.
      const displacedPrimary =
        evidence.caseId === previous.cases[0]?.id &&
        evidence.caseId !== next.cases[0]?.id &&
        evidence.screenshotPath === primaryPath;
      const archivedPath = displacedPrimary
        ? `${next.category}/${next.name}/evidence/former-primary-${renderer}-${evidence.caseId}-${evidence.screenshotSha256.slice(0, 12)}.png`
        : evidence.screenshotPath;
      if (displacedPrimary && !/^[a-zA-Z0-9_-]+$/.test(evidence.caseId))
        throw new Error(`Unsafe primary case id: ${evidence.caseId}`);
      const from = resolveLayoutCatalogPath(outputRoot, evidence.screenshotPath);
      const to = resolveLayoutCatalogPath(outputRoot, archivedPath);
      for (const path of [from, to]) {
        const withinEntry = relative(directory, path);
        if (
          !withinEntry ||
          withinEntry === '..' ||
          withinEntry.startsWith('..\\') ||
          withinEntry.startsWith('../') ||
          isAbsolute(withinEntry)
        )
          throw new Error(`Evidence path escapes catalog entry: ${path}`);
      }
      const actualHash = createHash('sha256')
        .update(await readFile(from))
        .digest('hex');
      if (actualHash !== evidence.screenshotSha256)
        throw new Error(
          `Evidence screenshot hash differs: ${evidence.screenshotPath}`,
        );
      if (displacedPrimary) pendingMoves.push([from, to]);
      items.push({ ...evidence, screenshotPath: archivedPath });
    }
    if (items.length) retained[field] = items;
  }

  if (shouldWrite) {
    // Check every destination before the first move, leaving previous evidence
    // intact if a prior migration or user file already occupies an archive name.
    for (const [, to] of pendingMoves) {
      try {
        await readFile(to);
        throw new Error(`Evidence archive already exists: ${to}`);
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
      }
    }
    for (const [from, to] of pendingMoves) await rename(from, to);
  }
  return retained;
}
