import type { ZuiDocument, ZuiNode, ZuiValue } from './zui-document';

/**
 * Collection states are authored on the source node through
 * `collection_test_cases`.  They are review-only materializations: the
 * source document is never changed and no business copy is selected by file
 * name.  Keeping the vocabulary here gives Penpot and the native capture
 * path one deterministic scenario contract.
 */
export const COLLECTION_REVIEW_CASES = [
  'empty',
  'one',
  'normal',
  'overflow',
  'narrow',
  'long-locale',
] as const;

export type CollectionReviewCase = (typeof COLLECTION_REVIEW_CASES)[number];

export interface CollectionReviewData {
  collectionCase: CollectionReviewCase;
}

const COLLECTION_CASE_SET = new Set<string>(COLLECTION_REVIEW_CASES);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function stringArray(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === 'string')
    : [];
}

function collectionCaseFrom(value: unknown): CollectionReviewCase | null {
  return typeof value === 'string' && COLLECTION_CASE_SET.has(value)
    ? (value as CollectionReviewCase)
    : null;
}

function collectionNodes(document: ZuiDocument): ZuiNode[] {
  return Object.values(document.nodes ?? {}).filter((node) => {
    const cases = stringArray(node.props?.['collection_test_cases']);
    return cases.some((value) => COLLECTION_CASE_SET.has(value));
  });
}

/** Return the authored collection matrix in stable source order. */
export function authoredCollectionReviewCases(
  document?: ZuiDocument,
): CollectionReviewCase[] {
  const result: CollectionReviewCase[] = [];
  const seen = new Set<CollectionReviewCase>();
  for (const node of collectionNodes(
    document ?? { asset: { kind: 'view', id: '', version: 2 } },
  )) {
    for (const value of stringArray(node.props?.['collection_test_cases'])) {
      const state = collectionCaseFrom(value);
      if (state && !seen.has(state)) {
        seen.add(state);
        result.push(state);
      }
    }
  }
  return result;
}

export function isCollectionReviewData(
  value: unknown,
): value is CollectionReviewData {
  return (
    isRecord(value) &&
    Object.keys(value).length === 1 &&
    collectionCaseFrom(value['collectionCase']) !== null
  );
}

function cloneValue(value: ZuiValue): ZuiValue {
  if (Array.isArray(value)) return value.map(cloneValue);
  if (value !== null && typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value).map(([key, child]) => [key, cloneValue(child)]),
    );
  }
  return value;
}

function delimitedLabel(value: string, state: CollectionReviewCase): string {
  const parts = value.split('|');
  if (state === 'long-locale') {
    const long =
      'A deliberately long localized collection label for overflow coverage';
    if (parts.length > 1) parts[1] = long;
    else parts[0] = long;
  }
  return parts.join('|');
}

function itemNodes(document: ZuiDocument, ownerId: string): string[] {
  const owner = document.nodes?.[ownerId];
  if (!owner?.children?.length) return [];
  const direct = owner.children
    .map((child) => child.node)
    .filter((nodeId) => Boolean(document.nodes?.[nodeId]));
  const explicit = direct.filter((nodeId) => {
    const props = document.nodes?.[nodeId]?.props;
    return typeof props?.['collection_item'] === 'string';
  });
  if (explicit.length) return explicit;

  // Some source-owned fixture lists expose their collection through an
  // authored `collection_items` array and ordinary Button children.  Keep
  // heading/add controls out of the transient list while preserving all
  // authored nodes in the source document.
  const candidates = direct.filter((nodeId) => {
    const node = document.nodes?.[nodeId];
    const text = node?.props?.['text'];
    return (
      node !== undefined &&
      (node.component === 'Button' ||
        node.component === 'Chip' ||
        node.component === 'Paper') &&
      typeof text === 'string' &&
      !/^add\b/i.test(text)
    );
  });
  const items = stringArray(owner.props?.['collection_items']);
  return items.length ? candidates.slice(0, items.length) : [];
}

function setVisibility(node: ZuiNode, visible: boolean): void {
  node.props ??= {};
  if (visible) delete node.props['visibility'];
  else node.props['visibility'] = 'collapsed';
}

function applyOwnerCollectionState(
  document: ZuiDocument,
  ownerId: string,
  state: CollectionReviewCase,
): void {
  const owner = document.nodes?.[ownerId];
  if (!owner?.props) return;
  owner.props['collection_review_case'] = state;
  // Resolve authored item nodes before changing collection_items.  Empty
  // materialization intentionally clears that array, but the source children
  // still identify which authored rows must become collapsed.
  const children = itemNodes(document, ownerId);

  const original = Array.isArray(owner.props['collection_items'])
    ? owner.props['collection_items'].map((value) => cloneValue(value))
    : [];
  if (Array.isArray(owner.props['collection_items'])) {
    if (state === 'empty') owner.props['collection_items'] = [];
    else if (state === 'one')
      owner.props['collection_items'] = original.slice(0, 1);
    else if (state === 'overflow') {
      const extra = original.slice(
        0,
        Math.max(1, Number(owner.props['visible_limit']) || 3),
      );
      owner.props['collection_items'] = [...original, ...extra];
    } else if (state === 'long-locale') {
      owner.props['collection_items'] = original.map((value) =>
        typeof value === 'string' ? delimitedLabel(value, state) : value,
      );
    }
  }
  if (state === 'overflow') owner.props['collection_overflow'] = true;
  if (state === 'narrow') owner.props['collection_review_width'] = 'narrow';

  const visibleCount =
    state === 'empty' ? 0 : state === 'one' ? 1 : children.length;
  children.forEach((nodeId, index) => {
    const node = document.nodes?.[nodeId];
    if (!node) return;
    setVisibility(node, index < visibleCount);
    if (state === 'long-locale' && typeof node.props?.['text'] === 'string') {
      node.props['text'] = delimitedLabel(node.props['text'], state);
    }
    if (
      state === 'long-locale' &&
      typeof node.props?.['collection_item'] === 'string'
    ) {
      node.props['collection_item'] = delimitedLabel(
        node.props['collection_item'],
        state,
      );
    }
  });
}

/** Apply a collection scenario to a review-only cloned document. */
export function applyCollectionReviewData(
  document: ZuiDocument,
  data: unknown,
): boolean {
  if (!isCollectionReviewData(data)) return false;
  const state = collectionCaseFrom(data.collectionCase);
  if (!state) return false;
  for (const [nodeId, node] of Object.entries(document.nodes ?? {})) {
    const cases = stringArray(node.props?.['collection_test_cases']);
    if (cases.includes(state))
      applyOwnerCollectionState(document, nodeId, state);
  }
  return true;
}
