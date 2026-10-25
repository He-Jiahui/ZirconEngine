import type { Board } from '@penpot/plugin-types';
import { applyWeightedFlexLayout } from './penpot-weighted-flex';
import type { ZuiDocument, ZuiTable } from './bridge/zui-document';
import type { ProjectionContainer } from './bridge/penpot-projection-model';
import { assertExportShapeGuard, writeExportShapeGuard } from './penpot-export-guard';
import { ZUI_METADATA_NAMESPACE } from './metadata';

function board(nodeId: string, width: number, height: number): Board {
  const data = new Map<string, string>();
  const value = {
    id: nodeId,
    name: nodeId,
    type: 'board',
    x: 0,
    y: 0,
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

describe('authored native slot cross alignment', () => {
  function sdkAlignment(child: Board, parent: Board, lane: ProjectionContainer): string[] {
    const writes: string[] = [];
    let current: string | undefined;
    Object.assign(child, { parent });
    Object.defineProperty(child.layoutChild!, 'alignSelf', {
      get: () => current,
      set(value: string) {
        current = value;
        writes.push(value);
        // Model the official flex child's cross placement, including margins.
        const row = lane.direction === 'row';
        const start = row ? lane.padding.top : lane.padding.left;
        const end = row ? lane.padding.bottom : lane.padding.right;
        const marginStart = row ? child.layoutChild!.topMargin ?? 0 : child.layoutChild!.leftMargin ?? 0;
        const marginEnd = row ? child.layoutChild!.bottomMargin ?? 0 : child.layoutChild!.rightMargin ?? 0;
        const room = (row ? parent.height : parent.width) - start - end - marginStart - marginEnd;
        const remaining = Math.max(0, room - (row ? child.height : child.width));
        const offset = value === 'center' ? remaining / 2 : value === 'end' ? remaining : 0;
        if (row) child.y = parent.y + start + marginStart + offset;
        else child.x = parent.x + start + marginStart + offset;
      },
      configurable: true,
    });
    return writes;
  }

  it('centers the fixed command scroll and Settings once and restores Start when its slot is removed', () => {
    const parent = board('toolbar', 1280, 48);
    const scroll = board('command_scroll', 1166, 32), settings = board('settings', 106, 32);
    Object.assign(parent, { id: 'toolbar', x: 0, y: 61, children: [scroll, settings] });
    const lane = container('row');
    lane.columnGap = 8;
    const scrollWrites = sdkAlignment(scroll, parent, lane), settingsWrites = sdkAlignment(settings, parent, lane);
    const fixed32 = { min: 32, preferred: 32, max: 32, stretch: 'Fixed' };
    const doc: ZuiDocument = {
      asset: { kind: 'view', id: 'authored-toolbar', version: 2 },
      nodes: {
        toolbar: { component: 'HorizontalBox', children: [
          { node: 'command_scroll', slot: { layout: { alignment: { vertical: 'Center' } } } },
          { node: 'settings', slot: { layout: { alignment: { vertical: 'Center' } } } },
        ] },
        command_scroll: { component: 'ScrollableBox', layout: {
          width: { min: 0, stretch: 'Stretch' }, height: { ...fixed32 },
        } },
        settings: { component: 'Button', control_id: 'UnrealToolbarSettings',
          layout: { width: { min: 106, preferred: 106, max: 106, stretch: 'Fixed' }, height: { ...fixed32 } } },
      },
    };
    const authored = structuredClone(doc);
    const identity = { sourcePath: 'res://ui/editor/unreal/components/toolbar.zui', sourceNodeId: 'settings',
      instancePath: JSON.stringify([{ sourcePath: 'res://ui/editor/unreal/level_editor_window.zui', sourceNodeId: 'toolbar' }]),
      controlId: 'UnrealToolbarSettings' };
    for (const [key, value] of Object.entries(identity)) settings.setSharedPluginData(ZUI_METADATA_NAMESPACE, key, value);
    const desired = new Map([['command_scroll', { width: 1166, height: 32 }], ['settings', { width: 106, height: 32 }]]);
    applyWeightedFlexLayout(parent, doc, lane, desired);
    expect([scroll.height, settings.height, scroll.y, settings.y]).toEqual([32, 32, 69, 69]);
    expect([scroll.width, settings.width]).toEqual([1166, 106]);
    expect([scroll.layoutChild!.alignSelf, settings.layoutChild!.alignSelf]).toEqual(['center', 'center']);
    writeExportShapeGuard(parent); // Actual import captures guards after layout settlement.
    applyWeightedFlexLayout(parent, doc, lane, desired);
    expect(() => assertExportShapeGuard(parent)).not.toThrow();
    expect([scrollWrites, settingsWrites]).toEqual([['center'], ['center']]);
    expect(doc).toEqual(authored);
    for (const [key, value] of Object.entries(identity)) expect(settings.getSharedPluginData(ZUI_METADATA_NAMESPACE, key)).toBe(value);
    settings.layoutChild!.alignSelf = 'end';
    expect(() => assertExportShapeGuard(parent)).toThrow(/changed alignSelf/);
    settings.layoutChild!.alignSelf = 'center';
    doc.nodes!.toolbar.children![1].slot = undefined;
    applyWeightedFlexLayout(parent, doc, lane, desired);
    expect(settings.layoutChild!.alignSelf).toBe('start');
    expect(settings.y).toBe(61);
    const writesAfterRestore = settingsWrites.length;
    applyWeightedFlexLayout(parent, doc, lane, desired);
    expect(settingsWrites).toHaveLength(writesAfterRestore);
    expect(scrollWrites).toEqual(['center']);
  });

  it.each(['row', 'column'] as const)('uses the %s cross axis and preserves asymmetric padding and fixed extents', (direction) => {
    const parent = board('parent', direction === 'row' ? 100 : 80, direction === 'row' ? 80 : 100);
    const child = board('label', direction === 'row' ? 40 : 20, direction === 'row' ? 20 : 40);
    Object.assign(parent, { x: 0, y: 0, children: [child] });
    const lane = container(direction);
    if (direction === 'row') { lane.padding.top = 6; lane.padding.bottom = 10; }
    else { lane.padding.left = 6; lane.padding.right = 10; }
    Object.assign(child.layoutChild!, direction === 'row' ? { topMargin: 3, bottomMargin: 5 } : { leftMargin: 3, rightMargin: 5 });
    const writes = sdkAlignment(child, parent, lane);
    const main = direction === 'row' ? 'width' : 'height', cross = direction === 'row' ? 'height' : 'width';
    const crossAxis = direction === 'row' ? 'vertical' : 'horizontal';
    const doc: ZuiDocument = {
      asset: { kind: 'view', id: 'slot-cross-padding', version: 2 },
      nodes: {
        parent: { component: direction === 'row' ? 'HorizontalGroup' : 'VerticalGroup', children: [
          { node: 'label', slot: { layout: { alignment: { [crossAxis]: 'Center' },
            padding: direction === 'row' ? { top: 3, bottom: 5 } : { left: 3, right: 5 } } } },
        ] },
        label: { component: 'Label', layout: {
          [main]: { min: 40, preferred: 40, max: 40, stretch: 'Fixed' },
          [cross]: { min: 20, preferred: 20, max: 20, stretch: 'Fixed' },
        } },
      },
    };
    const desired = new Map([['label', { width: child.width, height: child.height }]]);
    for (const [source, sdk, expected] of [
      ['Center', 'center', 27], ['End', 'end', 45], ['Fill', 'stretch', 9], ['Start', 'start', 9],
    ] as const) {
      const alignment = (doc.nodes!.parent.children![0].slot!.layout as ZuiTable)['alignment'] as ZuiTable;
      alignment[crossAxis] = source;
      const authored = structuredClone(doc);
      applyWeightedFlexLayout(parent, doc, lane, desired);
      expect(child.layoutChild!.alignSelf).toBe(sdk);
      expect(direction === 'row' ? child.y : child.x).toBe(expected);
      expect([child[main], child[cross]]).toEqual([40, 20]);
      expect(doc).toEqual(authored);
    }
    const count = writes.length;
    applyWeightedFlexLayout(parent, doc, lane, desired);
    expect(writes).toHaveLength(count);
  });

  const invalidAlignments: Array<string | ZuiTable> = ['Center', { vertical: 'center' }, { vertical: 1 }, { horizontal: 'Unknown' }];
  it.each(invalidAlignments)(
    'rejects malformed or unsupported native slot alignment %j', (alignment) => {
      const parent = board('parent', 100, 48), child = board('child', 32, 32);
      Object.assign(parent, { children: [child] });
      const doc: ZuiDocument = { asset: { kind: 'view', id: 'invalid-alignment', version: 2 }, nodes: {
        parent: { component: 'HorizontalBox', children: [{ node: 'child', slot: { layout: { alignment } } }] },
        child: { component: 'Button' },
      } };
      expect(() => applyWeightedFlexLayout(parent, doc, container('row'))).toThrow(/native slot alignment/);
      expect(child.layoutChild!.alignSelf).toBeUndefined();
      expect([child.width, child.height]).toEqual([32, 32]);
    },
  );

  it('leaves unauthored defaults and non-native flex alignment untouched', () => {
    const parent = board('parent', 100, 48), child = board('child', 32, 32);
    Object.assign(parent, { x: 0, y: 0, children: [child] });
    const lane = container('row'), writes = sdkAlignment(child, parent, lane);
    const doc: ZuiDocument = { asset: { kind: 'view', id: 'alignment-default', version: 2 }, nodes: {
      parent: { component: 'HorizontalBox', children: [{ node: 'child' }] }, child: { component: 'Button' },
    } };
    applyWeightedFlexLayout(parent, doc, lane);
    expect(writes).toEqual([]);
    doc.nodes!.parent.component = 'Stack';
    child.layoutChild!.alignSelf = 'end';
    applyWeightedFlexLayout(parent, doc, lane);
    expect(writes).toEqual(['end']);
  });
});

describe.each(['row', 'column'] as const)('native Fixed main axis in a %s', (direction) => {
  const main = direction === 'row' ? 'width' : 'height';
  const cases: Array<{
    name: string;
    constraint: ZuiTable;
    room: number;
    expected: number;
    padding?: number;
  }> = [
    { name: 'the 28 + 60 DIP snapping group', constraint: {}, room: 88, expected: 60 },
    { name: 'zero preferred with measured content', constraint: { preferred: 0 }, room: 88, expected: 60 },
    { name: 'preferred below measured content', constraint: { preferred: 32 }, room: 88, expected: 60 },
    { name: 'preferred above measured content', constraint: { preferred: 80 }, room: 200, expected: 80 },
    { name: 'a maximum below measured content', constraint: { max: 48 }, room: 88, expected: 48 },
    { name: 'a minimum above measured content', constraint: { min: 64 }, room: 100, expected: 64 },
    { name: 'outer slot padding once', constraint: {}, room: 100, expected: 60, padding: 12 },
  ];

  it.each(cases)('preserves $name', ({ constraint, room, expected, padding }) => {
    const parent = board('parent', direction === 'row' ? room : 28, direction === 'row' ? 28 : room);
    const toggle = board('toggle', 28, 28);
    const dropdown = board('dropdown', direction === 'row' ? 0.01 : 28, direction === 'row' ? 28 : 0.01);
    Object.assign(parent, { children: [toggle, dropdown] });
    const doc: ZuiDocument = {
      asset: { kind: 'view', id: 'snapping', version: 2 },
      nodes: {
        parent: {
          component: direction === 'row' ? 'HorizontalBox' : 'VerticalBox',
          children: [
            { node: 'toggle' },
            {
              node: 'dropdown',
              ...(padding ? { slot: { layout: { padding: direction === 'row'
                ? { left: 5, right: 7 } : { top: 5, bottom: 7 } } } } : {}),
            },
          ],
        },
        toggle: { component: 'Button', layout: { [main]: { preferred: 28, stretch: 'Fixed' } } },
        dropdown: { component: 'Dropdown', layout: { [main]: { ...constraint, stretch: 'Fixed' } } },
      },
    };
    const desired = new Map([
      ['toggle', { width: 28, height: 28 }],
      ['dropdown', { width: direction === 'row' ? 60 : 28, height: direction === 'row' ? 28 : 60 }],
    ]);
    applyWeightedFlexLayout(parent, doc, container(direction), desired);
    expect(toggle[main]).toBe(28);
    expect(dropdown[main]).toBe(expected);
    // Projection must not change the authored constraint to encode measurement.
    expect(doc.nodes!.dropdown.layout![main]).toEqual({ ...constraint, stretch: 'Fixed' });
  });

  it('keeps measured Fixed content beside an explicit Stretch slot', () => {
    const parent = board('parent', 200, 200);
    const fixed = board('fixed', 0.01, 0.01);
    const fill = board('fill', 1, 1);
    Object.assign(parent, { children: [fixed, fill] });
    const doc: ZuiDocument = {
      asset: { kind: 'view', id: 'mixed', version: 2 },
      nodes: {
        parent: {
          component: direction === 'row' ? 'HorizontalBox' : 'VerticalBox',
          children: [{ node: 'fixed' }, { node: 'fill', slot: { layout: { linear_size: { rule: 'Stretch' } } } }],
        },
        fixed: { component: 'Dropdown', layout: { [main]: { preferred: 0, stretch: 'Fixed' } } },
        fill: { component: 'Space' },
      },
    };
    const desired = new Map([['fixed', { width: 60, height: 60 }], ['fill', { width: 1, height: 1 }]]);
    applyWeightedFlexLayout(parent, doc, container(direction), desired);
    expect(fixed[main]).toBe(60);
    expect(fill[main]).toBe(140);
  });
});

describe.each(['row', 'column'] as const)('native linear deficit at the Board boundary in a %s', (direction) => {
  const main = direction === 'row' ? 'width' : 'height';
  const cases: Array<{
    name: string;
    margin?: number;
    firstMin?: number;
    firstMax?: number;
    firstDesired?: number;
    firstPriority?: number;
    positioned?: boolean;
    expected: [number, number];
  }> = [
    {name: 'implicit Fixed slots shrink by their 28 / 60 content bases', expected: [22.272727, 47.727273]},
    {name: 'the 12 DIP outer margin remains fixed during shrink', margin: 12, expected: [18.454545, 39.545455]},
    {name: 'a content minimum freezes before the remaining deficit is redistributed', margin: 12, firstMin: 24, expected: [24, 34]},
    {name: 'the resolved preferred value respects the node maximum before becoming a basis', margin: 12, firstMax: 20, firstDesired: 20, expected: [14.5, 43.5]},
    {name: 'nonzero priority keeps the existing weighted projection policy', firstPriority: 1, expected: [28, 42]},
    {name: 'nondefault placement keeps the existing weighted projection policy', positioned: true, expected: [19, 51]},
  ];

  it.each(cases)('$name', ({margin = 0, firstMin = 0, firstMax = -1, firstDesired = 28, firstPriority = 0, positioned, expected}) => {
    const parent = board('parent', direction === 'row' ? 70 : 28, direction === 'row' ? 28 : 70);
    const first = board('first', direction === 'row' ? firstDesired : 28, direction === 'row' ? 28 : firstDesired);
    const second = board('second', direction === 'row' ? 60 : 28, direction === 'row' ? 28 : 60);
    Object.assign(parent, {children: [first, second]});
    if (margin) {
      Object.assign(second.layoutChild!, direction === 'row'
        ? {leftMargin: 5, rightMargin: 7}
        : {topMargin: 5, bottomMargin: 7});
    }
    const doc: ZuiDocument = {
      asset: {kind: 'view', id: 'fixed-deficit', version: 2},
      nodes: {
        parent: {
          component: direction === 'row' ? 'HorizontalBox' : 'VerticalBox',
          children: [
            {node: 'first'},
            {node: 'second', ...(margin ? {slot: {layout: {padding: direction === 'row'
              ? {left: 5, right: 7} : {top: 5, bottom: 7}}}} : {})},
          ],
        },
        first: {
          component: 'Button',
          layout: {
            [main]: {min: firstMin, max: firstMax, preferred: 28, priority: firstPriority, weight: 1, stretch: 'Fixed'},
            ...(positioned ? {position: direction === 'row' ? {x: 1, y: 0} : {x: 0, y: 1}} : {}),
          },
        },
        second: {component: 'Dropdown', layout: {[main]: {min: 0, max: -1, preferred: 0, priority: 0, weight: 1, stretch: 'Fixed'}}},
      },
    };
    const authored = JSON.stringify(doc);
    const desired = new Map([
      ['first', {width: direction === 'row' ? firstDesired : 28, height: direction === 'row' ? 28 : firstDesired}],
      ['second', {width: direction === 'row' ? 60 : 28, height: direction === 'row' ? 28 : 60}],
    ]);
    applyWeightedFlexLayout(parent, doc, container(direction), desired);
    expect(first[main]).toBeCloseTo(expected[0], 6);
    expect(second[main]).toBeCloseTo(expected[1], 6);
    expect(first[main] + second[main] + margin).toBeCloseTo(70, 6);
    if (margin) {
      expect(direction === 'row' ? second.layoutChild!.leftMargin : second.layoutChild!.topMargin).toBe(5);
      expect(direction === 'row' ? second.layoutChild!.rightMargin : second.layoutChild!.bottomMargin).toBe(7);
    }
    expect(JSON.stringify(doc)).toBe(authored);
  });
});
