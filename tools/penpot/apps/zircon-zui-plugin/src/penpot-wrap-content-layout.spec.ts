import { describe, expect, it } from 'vitest';
import type { Board } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import type { ProjectionContainer } from './bridge/penpot-projection-model';
import { applyNativeWrapContentLayout } from './penpot-wrap-content-layout';

function board(id: string, width: number, height: number): Board {
  const data = new Map<string, string>();
  return {
    type: 'board',
    width,
    height,
    hidden: false,
    children: [],
    flex: {
      alignItems: 'center',
      alignContent: 'center',
      justifyContent: 'start',
    },
    layoutChild: {
      absolute: false,
      horizontalSizing: 'fill',
      verticalSizing: 'fill',
      topMargin: 0,
      bottomMargin: 0,
      leftMargin: 0,
      rightMargin: 0,
    },
    getSharedPluginData: (_: string, key: string) =>
      data.get(key) ?? (key === 'node-id' ? id : key === 'role' ? 'node' : ''),
    setSharedPluginData: (_: string, key: string, value: string) => {
      data.set(key, value);
    },
    resize(this: { width: number; height: number }, w: number, h: number) {
      this.width = w;
      this.height = h;
    },
  } as unknown as Board;
}
const flow: ProjectionContainer = {
  kind: 'flex',
  direction: 'row',
  wrap: true,
  gap: 8,
  rowGap: 8,
  columnGap: 8,
  columns: 1,
  rows: 1,
  padding: { left: 0, right: 0, top: 0, bottom: 0 },
  alignItems: 'center',
  justifyContent: 'start',
  clip: false,
};
function document(children: string[], minimum = 0): ZuiDocument {
  return {
    asset: { kind: 'view', id: 'test', version: 2 },
    nodes: {
      root: {
        component: 'WrapBox',
        layout: { container: { kind: 'WrapBox', item_min_width: minimum } },
        children: children.map((node) => ({ node })),
      },
      ...Object.fromEntries(
        children.map((id) => [
          id,
          {
            component: 'VerticalGroup',
            layout: {
              width: { preferred: 260, stretch: 'Stretch' },
              height: { stretch: 'Fixed' },
            },
          },
        ]),
      ),
    },
  };
}
describe('native Wrap content at the Penpot Board boundary', () => {
  it('keeps a tall intrinsic column visible and distributes each row using native stretch weights', () => {
    const root = board('root', 960, 934),
      a = board('a', 476, 240),
      b = board('b', 476, 240);
    Object.assign(root, { children: [a, b] });
    applyNativeWrapContentLayout(
      root,
      document(['a', 'b'], 232),
      flow,
      new Map([
        ['a', { width: 260, height: 934 }],
        ['b', { width: 260, height: 700 }],
      ]),
    );
    expect([a.width, b.width]).toEqual([476, 476]);
    expect([a.height, b.height]).toEqual([934, 700]);
    expect(a.layoutChild?.verticalSizing).toBe('fix');
    expect(root.flex?.alignItems).toBe('start');
    expect(root.flex?.alignContent).toBe('start');
  });
  it('recomputes rows after the viewport narrows without borrowing an allocated child height', () => {
    const root = board('root', 400, 934),
      a = board('a', 476, 240),
      b = board('b', 476, 240);
    Object.assign(root, { children: [a, b] });
    const doc = document(['a', 'b'], 232);
    const desired = new Map([
      ['a', { width: 260, height: 934 }],
      ['b', { width: 260, height: 700 }],
    ]);
    applyNativeWrapContentLayout(root, doc, flow, desired);
    expect([a.width, b.width]).toEqual([400, 400]);
    expect([a.height, b.height]).toEqual([934, 700]);
    root.resize(960, 934);
    applyNativeWrapContentLayout(root, doc, flow, desired);
    expect([a.width, b.width]).toEqual([476, 476]);
  });
  it('keeps Fixed width at intrinsic size and expands only Stretch children', () => {
    const root = board('root', 308, 50),
      a = board('a', 150, 20),
      b = board('b', 150, 20);
    Object.assign(root, { children: [a, b] });
    const doc = document(['a', 'b']);
    doc.nodes!['a'].layout!['width'] = { stretch: 'Fixed' };
    doc.nodes!['b'].layout!['width'] = { stretch: 'Stretch' };
    applyNativeWrapContentLayout(
      root,
      doc,
      flow,
      new Map([
        ['a', { width: 80, height: 40 }],
        ['b', { width: 100, height: 50 }],
      ]),
    );
    expect([a.width, b.width, a.height, b.height]).toEqual([80, 220, 40, 50]);
  });
  it('honors item_min_width, slot margins, and native cross-axis stretch per row', () => {
    const root = board('root', 180, 99),
      a = board('a', 90, 20),
      b = board('b', 90, 20);
    Object.assign(root, { children: [a, b] });
    const doc = document(['a', 'b'], 60);
    doc.nodes!['a'].layout = {
      width: { stretch: 'Fixed' },
      height: { stretch: 'Fixed' },
    };
    doc.nodes!['b'].layout = {
      width: { stretch: 'Fixed' },
      height: { stretch: 'Stretch', max: 45 },
    };
    for (const mount of doc.nodes!['root'].children!)
      mount.slot = {
        layout: {
          padding: { left: 5, right: 5, top: 3, bottom: 7 },
          linear_size: { rule: 'Stretch', value: 99 },
        },
      };
    applyNativeWrapContentLayout(
      root,
      doc,
      flow,
      new Map([
        ['a', { width: 20, height: 40 }],
        ['b', { width: 30, height: 20 }],
      ]),
    );
    expect([a.width, b.width, a.height, b.height]).toEqual([60, 60, 40, 40]);
    expect([
      a.layoutChild?.leftMargin,
      a.layoutChild?.rightMargin,
      a.layoutChild?.topMargin,
      a.layoutChild?.bottomMargin,
    ]).toEqual([5, 5, 3, 7]);
  });
  it('uses native width weights and caps without converting Flow slots into linear sizing', () => {
    const root = board('root', 408, 50),
      a = board('a', 150, 20),
      b = board('b', 150, 20);
    Object.assign(root, { children: [a, b] });
    const doc = document(['a', 'b']);
    doc.nodes!['a'].layout!['width'] = {
      preferred: 100,
      max: 150,
      stretch: 'Stretch',
      weight: 1,
    };
    doc.nodes!['b'].layout!['width'] = {
      preferred: 100,
      stretch: 'Stretch',
      weight: 3,
    };
    doc.nodes!['root'].children![0].slot = {
      layout: { linear_size: { rule: 'Auto' } },
    };
    applyNativeWrapContentLayout(
      root,
      doc,
      flow,
      new Map([
        ['a', { width: 100, height: 40 }],
        ['b', { width: 100, height: 50 }],
      ]),
    );
    expect([a.width, b.width]).toEqual([150, 250]);
  });
});

