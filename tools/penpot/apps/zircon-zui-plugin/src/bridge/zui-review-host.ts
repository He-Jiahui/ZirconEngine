import { validateZuiDocument, type ZuiDocument } from './zui-document';
import {
  reconcileZuiDocument,
  type PenpotAssetSnapshot,
} from './penpot-projection';
import { restoreOriginalZuiSource } from './zui-source-export';

export function reviewHost(source: ZuiDocument): ZuiDocument | null {
  const value = source['penpot_review_host'];
  if (value === undefined) return null;
  if (!['style', 'theme_tokens', 'component', 'view'].includes(source.asset.kind))
    throw new Error('Only view, theme, or component assets can declare a review host');
  const host = value as ZuiDocument;
  const errors = validateZuiDocument(host).filter(
    (item) => item.severity === 'error',
  );
  if (
    errors.length ||
    host.asset.kind !== 'view' ||
    !Object.keys(host.nodes ?? {}).length
  )
    throw new Error('Review host must be a valid nonempty ZUI view');
  return host;
}

export function reconcileReviewSource(
  source: ZuiDocument,
  snapshot: PenpotAssetSnapshot,
) {
  const host = reviewHost(source);
  const result = reconcileZuiDocument(host ?? source, snapshot);
  if (!host) return restoreOriginalZuiSource(source, result, snapshot);
  // Specimen geometry and slot content belong to the case, not its source asset.
  if (
    result.changes.length &&
    (source['penpot_original_source_path'] === undefined ||
      source['penpot_source_fingerprints'] === undefined)
  )
    throw new Error(
      'Review consumer edits require an explicit source mapping; re-import the review host',
    );
  return restoreOriginalZuiSource(
    source,
    result.changes.length ? result : { ...result, document: source },
    snapshot,
  );
}
