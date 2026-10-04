import type { Board } from '@penpot/plugin-types';
import { applyWeightedFlexLayout } from './penpot-weighted-flex';
import type { ZuiDocument } from './bridge/zui-document';
import type { ProjectionContainer } from './bridge/penpot-projection-model';

function board(nodeId: string, width: number, height: number): Board {
  const data = new Map<string, string>();
  const value = {
    type: 'board',
    width,
    height,
    hidden: false,
    children: [] as Board[],
    layoutChild: {
      absolute: false,
      horizontalSizing: 'fill',
      verticalSizing: 'fill',
    },
    getSharedPluginData: (_namespace: string, key: string) =>
      data.get(key) ??
      (key === 'node-id' ? nodeId : key === 'role' ? 'node' : ''),
    setSharedPluginData: (_: string, key: string, value: string) => {
      data.set(key, value);
    },
    resize(nextWidth: number, nextHeight: number) {
      this.width = nextWidth;
      this.height = nextHeight;
    },
  };
  return value as unknown as Board;
}
function container(direction: 'row' | 'column'): ProjectionContainer {
  return {
    kind: 'flex',
    direction,
    wrap: false,
    gap: 0,
    rowGap: 0,
    columnGap: 0,
    columns: 1,
    rows: 1,
    padding: { left: 0, right: 0, top: 0, bottom: 0 },
    alignItems: 'stretch',
    justifyContent: 'start',
    clip: false,
  };
}
function document(axis: 'width' | 'height'): ZuiDocument {
  return {
    asset: { kind: 'view', id: 'test', version: 2 },
    nodes: {
      parent: {
        component: axis === 'height' ? 'VerticalBox' : 'HorizontalBox',
        children: [
          {
            node: 'toolbar',
            slot: {
              layout: { linear_size: { rule: 'Auto', shrink_value: 0 } },
            },
          },
          {
            node: 'main',
            slot: { layout: { linear_size: { rule: 'Stretch', value: 4 } } },
          },
          {
            node: 'drawer',
            slot: {
              layout: { linear_size: { rule: 'StretchContent', value: 1 } },
            },
          },
        ],
      },
      toolbar: {
        component: 'HorizontalBox',
        layout: { [axis]: { stretch: 'Stretch' } },
      },
      main: {
        component: 'Overlay',
        layout: { [axis]: { stretch: 'Stretch' } },
      },
      drawer: {
        component: 'VerticalBox',
        layout: { [axis]: { stretch: 'Stretch' } },
      },
    },
  };
}
describe('native linear slot allocation at the Penpot Board boundary', () => {
  it('allocates column Auto / Stretch / StretchContent from content then weights', () => {
    const parent = board('parent', 480, 640),
      children = [
        board('toolbar', 120, 66),
        board('main', 120, 20),
        board('drawer', 120, 190),
      ];
    Object.assign(parent, { children });
    const desired = new Map([
      ['toolbar', { width: 120, height: 66 }],
      ['main', { width: 120, height: 20 }],
      ['drawer', { width: 120, height: 190 }],
    ]);
    applyWeightedFlexLayout(
      parent,
      document('height'),
      container('column'),
      desired,
    );
    expect(children.map((child) => child.height)).toEqual([66, 307.2, 266.8]);
    expect(children.map((child) => child.layoutChild?.verticalSizing)).toEqual([
      'fix',
      'fix',
      'fix',
    ]);
    expect(
      children.map((child) => child.layoutChild?.horizontalSizing),
    ).toEqual(['fix', 'fix', 'fix']);
    Object.assign(parent, { height: 840 });
    applyWeightedFlexLayout(
      parent,
      document('height'),
      container('column'),
      desired,
    );
    expect(children.map((child) => child.height)).toEqual([66, 467.2, 306.8]);
  });
  it('applies the same slot rules along rows even when authored weights are equal', () => {
    const parent = board('parent', 640, 80),
      children = [
        board('toolbar', 66, 20),
        board('main', 20, 20),
        board('drawer', 190, 20),
      ];
    Object.assign(parent, { children });
    const desired = new Map([
      ['toolbar', { width: 66, height: 20 }],
      ['main', { width: 20, height: 20 }],
      ['drawer', { width: 190, height: 20 }],
    ]);
    applyWeightedFlexLayout(
      parent,
      document('width'),
      container('row'),
      desired,
    );
    expect(children.map((child) => child.width)).toEqual([66, 307.2, 266.8]);
  });
  it('subtracts outer slot padding only after allocating it and ignores hidden/absolute children', () => {
    const parent = board('parent', 500, 80),
      content = board('content', 190, 20),
      fill = board('fill', 20, 20),
      hidden = board('hidden', 100, 20),
      overlay = board('overlay', 100, 20);
    Object.assign(hidden, { hidden: true });
    Object.assign(overlay.layoutChild!, { absolute: true });
    Object.assign(parent, { children: [content, fill, hidden, overlay] });
    const doc: ZuiDocument = {
      asset: { kind: 'view', id: 'test', version: 2 },
      nodes: {
        parent: {
          component: 'HorizontalBox',
          children: [
            {
              node: 'content',
              slot: {
                layout: {
                  linear_size: { rule: 'Auto' },
                  padding: { left: 5, right: 7 },
                },
              },
            },
            {
              node: 'fill',
              slot: { layout: { linear_size: { rule: 'Stretch' } } },
            },
            { node: 'hidden' },
            { node: 'overlay' },
          ],
        },
        content: {
          component: 'Label',
          layout: {
            width: { min: 190, preferred: 190, max: 190, stretch: 'Fixed' },
          },
        },
        fill: { component: 'Space', layout: { width: { stretch: 'Stretch' } } },
        hidden: { component: 'Label' },
        overlay: { component: 'Overlay' },
      },
    };
    const lane = container('row');
    lane.columnGap = 10;
    lane.padding.left = 10;
    lane.padding.right = 10;
    const desired = new Map([
      ['content', { width: 190, height: 20 }],
      ['fill', { width: 20, height: 20 }],
    ]);
    applyWeightedFlexLayout(parent, doc, lane, desired);
    expect(content.width).toBe(190);
    expect(fill.width).toBe(268);
    expect(hidden.width).toBe(100);
    expect(overlay.width).toBe(100);
  });
});

