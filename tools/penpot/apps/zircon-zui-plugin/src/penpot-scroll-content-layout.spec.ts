import { describe, expect, it } from 'vitest';
import type { Board } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import type { ProjectionContainer } from './bridge/penpot-projection-model';
import { applyNativeScrollContentLayout } from './penpot-scroll-content-layout';

function board(
  nodeId: string,
  width: number,
  height: number,
  scroll = false,
): Board {
  const data = new Map<string, string>();
  const raw = {
    type: 'board',
    width,
    height,
    hidden: false,
    children: [] as Board[],
    layoutChild: {
      absolute: false,
      horizontalSizing: 'fill',
      verticalSizing: 'fill',
      topMargin: 8,
      bottomMargin: 8,
      leftMargin: 4,
      rightMargin: 4,
    },
    getSharedPluginData: (_: string, key: string) =>
      data.get(key) ??
      (key === 'node-id'
        ? nodeId
        : key === 'role'
          ? 'node'
          : key === 'container-kind' && scroll
            ? 'scroll'
            : ''),
    setSharedPluginData: (_: string, key: string, value: string) => {
      data.set(key, value);
    },
    resize(w: number, h: number) {
      this.width = w;
      this.height = h;
    },
  };
  return raw as unknown as Board;
}
const lane: ProjectionContainer = {
  kind: 'flex',
  direction: 'column',
  wrap: false,
  gap: 4,
  rowGap: 4,
  columnGap: 4,
  columns: 1,
  rows: 1,
  padding: { left: 10, right: 10, top: 0, bottom: 0 },
  alignItems: 'start',
  justifyContent: 'start',
  clip: true,
};
describe('native scroll allocation at the Penpot Board boundary', () => {
  it('keeps cards at desired height when the viewport is shorter than the content', () => {
    const parent = board('parent', 480, 120, true),
      first = board('a', 480, 60),
      second = board('b', 480, 60);
    Object.assign(parent, { children: [first, second] });
    const doc: ZuiDocument = {
      asset: { kind: 'view', id: 'test', version: 2 },
      nodes: {
        parent: {
          component: 'ScrollableBox',
          children: [{ node: 'a' }, { node: 'b' }],
        },
        a: {
          component: 'VerticalBox',
          layout: {
            width: { min: 100, max: 300, stretch: 'Stretch' },
            height: { stretch: 'Stretch' },
          },
        },
        b: {
          component: 'VerticalBox',
          layout: {
            width: { stretch: 'Fixed' },
            height: { stretch: 'Stretch' },
          },
        },
      },
    };
    const desired = new Map([
      ['a', { width: 120, height: 190 }],
      ['b', { width: 80, height: 50 }],
    ]);
    applyNativeScrollContentLayout(parent, doc, lane, desired);
    expect([first.height, second.height]).toEqual([190, 50]);
    expect([first.width, second.width]).toEqual([300, 80]);
    expect(first.layoutChild?.verticalSizing).toBe('fix');
    expect(first.layoutChild?.topMargin).toBe(0);
    Object.assign(parent, { height: 600, width: 220 });
    applyNativeScrollContentLayout(parent, doc, lane, desired);
    expect([first.height, second.height]).toEqual([190, 50]);
    expect(first.width).toBe(200);
  });
  it('uses the same desired main-axis rule for horizontal scroll content', () => {
    const parent = board('parent', 60, 100, true),
      child = board('a', 60, 100);
    Object.assign(parent, { children: [child] });
    const doc: ZuiDocument = {
      asset: { kind: 'view', id: 'test', version: 2 },
      nodes: {
        parent: { component: 'ScrollableBox', children: [{ node: 'a' }] },
        a: {
          component: 'Label',
          layout: {
            width: { stretch: 'Stretch' },
            height: { stretch: 'Stretch' },
          },
        },
      },
    };
    applyNativeScrollContentLayout(
      parent,
      doc,
      { ...lane, direction: 'row' },
      new Map([['a', { width: 320, height: 20 }]]),
    );
    expect([child.width, child.height]).toEqual([320, 100]);
  });
});
