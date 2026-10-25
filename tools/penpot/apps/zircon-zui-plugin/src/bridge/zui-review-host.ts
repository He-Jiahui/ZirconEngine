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
  // The outer document owns the source-export closure. Render and capture the
  // same pins in its editable host; nested metadata cannot replace that owner.
  const pins = source['penpot_source_fingerprints'];
  if (pins === undefined) return host;
  const canonicalPins = (value: unknown): string => {
    if (!Array.isArray(value)) throw new Error('Review host source fingerprints must be an array');
    const paths = new Set<string>();
    return JSON.stringify(value.map(pin => {
      if (!pin || typeof pin !== 'object' || Array.isArray(pin) ||
          typeof pin.sourcePath !== 'string' || !pin.sourcePath ||
          typeof pin.sha256 !== 'string' || !/^[a-f\d]{64}$/i.test(pin.sha256) ||
          paths.has(pin.sourcePath))
        throw new Error('Review host source fingerprints are invalid or duplicated');
      paths.add(pin.sourcePath);
      return [pin.sourcePath, pin.sha256.toLowerCase()];
    }).sort(([left], [right]) => left.localeCompare(right)));
  };
  const expected = canonicalPins(pins);
  const nested = host['penpot_source_fingerprints'];
  if (nested !== undefined && canonicalPins(nested) !== expected)
    throw new Error('Review host source fingerprints differ from its source owner');
  return { ...host, penpot_source_fingerprints: structuredClone(pins) };
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
