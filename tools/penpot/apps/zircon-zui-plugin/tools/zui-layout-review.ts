import { resolveLayoutCatalogPath } from './zui-layout-paths';
import { readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import {
  renderIndex,
  renderResult,
  type CatalogManifest,
  type CatalogEntry,
} from './zui-layout-catalog';
import {
  casesSha256,
  type LayoutCaseReview,
} from './zui-layout-review-contract';
import {
  containedPath,
  verifyCurrentFiles,
  dualEvidenceErrors,
  caseReviewErrors,
  renderEvidenceSha256,
} from './zui-layout-evidence';
import { withLayoutCatalogLock } from './zui-layout-catalog-lock';

export interface ReviewInput {
  sourcePath: string;
  decision: 'accepted' | 'needs_revision';
  observations: string;
  caseReviews?: LayoutCaseReview[];
}

export function validateReviewInput(
  entry: CatalogEntry,
  input: ReviewInput,
): void {
  if (
    input.sourcePath !== entry.sourcePath ||
    !['accepted', 'needs_revision'].includes(input.decision) ||
    !input.observations?.trim()
  )
    throw new Error(
      'Each review requires the matching source, decision, and concrete observations',
    );
  if (input.decision === 'accepted') {
    const errors = [
      ...dualEvidenceErrors(entry),
      ...caseReviewErrors(entry, input.caseReviews),
    ];
    if (errors.length) throw new Error(errors.join('; '));
  }
}

export function recordReview(entry: CatalogEntry, input: ReviewInput): void {
  validateReviewInput(entry, input);
  if (entry.review) (entry.reviewHistory ??= []).push(entry.review);
  const hashes = (kind: 'penpotEvidence' | 'engineEvidence') =>
    Object.fromEntries(
      (entry[kind] ?? []).map((item) => [item.caseId, item.screenshotSha256]),
    );
  entry.review = {
    contract: 'dual-renderer-v1',
    status: input.decision,
    sourceSha256: entry.sourceSha256,
    outputSha256: entry.outputSha256,
    dependencySha256: entry.dependencySha256,
    penpotInputSha256: entry.penpotInputSha256,
    casesSha256: casesSha256(entry.cases ?? []),
    screenshotSha256:
      entry.engineEvidence?.[0]?.screenshotSha256 ??
      entry.penpotEvidence?.[0]?.screenshotSha256 ??
      '',
    penpotScreenshotHashes: hashes('penpotEvidence'),
    engineScreenshotHashes: hashes('engineEvidence'),
    penpotEvidenceSha256: renderEvidenceSha256(entry.penpotEvidence),
    engineEvidenceSha256: renderEvidenceSha256(entry.engineEvidence),
    caseReviews: structuredClone(input.caseReviews ?? []),
    observations: input.observations,
    reviewedAt: new Date().toISOString(),
  };
}

async function main(): Promise<void> {
  const catalogRoot = resolve(
    process.env['ZUI_LAYOUT_OUTPUT_ROOT'] ??
      resolve(
        dirname(fileURLToPath(import.meta.url)),
        '../../../../../docs/_data/layout',
      ),
  );
  const [sourcePath, decision, ...notes] = process.argv.slice(2);
  const inputs: ReviewInput[] =
    sourcePath === '--batch'
      ? JSON.parse(await readFile(resolve(decision), 'utf8'))
      : [
          {
            sourcePath,
            decision: decision as ReviewInput['decision'],
            observations: notes.join(' '),
          },
        ];
  for (const input of inputs) {
    if (
      !input.sourcePath ||
      !['accepted', 'needs_revision'].includes(input.decision) ||
      !input.observations?.trim()
    )
      throw new Error(
        'Each review requires a source, decision, and concrete observations',
      );
  }
  await withLayoutCatalogLock(
    catalogRoot,
    { command: `zui-layout-review ${process.argv.slice(2).join(' ')}` },
    async () => {
      const manifest = JSON.parse(
        await readFile(resolveLayoutCatalogPath(catalogRoot, 'catalog.json'), 'utf8'),
      ) as CatalogManifest;
      // Validate the entire batch before recording any decisions.
      for (const input of inputs) {
        const entry = manifest.entries.find(
          (candidate) => candidate.sourcePath === input.sourcePath,
        );
        if (!entry) throw new Error(`No catalog entry for ${input.sourcePath}`);
        validateReviewInput(entry, input);
        await verifyCurrentFiles(
          entry,
          manifest.repoRoot,
          catalogRoot,
          input.decision === 'accepted',
        );
      }
      for (const input of inputs) {
        const entry = manifest.entries.find(
          (candidate) => candidate.sourcePath === input.sourcePath,
        )!;
        recordReview(entry, input);
        await writeFile(
          containedPath(catalogRoot, entry.resultPath),
          renderResult(entry),
          'utf8',
        );
        console.log(
          JSON.stringify({
            sourcePath: input.sourcePath,
            decision: input.decision,
          }),
        );
      }
      await writeFile(
        resolveLayoutCatalogPath(catalogRoot, 'catalog.json'),
        `${JSON.stringify(manifest, null, 2)}\n`,
      );
      await writeFile(resolveLayoutCatalogPath(catalogRoot, 'index.md'), renderIndex(manifest));
    },
  );
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(resolve(process.argv[1])).href
) {
  void main().catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
}
