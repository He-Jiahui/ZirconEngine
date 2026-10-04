import { describe, expect, it } from 'vitest';

import {
  applyReviewScrollPresentation,
  captureReviewScrollLayoutSnapshot,
  captureReviewScrollLayoutPositions,
  restoreReviewScrollPresentation,
  reviewScrollPosition,
  reviewScrollLayoutReady,
  reviewScrollLayoutReadyForCapture,
  type ReviewScrollShape,
} from './penpot-review-scroll';
import {
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_LAYOUT_MODE,
  ZUI_METADATA_ROLE,
  ZUI_METADATA_SCROLL_AXIS,
  ZUI_ROLE_NODE,
} from './metadata';

function board(
  x: number,
  y: number,
  width: number,
  height: number,
): ReviewScrollShape {
  const metadata = new Map<string, string>();
  return {
    type: 'board',
    x,
    y,
    width,
    height,
    children: [],
    layoutChild: { absolute: false },
    getSharedPluginData(namespace, key) {
      return metadata.get(`${namespace}/${key}`) ?? '';
    },
    setSharedPluginData(namespace, key, value) {
      metadata.set(`${namespace}/${key}`, value);
    },
  };
}

function metadata(shape: ReviewScrollShape, key: string, value: string): void {
  shape.setSharedPluginData(ZUI_METADATA_NAMESPACE, key, value);
}

function resetPositionWhenDetached(shape: ReviewScrollShape): void {
  let absolute = shape.layoutChild?.absolute ?? false;
  Object.defineProperty(shape.layoutChild!, 'absolute', {
    get: () => absolute,
    set: (value: boolean) => {
      absolute = value;
      if (value) {
        shape.x = 0;
        shape.y = 0;
      }
    },
  });
}

function resetPositionWhileAutoLayout(shape: ReviewScrollShape): void {
  let x = shape.x;
  let y = shape.y;
  Object.defineProperties(shape, {
    x: {
      get: () => x,
      set: (value: number) => {
        x = shape.layoutChild?.absolute ? value : 0;
      },
    },
    y: {
      get: () => y,
      set: (value: number) => {
        y = shape.layoutChild?.absolute ? value : 0;
      },
    },
  });
}

