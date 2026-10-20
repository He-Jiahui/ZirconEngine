import { assertLayoutCurrentRoots, createLayoutCurrentContext, layoutCurrentSourcePath, layoutCurrentOutputPath, resolveLayoutCatalogPath } from './zui-layout-paths';
import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
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
  verifyCurrentFiles,
  dualEvidenceErrors,
  caseReviewErrors,
  renderEvidenceSha256,
} from './zui-layout-evidence';
import { withLayoutCatalogLock } from './zui-layout-catalog-lock';

const rendererRepoEnvironment = process.env['ZUI_LAYOUT_REPO_ROOT'];
const rendererOutputEnvironment = process.env['ZUI_LAYOUT_OUTPUT_ROOT'];

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
  // Fail before reading a batch file when output has not been explicitly selected.
  createLayoutCurrentContext();
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
  await runCurrentLayoutReview(inputs);
}

/** The CLI and renderer module share the same explicit environment roots. */
export async function runCurrentLayoutReview(inputs: ReviewInput[]): Promise<void> {
  const context = createLayoutCurrentContext();
  if (process.env['ZUI_LAYOUT_REPO_ROOT'] !== rendererRepoEnvironment ||
      process.env['ZUI_LAYOUT_OUTPUT_ROOT'] !== rendererOutputEnvironment)
    throw new Error('Review environment changed after renderer initialization');
  const catalogRoot = context.outputRoot;
  const loadManifest = async (): Promise<CatalogManifest> => {
    const manifest = JSON.parse(
      await readFile(layoutCurrentOutputPath(context, 'catalog.json'), 'utf8'),
    ) as CatalogManifest & { historical_only?: boolean; role?: string };
    if (manifest.schema !== 'dev.zircon.zui.layout-catalog' || manifest.version !== 1 ||
        manifest.historical_only || (manifest.role && manifest.role !== 'current'))
      throw new Error('Historical or package catalog cannot be reviewed as current');
    assertLayoutCurrentRoots(context, manifest.repoRoot, catalogRoot);
    for (const input of inputs) {
      const entry = manifest.entries.find((candidate) => candidate.sourcePath === input.sourcePath);
      if (!entry) throw new Error(`No catalog entry for ${input.sourcePath}`);
      layoutCurrentSourcePath(context, entry.sourcePath);
      for (const path of [entry.resultPath, entry.outputPath, entry.previewPath,
        entry.penpotInputPath, entry.penpotPreviewPath,
        ...(entry.cases ?? []).map((item) => item.reviewHost?.path),
        ...[...(entry.penpotEvidence ?? []), ...(entry.engineEvidence ?? [])]
          .flatMap((item) => [item.screenshotPath, item.geometryPath, item.textPath])])
        if (path) {
          const current = layoutCurrentOutputPath(context, path);
          if (resolveLayoutCatalogPath(catalogRoot, path) !== current)
            throw new Error('Legacy renderer would rewrite current output path');
        }
      for (const dependency of entry.dependencyFingerprints ?? [])
        layoutCurrentSourcePath(context, dependency.sourcePath);
    }
    return manifest;
  };
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
  // Reject invalid identities and paths before creating even the writer lock.
  await loadManifest();
  await withLayoutCatalogLock(
    catalogRoot,
    { command: `zui-layout-review ${process.argv.slice(2).join(' ')}`, context },
    async () => {
      const manifest = await loadManifest();
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
          context,
        );
      }
      for (const input of inputs) {
        const entry = manifest.entries.find(
          (candidate) => candidate.sourcePath === input.sourcePath,
        )!;
        recordReview(entry, input);
        await writeFile(
          layoutCurrentOutputPath(context, entry.resultPath),
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
        layoutCurrentOutputPath(context, 'catalog.json'),
        `${JSON.stringify(manifest, null, 2)}\n`,
      );
      await writeFile(layoutCurrentOutputPath(context, 'index.md'), renderIndex(manifest, context));
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