describe('native linear Fixed cross axis', () => {
  it.each(['row', 'column'] as const)(
    'uses measured content and fixes allocated cross size for %s',
    (direction) => {
      const parent = board('parent', 500, 500),
        child = board('fixed', 500, 500);
      Object.assign(parent, { children: [child] });
      const main = direction === 'row' ? 'width' : 'height',
        cross = direction === 'row' ? 'height' : 'width';
      const doc: ZuiDocument = {
        asset: { kind: 'view', id: 'test', version: 2 },
        nodes: {
          parent: {
            component: direction === 'row' ? 'HorizontalBox' : 'VerticalBox',
            children: [
              {
                node: 'fixed',
                slot: { layout: { linear_size: { rule: 'Stretch' } } },
              },
            ],
          },
          fixed: {
            component: 'Label',
            layout: {
              [main]: { stretch: 'Stretch' },
              [cross]: { min: 16, preferred: 20, max: 32, stretch: 'Fixed' },
            },
          },
        },
      };
      const desired = new Map([['fixed', { width: 20, height: 20 }]]);
      applyWeightedFlexLayout(parent, doc, container(direction), desired);
      expect(child[cross]).toBe(20);
      expect(
        direction === 'row'
          ? child.layoutChild?.verticalSizing
          : child.layoutChild?.horizontalSizing,
      ).toBe('fix');
      Object.assign(parent, { width: 700, height: 700 });
      applyWeightedFlexLayout(parent, doc, container(direction), desired);
      expect(child[cross]).toBe(20);
    },
  );
});