describe('native Wrap source ownership and supported slot policies', () => {
  it('restores the whole node width table while keeping slot height precedence', () => {
    const root = board('root', 308, 95),
      a = board('a', 80, 95),
      b = board('b', 100, 50);
    Object.assign(root, { children: [a, b] });
    const doc = document(['a', 'b']);
    doc.nodes!['a'].layout!['width'] = { preferred: 80, stretch: 'Fixed' };
    doc.nodes!['b'].layout!['width'] = { preferred: 100, stretch: 'Stretch' };
    doc.nodes!['root'].children![0].slot = {
      layout: {
        width: { preferred: 200, stretch: 'Stretch', priority: 1 },
        height: { stretch: 'Fixed', min: 95, preferred: 95 },
      },
    };
    applyNativeWrapContentLayout(
      root,
      doc,
      flow,
      new Map([
        ['a', { width: 80, height: 95 }],
        ['b', { width: 100, height: 50 }],
      ]),
    );
    expect([a.width, b.width, a.height]).toEqual([80, 220, 95]);
  });
  it('preserves supported Fixed cross-axis slot alignment', () => {
    const root = board('root', 208, 100),
      a = board('a', 100, 40),
      b = board('b', 100, 100);
    Object.assign(root, { children: [a, b] });
    const doc = document(['a', 'b']);
    for (const id of ['a', 'b'])
      doc.nodes![id].layout!['width'] = { preferred: 100, stretch: 'Fixed' };
    doc.nodes!['root'].children![0].slot = {
      layout: { alignment: { vertical: 'Center' } },
    };
    applyNativeWrapContentLayout(
      root,
      doc,
      flow,
      new Map([
        ['a', { width: 100, height: 40 }],
        ['b', { width: 100, height: 100 }],
      ]),
    );
    expect(a.layoutChild?.alignSelf).toBe('center');
    doc.nodes!['root'].children![0].slot = {
      layout: { alignment: { vertical: 'End' } },
    };
    applyNativeWrapContentLayout(
      root,
      doc,
      flow,
      new Map([
        ['a', { width: 100, height: 40 }],
        ['b', { width: 100, height: 100 }],
      ]),
    );
    expect(a.layoutChild?.alignSelf).toBe('end');
  });
  it('keeps the current Penpot flow order after a visual child reorder', () => {
    const root = board('root', 250, 80),
      a = board('a', 100, 20),
      b = board('b', 100, 20),
      c = board('c', 100, 20);
    Object.assign(root, { children: [b, c, a] });
    const doc = document(['a', 'b', 'c']);
    for (const id of ['a', 'b', 'c'])
      doc.nodes![id].layout!['width'] = { preferred: 100, stretch: 'Stretch' };
    applyNativeWrapContentLayout(
      root,
      doc,
      flow,
      new Map(['a', 'b', 'c'].map((id) => [id, { width: 100, height: 20 }])),
    );
    expect([b.width, c.width, a.width]).toEqual([121, 121, 250]);
  });
  it('rejects unmapped child placement instead of rendering a misleading native Wrap projection', () => {
    const root = board('root', 300, 80),
      a = board('a', 100, 20);
    Object.assign(root, { children: [a] });
    const doc = document(['a']);
    doc.nodes!['a'].layout!['position'] = { x: 1, y: 0 };
    expect(() =>
      applyNativeWrapContentLayout(
        root,
        doc,
        flow,
        new Map([['a', { width: 100, height: 20 }]]),
      ),
    ).toThrow(/Unsupported native Wrap/);
  });
});
