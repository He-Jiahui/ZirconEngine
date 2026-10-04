import type {
  CatalogEntry,
  CatalogManifest,
  CatalogReview,
} from './zui-layout-catalog';
import { currentDualReview } from './zui-layout-evidence';
import { canonicalSha256 } from './zui-layout-review-contract';

export function assertCheckpointSources(
  capturing: CatalogManifest,
  disk: CatalogManifest,
): void {
  const identity = (manifest: CatalogManifest) =>
    canonicalSha256(
      manifest.entries
        .map((entry) => ({
          sourcePath: entry.sourcePath,
          sourceSha256: entry.sourceSha256,
          outputSha256: entry.outputSha256,
          dependencySha256: entry.dependencySha256,
          penpotInputSha256: entry.penpotInputSha256,
          cases: entry.cases,
        }))
        .sort((left, right) => left.sourcePath.localeCompare(right.sourcePath)),
    );
  if (identity(capturing) !== identity(disk))
    throw new Error(
      'Catalog sources or cases changed during capture; checkpoint refused. Restart capture from the regenerated catalog.',
    );
}

function appendHistory(entry: CatalogEntry, reviews: CatalogReview[]): void {
  const unique = new Map(
    [...(entry.reviewHistory ?? []), ...reviews].map((review) => [
      canonicalSha256(review),
      review,
    ]),
  );
  entry.reviewHistory = [...unique.values()].map((review) =>
    structuredClone(review),
  );
}

export function archiveReview(entry: CatalogEntry): void {
  if (!entry.review) return;
  appendHistory(entry, [entry.review]);
  delete entry.review;
}

/** Checkpoints can observe a review saved while the browser was capturing. */
export function mergeCheckpointReviews(
  entry: CatalogEntry,
  disk?: CatalogEntry,
): void {
  if (!currentDualReview(entry)) archiveReview(entry);
  if (!disk) return;
  appendHistory(entry, disk.reviewHistory ?? []);
  if (!disk.review) return;
  if (
    entry.visualStatus === 'passed' &&
    currentDualReview({ ...entry, review: disk.review })
  ) {
    if (!entry.review || disk.review.reviewedAt >= entry.review.reviewedAt) {
      if (
        entry.review &&
        canonicalSha256(entry.review) !== canonicalSha256(disk.review)
      )
        archiveReview(entry);
      entry.review = structuredClone(disk.review);
    } else {
      appendHistory(entry, [disk.review]);
    }
  } else {
    appendHistory(entry, [disk.review]);
  }
}
