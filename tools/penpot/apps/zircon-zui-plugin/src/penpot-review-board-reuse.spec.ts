import type { Board } from '@penpot/plugin-types';
import type { LayoutReviewCaseMessage } from './model';
import { parseZuiDocument } from './bridge/zui-document';
import fixtureSource from './bridge/roundtrip-fixture.zui?raw';
import { canReuseReviewBoard } from './penpot-review-board-reuse';

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
});
