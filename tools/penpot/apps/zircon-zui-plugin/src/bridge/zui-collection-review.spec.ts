import type { LayoutReviewCaseMessage } from '../model';
import {
  applyCollectionReviewData,
  authoredCollectionReviewCases,
} from './zui-collection-review';
import { normalizeZuiDocument, type ZuiDocument } from './zui-document';
import { reviewDocumentForCase } from './zui-review-case';

const source: ZuiDocument = {
  asset: { kind: 'view', id: 'res://collection.zui', version: 2 },
  root: { node: 'root' },
  nodes: {
    root: {
      component: 'ScrollableBox',
      props: {
        collection_test_cases: [
          'empty',
          'one',
          'normal',
          'overflow',
          'narrow',
          'long-locale',
        ],
        collection_items: ['selected|Alpha|owner', 'normal|Beta|member'],
        visible_limit: 1,
        empty_text: 'No workspaces',
      },
      children: [{ node: 'heading' }, { node: 'alpha' }, { node: 'beta' }],
    },
    heading: { component: 'Label', props: { text: 'Workspaces' } },
    alpha: { component: 'Button', props: { text: 'Alpha' } },
    beta: { component: 'Button', props: { text: 'Beta' } },
  },
};

const reviewCase: LayoutReviewCaseMessage = {
  id: 'collection-empty-640x520-dpi1',
  sourcePath: 'fixture.zui',
  host: 'fixture',
  viewport: { width: 640, height: 520 },
  dpi: 1,
  locale: 'en-US',
  state: 'default',
  data: { collectionCase: 'empty' },
};

describe('source-owned collection review adapter', () => {
  it('discovers the authored collection matrix in source order', () => {
    expect(authoredCollectionReviewCases(source)).toEqual([
      'empty',
      'one',
      'normal',
      'overflow',
      'narrow',
      'long-locale',
    ]);
  });

  it.each([
    ['empty', 0],
    ['one', 1],
    ['normal', 2],
    ['overflow', 3],
  ] as const)('materializes %s without mutating source', (state, count) => {
    const before = normalizeZuiDocument(source);
    const clone = structuredClone(source);
    expect(applyCollectionReviewData(clone, { collectionCase: state })).toBe(
      true,
    );
    expect(
      ['alpha', 'beta'].filter(
        (nodeId) =>
          clone.nodes?.[nodeId]?.props?.['visibility'] !== 'collapsed',
      ),
    ).toHaveLength(Math.min(count, 2));
    expect(normalizeZuiDocument(source)).toEqual(before);
  });

  it('keeps long-locale text explicit in the review clone', () => {
    const clone = structuredClone(source);
    applyCollectionReviewData(clone, { collectionCase: 'long-locale' });
    expect(clone.nodes?.alpha?.props?.['text']).toContain(
      'deliberately long localized collection label',
    );
    expect(clone.nodes?.root?.props?.['collection_review_case']).toBe(
      'long-locale',
    );
  });

  it('routes collection data through the shared review document entry point', () => {
    const clone = reviewDocumentForCase(source, reviewCase);
    expect(clone.nodes?.alpha?.props?.['visibility']).toBe('collapsed');
    expect(clone.nodes?.beta?.props?.['visibility']).toBe('collapsed');
    expect(clone.nodes?.root?.props?.['collection_items']).toEqual([]);
    expect(normalizeZuiDocument(source)).not.toEqual(
      normalizeZuiDocument(clone),
    );
  });
});