describe('Penpot review scroll presentation', () => {
  it('keeps scroll offset independent of the interaction state', () => {
    expect(reviewScrollPosition({ state: 'open', scrollPosition: 'end' })).toBe(
      'end',
    );
    expect(reviewScrollPosition({ state: 'scroll-before' })).toBe('start');
    expect(reviewScrollPosition({ state: 'scroll-after' })).toBe('end');
    expect(reviewScrollPosition({ state: 'open' })).toBeUndefined();
  });

  it('moves only semantic scroll content to the end of an explicit vertical viewport', () => {
    const root = board(0, 0, 240, 160);
    const scroll = board(0, 20, 240, 100);
    const first = board(0, 20, 240, 60);
    const last = board(0, 90, 240, 80);
    const textLeaf = {
      type: 'text',
      x: 8,
      y: 28,
      width: 80,
      height: 16,
      getSharedPluginData: () => '',
      setSharedPluginData: () => undefined,
    } as unknown as ReviewScrollShape;
    root.children.push(scroll);
    scroll.children.push(first, textLeaf, last);
    metadata(scroll, ZUI_METADATA_SCROLL_AXIS, 'vertical');
    metadata(first, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);
    metadata(last, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);

    expect(applyReviewScrollPresentation(root, 'end')).toBe(1);
    expect(first.y).toBe(-30);
    expect(last.y).toBe(40);
    expect(first.layoutChild?.absolute).toBe(true);
    expect(last.layoutChild?.absolute).toBe(true);
    expect(
      scroll.getSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        'review-scroll-position',
      ),
    ).toBe('end');
    expect(
      scroll.getSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        'review-scroll-offset-y',
      ),
    ).toBe('50');
  });

  it('records the start position without moving content', () => {
    const root = board(0, 0, 240, 160);
    const scroll = board(0, 20, 240, 100);
    const child = board(0, 20, 240, 180);
    root.children.push(scroll);
    scroll.children.push(child);
    metadata(scroll, ZUI_METADATA_SCROLL_AXIS, 'vertical');
    metadata(child, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);

    expect(applyReviewScrollPresentation(root, 'start')).toBe(1);
    expect(child.y).toBe(20);
    expect(child.layoutChild?.absolute).toBe(false);
  });

  it('requires realized flex positions before a scroll-end presentation', () => {
    const root = board(0, 0, 480, 520);
    const scroll = board(0, 0, 480, 520);
    const first = board(0, 0, 480, 24);
    const last = board(0, 0, 480, 132);
    root.children!.push(scroll);
    scroll.children!.push(first, last);
    metadata(scroll, ZUI_METADATA_SCROLL_AXIS, 'vertical');
    metadata(scroll, ZUI_METADATA_LAYOUT_MODE, 'flex');
    metadata(first, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);
    metadata(last, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);

    expect(reviewScrollLayoutReady(root)).toBe(false);
    last.y = 684;
    expect(reviewScrollLayoutReady(root)).toBe(true);
  });

  it('allows an unpositioned auto-layout frame when its content fits', () => {
    const root = board(0, 0, 480, 520);
    const scroll = board(0, 0, 480, 520);
    const first = board(0, 0, 480, 24);
    const last = board(0, 0, 480, 132);
    root.children!.push(scroll);
    scroll.children!.push(first, last);
    metadata(scroll, ZUI_METADATA_SCROLL_AXIS, 'vertical');
    metadata(scroll, ZUI_METADATA_LAYOUT_MODE, 'flex');
    metadata(first, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);
    metadata(last, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);

    expect(reviewScrollLayoutReady(root)).toBe(false);
    expect(reviewScrollLayoutReadyForCapture(root)).toBe(true);
  });

  it('keeps an unpositioned overflowing frame blocked for capture', () => {
    const root = board(0, 0, 480, 520);
    const scroll = board(0, 0, 480, 520);
    const first = board(0, 0, 480, 400);
    const last = board(0, 0, 480, 400);
    root.children!.push(scroll);
    scroll.children!.push(first, last);
    metadata(scroll, ZUI_METADATA_SCROLL_AXIS, 'vertical');
    metadata(scroll, ZUI_METADATA_LAYOUT_MODE, 'flex');
    metadata(first, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);
    metadata(last, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);

    expect(reviewScrollLayoutReadyForCapture(root)).toBe(false);
  });

  it('uses a captured stable layout when Penpot exposes transient origin coordinates', () => {
    const root = board(0, 0, 480, 520);
    const scroll = board(0, 0, 480, 520);
    const first = board(0, 0, 480, 24);
    const last = board(0, 684, 480, 132);
    root.children!.push(scroll);
    scroll.children!.push(first, last);
    metadata(scroll, ZUI_METADATA_SCROLL_AXIS, 'vertical');
    metadata(scroll, ZUI_METADATA_LAYOUT_MODE, 'flex');
    metadata(first, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);
    metadata(last, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);

    expect(captureReviewScrollLayoutPositions(root)).toBe(1);
    // A later Penpot mutation can clear temporary plugin data. The scroll
    // reviewer must still retain the settled layout.
    first.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'review-scroll-captured-y',
      '',
    );
    last.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'review-scroll-captured-y',
      '',
    );
    last.y = 0;
    expect(applyReviewScrollPresentation(root, 'end')).toBe(1);
    expect(first.y).toBe(-296);
    expect(last.y).toBe(388);
  });

  it('calculates scroll targets before Flex children accept manual coordinates', () => {
    const root = board(0, 0, 480, 520);
    const scroll = board(0, 0, 480, 520);
    const first = board(0, 0, 480, 24);
    const last = board(0, 684, 480, 132);
    root.children!.push(scroll);
    scroll.children!.push(first, last);
    metadata(scroll, ZUI_METADATA_SCROLL_AXIS, 'vertical');
    metadata(scroll, ZUI_METADATA_LAYOUT_MODE, 'flex');
    metadata(first, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);
    metadata(last, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);

    expect(captureReviewScrollLayoutPositions(root)).toBe(1);
    resetPositionWhileAutoLayout(first);
    resetPositionWhileAutoLayout(last);
    last.y = 0;

    expect(applyReviewScrollPresentation(root, 'end')).toBe(1);
    expect(first.y).toBe(-296);
    expect(last.y).toBe(388);
  });

  it('passes settled scroll geometry directly across the layout transaction', () => {
    const root = board(0, 0, 480, 520);
    const scroll = board(0, 0, 480, 520);
    const first = board(0, 0, 480, 24);
    const last = board(0, 684, 480, 132);
    root.children!.push(scroll);
    scroll.children!.push(first, last);
    metadata(scroll, ZUI_METADATA_SCROLL_AXIS, 'vertical');
    metadata(scroll, ZUI_METADATA_LAYOUT_MODE, 'flex');
    metadata(first, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);
    metadata(last, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);

    const snapshot = captureReviewScrollLayoutSnapshot(root);
    resetPositionWhileAutoLayout(first);
    resetPositionWhileAutoLayout(last);
    last.y = 0;

    expect(applyReviewScrollPresentation(root, 'end', snapshot)).toBe(1);
    expect(first.y).toBe(-296);
    expect(last.y).toBe(388);
  });

  it('resolves captured coordinates by semantic node id after a projected child is recreated', () => {
    const root = board(0, 0, 480, 520);
    const scroll = board(0, 0, 480, 520);
    const first = board(0, 0, 480, 24);
    const last = board(0, 684, 480, 132);
    root.children!.push(scroll);
    scroll.children!.push(first, last);
    metadata(scroll, ZUI_METADATA_SCROLL_AXIS, 'vertical');
    metadata(scroll, ZUI_METADATA_LAYOUT_MODE, 'flex');
    metadata(first, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);
    metadata(last, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);

    expect(captureReviewScrollLayoutPositions(root)).toBe(1);
    // The initial projection can receive stable node ids only after its first
    // layout transaction has completed.
    metadata(first, ZUI_METADATA_NODE_ID, 'first');
    metadata(last, ZUI_METADATA_NODE_ID, 'last');
    // Model a new capture session: object and module-local caches disappear,
    // but the scroll container's temporary snapshot remains with the board.
    expect(captureReviewScrollLayoutPositions(board(0, 0, 1, 1))).toBe(0);

    const recreatedFirst = board(0, 0, 480, 24);
    const recreatedLast = board(0, 0, 480, 132);
    metadata(recreatedFirst, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);
    metadata(recreatedLast, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);
    metadata(recreatedFirst, ZUI_METADATA_NODE_ID, 'first');
    metadata(recreatedLast, ZUI_METADATA_NODE_ID, 'last');
    scroll.children = [recreatedFirst, recreatedLast];

    expect(applyReviewScrollPresentation(root, 'end')).toBe(1);
    expect(recreatedFirst.y).toBe(-296);
    expect(recreatedLast.y).toBe(388);
  });

  it('uses measured positions after detaching auto-layout scroll content', () => {
    const root = board(0, 0, 480, 520);
    const scroll = board(0, 0, 480, 520);
    const first = board(0, 0, 480, 24);
    const last = board(0, 684, 480, 132);
    root.children!.push(scroll);
    scroll.children!.push(first, last);
    metadata(scroll, ZUI_METADATA_SCROLL_AXIS, 'vertical');
    metadata(first, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);
    metadata(last, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);
    resetPositionWhenDetached(first);
    resetPositionWhenDetached(last);

    applyReviewScrollPresentation(root, 'end');

    expect(first.y).toBe(-296);
    expect(last.y).toBe(388);
    expect(
      last.getSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        'review-scroll-target-y',
      ),
    ).toBe('388');
  });

  it('restores design-only scroll offsets before a reversible source export', () => {
    const root = board(0, 0, 240, 160);
    const scroll = board(0, 20, 240, 100);
    const first = board(0, 20, 240, 60);
    const last = board(0, 90, 240, 80);
    scroll.clipContent = false;
    last.layoutChild!.absolute = true;
    root.children!.push(scroll);
    scroll.children!.push(first, last);
    metadata(scroll, ZUI_METADATA_SCROLL_AXIS, 'vertical');
    metadata(first, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);
    metadata(last, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);

    applyReviewScrollPresentation(root, 'end');
    expect(first.y).toBe(-30);
    expect(last.y).toBe(40);
    expect(first.layoutChild?.absolute).toBe(true);
    expect(scroll.clipContent).toBe(true);

    expect(restoreReviewScrollPresentation(root)).toBe(1);
    expect(first.y).toBe(20);
    expect(last.y).toBe(90);
    expect(first.layoutChild?.absolute).toBe(false);
    expect(last.layoutChild?.absolute).toBe(true);
    expect(scroll.clipContent).toBe(false);
    expect(
      scroll.getSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        'review-scroll-position',
      ),
    ).toBe('');
  });
});
