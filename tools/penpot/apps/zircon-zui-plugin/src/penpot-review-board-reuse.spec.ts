import type { Board } from '@penpot/plugin-types';
import type { LayoutReviewCaseMessage } from './model';
import { parseZuiDocument } from './bridge/zui-document';
import fixtureSource from './bridge/roundtrip-fixture.zui?raw';
import { canReuseReviewBoard } from './penpot-review-board-reuse';
import { ZUI_METADATA_NAMESPACE } from './metadata';

const reviewCase: LayoutReviewCaseMessage = {
  id: 'default-640x520-dpi1',
  sourcePath: 'test.zui',
  host: 'fixture',
  viewport: { width: 640, height: 520 },
  dpi: 1,
  locale: 'en-US',
  state: 'default',
  data: {},
};

function board(
  previousCase?: LayoutReviewCaseMessage,
): Pick<Board, 'getSharedPluginData'> {
  return {
    getSharedPluginData: (_namespace, key) =>
      key === 'review-case' && previousCase ? JSON.stringify(previousCase) : '',
  };
}

describe('Penpot review board reuse', () => {
  it('rebuilds responsive topology on the first review and after viewport changes', () => {
    const { document } = parseZuiDocument(fixtureSource);
    document.nodes!['root'].component = 'Stack';
    document.nodes!['root'].props = {
      direction: { xs: 'column', md: 'row' },
    };

    expect(canReuseReviewBoard(board(), document, reviewCase)).toBe(false);
    expect(
      canReuseReviewBoard(
        board({ ...reviewCase, viewport: { width: 1280, height: 800 } }),
        document,
        reviewCase,
      ),
    ).toBe(false);
    expect(canReuseReviewBoard(board(reviewCase), document, reviewCase)).toBe(
      true,
    );
  });

  it('retains fast viewport clones for documents without responsive layout values', () => {
    const { document } = parseZuiDocument(fixtureSource);
    expect(canReuseReviewBoard(board(), document, reviewCase)).toBe(true);
  });

  it('rebuilds guarded boards at the same viewport for default and scroll reviews', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const rowId = '89651681-4b61-808b-8008-c8c6563e2ac6';
    const iconId = '89651681-4b61-808b-8008-c8c67dcf9b56';
    const viewportId = '89651681-4b61-808b-8008-c8c67dd0fbc6';
    const pathIds = [
      '89651681-4b61-808b-8008-c8c67dd12a1c',
      '89651681-4b61-808b-8008-c8c67dd14f56',
      '89651681-4b61-808b-8008-c8c67dd18122',
    ];
    // An isolated TreeRow guard retains captured SDK IDs and SVG child order.
    const guard = JSON.stringify(
      {
        version: 5,
        shapes: {
          [rowId]: { type: 'board', parent: null, children: [iconId] },
          [iconId]: {
            type: 'group',
            parent: rowId,
            children: [viewportId, ...pathIds],
          },
          [viewportId]: { type: 'rectangle', parent: iconId, children: [] },
          ...Object.fromEntries(
            pathIds.map((id) => [
              id,
              { type: 'path', parent: iconId, children: [] },
            ]),
          ),
        },
      },
      null,
      2,
    );
    const metadata = new Map([
      ['review-case', JSON.stringify(reviewCase)],
      ['export-shapes-guard', guard],
    ]);
    const before = [...metadata];
    const guardedBoard: Pick<Board, 'getSharedPluginData'> = {
      getSharedPluginData: (namespace, key) =>
        namespace === ZUI_METADATA_NAMESPACE ? (metadata.get(key) ?? '') : '',
    };

    for (const state of ['default', 'scroll-before', 'scroll-after'] as const) {
      const currentCase = { ...reviewCase, state };
      expect(canReuseReviewBoard(board(reviewCase), document, currentCase)).toBe(
        true,
      );
      expect(canReuseReviewBoard(guardedBoard, document, currentCase)).toBe(
        false,
      );
      expect([...metadata]).toEqual(before);
    }
  });
});
