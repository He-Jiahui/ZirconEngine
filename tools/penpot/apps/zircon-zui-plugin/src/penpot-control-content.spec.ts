import type { Board, Shape } from '@penpot/plugin-types';
import type { ZuiDocument, ZuiNode } from './bridge/zui-document';
import { refreshTreeRowContent } from './penpot-tree-row-controls';
import { assertExportShapeGuard, writeExportShapeGuard } from './penpot-export-guard';
import type { ProjectionText } from './bridge/penpot-projection-model';
import { createControlContent } from './penpot-control-content';

it('does not invent product pixels for an unfilled slot', () => {
  const children: Board['children'] = [];
  expect(
    createControlContent({ children } as Board, {
      component: 'Slot',
      props: { name: 'value' },
    }),
  ).toBe(false);
  expect(children).toHaveLength(0);
});

it('uses the editable source text instead of painting a second NumberField value', () => {
  const children: Board['children'] = [];
  const board = {
    children,
    width: 100,
    height: 32,
    x: 0,
    y: 0,
    appendChild(child: Board['children'][number]) {
      children.push(child);
    },
  } as unknown as Board;
  const text = {
    characters: '42',
    property: 'value_text',
  } as ProjectionText;
  const font = {
    fontFamily: 'Fira Sans',
    variants: [{ fontWeight: '400', fontStyle: 'normal' }],
    applyToText: () => undefined,
  };
  vi.stubGlobal('penpot', {
    fonts: { all: [font] },
    createText: (characters: string) => {
      const metadata = new Map<string, string>();
      return {
        characters,
        width: 1,
        height: 1,
        setSharedPluginData: (_namespace: string, key: string, value: string) =>
          metadata.set(key, value),
        getSharedPluginData: (_namespace: string, key: string) =>
          metadata.get(key) ?? '',
        resize(width: number, height: number) {
          this.width = width;
          this.height = height;
        },
      };
    },
  });
  try {
    expect(
      createControlContent(
        board,
        { component: 'NumberField', props: { value: 42, value_text: '42' } },
        undefined,
        text,
      ),
    ).toBe(false);
    expect(children).toHaveLength(0);
  } finally {
    vi.unstubAllGlobals();
  }
});

function stableTreeRowFixture(options: { svg?: string; svgSize?: number; strokeWidths?: number[]; emptyViewport?: boolean; depth?: number } = {}) {
  let nextId = 0;
  function shape(type: 'board' | 'group' | 'rectangle' | 'path' | 'text') {
    const metadata = new Map<string, string>();
    const writes = { fills: 0, strokes: 0, resize: 0 };
    let fills: Shape['fills'] = [], strokes: Shape['strokes'] = [];
    let x = 0, y = 0, width = options.svgSize ?? 24, height = options.svgSize ?? 24;
    const part = {
      id: 'tree-shape-' + nextId++, type, name: type,
      hidden: false, opacity: 1, borderRadius: 0, rotation: 0, flipX: false, flipY: false,
      parent: undefined as Shape | undefined, children: [] as Shape[],
      get x() { return x; },
      set x(value: number) { const delta = value - x; x = value; for (const child of this.children) child.x += delta; },
      get y() { return y; },
      set y(value: number) { const delta = value - y; y = value; for (const child of this.children) child.y += delta; },
      get width() { return width; }, set width(value: number) { width = value; },
      get height() { return height; }, set height(value: number) { height = value; },
      get content() { return 'M' + x + ' ' + y + ' L' + (x + width) + ' ' + (y + height); },
      characters: 'Camera', writes,
      get fills() { return fills; },
      set fills(value: Shape['fills']) { writes.fills++; fills = value; },
      get strokes() { return strokes; },
      set strokes(value: Shape['strokes']) { writes.strokes++; strokes = value; },
      getSharedPluginData: (_namespace: string, key: string) => metadata.get(key) ?? '',
      setSharedPluginData: (_namespace: string, key: string, value: string) => metadata.set(key, value),
      appendChild(child: Shape) { this.children.push(child); child.parent = this as unknown as Board; },
      resize(nextWidth: number, nextHeight: number) {
        writes.resize++;
        const scaleX = nextWidth / width, scaleY = nextHeight / height;
        for (const child of this.children) {
          const nextX = x + (child.x - x) * scaleX, nextY = y + (child.y - y) * scaleY;
          child.resize(child.width * scaleX, child.height * scaleY);
          child.x = nextX; child.y = nextY;
        }
        width = nextWidth; height = nextHeight;
      },
      remove() { throw new Error('Authoring reflow must retain every native part'); },
    };
    return part;
  }
  const asset = shape('board'), row = shape('board');
  asset.setSharedPluginData('zircon-zui', 'role', 'asset');
  row.setSharedPluginData('zircon-zui', 'role', 'node');
  row.setSharedPluginData('zircon-zui', 'node-id', 'outliner-row');
  asset.appendChild(row as unknown as Board);
  row.width = 300; row.height = 28;
  const allocate = (width: number, height: number) => {
    row.width = width; row.height = height;
    row.setSharedPluginData('zircon-zui', 'native-layout-allocation', JSON.stringify({ parent: asset.id, width, height }));
  };
  allocate(300, 28);
  const node: ZuiNode = { component: 'TreeRow', props: { text: 'Camera', component_variant: 'content', tree_depth: options.depth ?? 0 } };
  if (options.svg) node.penpot_icon_svg = options.svg;
  const document: ZuiDocument = { asset: { kind: 'view', id: 'tree-row', version: 2 },
    nodes: { row: node }, penpot_host_theme_source: 'editor_tokens.zui' };
  const icon = shape('group'), viewport = shape('rectangle');
  icon.fills = 'mixed';
  if (!options.emptyViewport) viewport.strokes = [{ strokeColor: '#b3b3b3', strokeWidth: 2, strokeOpacity: 1 }];
  icon.appendChild(viewport as unknown as Shape);
  const leaves = Array.from({ length: 3 }, (_, index) => {
    const leaf = shape('path');
    leaf.strokes = [{ strokeColor: '#b3b3b3', strokeWidth: options.strokeWidths?.[index] ?? 2, strokeOpacity: 1 }];
    icon.appendChild(leaf as unknown as Shape);
    return leaf;
  });
  const panels: ReturnType<typeof shape>[] = [];
  const createBoard = vi.fn(() => { const panel = shape('board'); panels.push(panel); return panel; });
  const createShapeFromSvg = vi.fn(() => icon);
  vi.stubGlobal('penpot', { createShapeFromSvg, createBoard });
  createControlContent(row as unknown as Board, node, document);
  const text = shape('text');
  text.setSharedPluginData('zircon-zui', 'role', 'text');
  row.appendChild(text as unknown as Shape);
  const refresh = () => refreshTreeRowContent(row as unknown as Board, node, document);
  const parts = () => [row, ...panels, icon, viewport, ...leaves, text];
  return { asset: asset as unknown as Board, row, icon, viewport, leaves, panels, text, node, document, allocate, refresh, parts, shape, createShapeFromSvg };
}

describe('stable TreeRow authoring reflow', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('preserves complete TreeRow identities and makes repeated allocated reflow write no paint or sizes', () => {
    const f = stableTreeRowFixture();
    f.refresh();
    const children = [...f.row.children], descendants = [...f.icon.children];
    const before = f.parts().map((part) => ({ ...part.writes }));
    writeExportShapeGuard(f.asset);
    for (let index = 0; index < 4; index++) {
      f.row.width = 300.00000953674316; f.row.height = 27.94538600140045;
      f.icon.width = 16.000000476836362; f.icon.height = 15.99999570846478;
      f.refresh();
      expect(f.row.children.map(({ id }) => id)).toEqual(children.map(({ id }) => id));
      expect(f.icon.children.map(({ id }) => id)).toEqual(descendants.map(({ id }) => id));
      expect(f.row.children[0]).toBe(f.icon);
      expect(f.icon.children[1]).toBe(f.leaves[0]);
      expect(f.parts().map((part) => part.writes)).toEqual(before);
      expect(() => assertExportShapeGuard(f.asset)).not.toThrow();
    }
    expect(f.icon.fills).toBe('mixed');
    expect(f.createShapeFromSvg).toHaveBeenCalledTimes(1);
  });

  it('updates allocated geometry and uses absolute SVG strokes through a 16 to 20 to 16 round trip', () => {
    const f = stableTreeRowFixture();
    f.refresh();
    const ids = f.parts().map(({ id }) => id);
    f.text.characters = 'Edited source label';
    f.allocate(400, 40);
    f.document.tokens = { 'editor.density.row_height': 32 };
    f.node.props!['icon_color'] = '#ff0000';
    f.refresh();
    expect([f.icon.width, f.icon.height, f.icon.x, f.icon.y]).toEqual([20, 20, 30, 14]);
    expect(f.leaves.map((part) => part.strokes[0]?.strokeWidth)).toEqual([2 * 20 / 24, 2 * 20 / 24, 2 * 20 / 24]);
    expect(f.viewport.strokes[0]?.strokeWidth).toBe(2 * 20 / 24);
    expect(f.leaves.map((part) => part.strokes[0]?.strokeColor)).toEqual(['#ff0000', '#ff0000', '#ff0000']);
    expect(f.parts().map(({ id }) => id)).toEqual(ids);
    expect(f.text.characters).toBe('Edited source label');
    expect(f.icon.fills).toBe('mixed');
    f.allocate(300, 28);
    f.document.tokens = { 'editor.density.row_height': 28 };
    f.refresh();
    expect(f.leaves.map((part) => part.strokes[0]?.strokeWidth)).toEqual([2 * 16 / 24, 2 * 16 / 24, 2 * 16 / 24]);
    expect(f.viewport.strokes[0]?.strokeWidth).toBe(2 * 16 / 24);
    expect(f.parts().map(({ id }) => id)).toEqual(ids);
    const writes = f.parts().map((part) => ({ ...part.writes }));
    f.refresh();
    expect(f.parts().map((part) => part.writes)).toEqual(writes);
    expect(f.createShapeFromSvg).toHaveBeenCalledTimes(1);
  });

  it('keeps incomplete asynchronous SVG children pending without capturing their paint', () => {
    const f = stableTreeRowFixture();
    const last = f.icon.children.pop()!;
    f.refresh();
    expect(JSON.parse(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).generated).toBeUndefined();
    f.icon.appendChild(last);
    f.refresh();
    expect(JSON.parse(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).generated).toBeDefined();
    expect(f.createShapeFromSvg).toHaveBeenCalledTimes(1);
  });

  it('preserves Source text edits and a whole-row translation without touching generated relative geometry', () => {
    const f = stableTreeRowFixture();
    f.refresh();
    writeExportShapeGuard(f.asset);
    const writes = f.parts().map((part) => ({ ...part.writes }));
    const metadata = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
    f.row.x += 125; f.row.y += 37;
    f.text.characters = 'Legitimate Source label';
    f.refresh();
    expect(f.parts().map((part) => part.writes)).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(metadata);
    expect(f.text.characters).toBe('Legitimate Source label');
    expect(() => assertExportShapeGuard(f.asset)).not.toThrow();
  });

  it('uses non24 SVG viewport and per-child original widths through a 16 to 20 to 16 size round trip', () => {
    const svg = '<svg viewBox="0 0 512 512" stroke="none" stroke-width="32"><path stroke="#000000" d="M0 0 L512 512"/><path stroke="#000000" stroke-width="16" d="M0 0 L512 512"/><path stroke="#000000" style="stroke-width:8px" d="M0 0 L512 512"/></svg>';
    const f = stableTreeRowFixture({ svg, svgSize: 512, strokeWidths: [32, 16, 8], emptyViewport: true });
    f.refresh();
    const ids = f.parts().map(({ id }) => id);
    expect(f.leaves.map((part) => part.strokes[0]?.strokeWidth)).toEqual([1, 0.5, 0.25]);
    const original = JSON.parse(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter'));
    f.document.tokens = { 'editor.density.row_height': 32 };
    f.refresh();
    expect(f.leaves.map((part) => part.strokes[0]?.strokeWidth)).toEqual([1.25, 0.625, 0.3125]);
    f.document.tokens = { 'editor.density.row_height': 28 };
    f.refresh();
    expect(f.leaves.map((part) => part.strokes[0]?.strokeWidth)).toEqual([1, 0.5, 0.25]);
    const final = JSON.parse(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter'));
    expect(final.originalWidths).toEqual(original.originalWidths);
    expect(final.originalGeometry).toEqual(original.originalGeometry);
    expect(f.parts().map(({ id }) => id)).toEqual(ids);
    const writes = f.parts().map((part) => ({ ...part.writes }));
    f.refresh();
    expect(f.parts().map((part) => part.writes)).toEqual(writes);
    expect(f.createShapeFromSvg).toHaveBeenCalledTimes(1);
  });

  it('rejects a native allocation with a different parent before any paint write', () => {
    const f = stableTreeRowFixture();
    f.refresh();
    const writes = f.parts().map((part) => ({ ...part.writes }));
    const metadata = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
    f.row.setSharedPluginData('zircon-zui', 'native-layout-allocation', JSON.stringify({ parent: 'foreign-parent', width: 300, height: 28 }));
    f.node.props!['icon_color'] = '#ff0000';
    expect(() => f.refresh()).toThrow(/allocation belongs to a different parent/);
    expect(f.parts().map((part) => part.writes)).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(metadata);
  });

  it('rejects a hand-edited generated Board before recoloring any icon', () => {
    const f = stableTreeRowFixture({ depth: 1 });
    f.refresh();
    expect(f.panels.length).toBeGreaterThan(0);
    writeExportShapeGuard(f.asset);
    const panel = f.panels[0]!;
    panel.width += 1;
    const writes = f.parts().map((part) => ({ ...part.writes }));
    const metadata = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
    f.node.props!['icon_color'] = '#ff0000';
    expect(() => f.refresh()).toThrow(/changed without a ZUI source mapping/);
    expect(f.parts().map((part) => part.writes)).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(metadata);
    expect(() => assertExportShapeGuard(f.asset)).toThrow(/changed geometry/);
  });

  const edits: Array<[string, (f: ReturnType<typeof stableTreeRowFixture>) => void]> = [
    ['direct removal', (f) => { f.row.children.shift(); }],
    ['direct addition', (f) => { f.row.appendChild(f.shape('rectangle') as unknown as Shape); }],
    ['direct reorder', (f) => { f.row.children.reverse(); }],
    ['descendant removal', (f) => { f.icon.children.pop(); }],
    ['descendant addition', (f) => { f.icon.appendChild(f.shape('path') as unknown as Shape); }],
    ['descendant reorder', (f) => { f.icon.children.reverse(); }],
    ['stroke width', (f) => { f.leaves[2]!.strokes = [{ ...f.leaves[2]!.strokes[0]!, strokeWidth: 3 }]; }],
    ['stroke color', (f) => { f.leaves[2]!.strokes = [{ ...f.leaves[2]!.strokes[0]!, strokeColor: '#00ff00' }]; }],
    ['fill', (f) => { f.leaves[2]!.fills = [{ fillColor: '#00ff00', fillOpacity: 1 }]; }],
    ['rectangle x', (f) => { f.viewport.x += 1; }],
    ['rectangle y', (f) => { f.viewport.y += 1; }],
    ['rectangle width', (f) => { f.viewport.width += 1; }],
    ['rectangle height', (f) => { f.viewport.height += 1; }],
    ['SDK icon width', (f) => { f.icon.width += 1; }],
    ['SDK icon height', (f) => { f.icon.height += 1; }],
  ];
  it.each(edits)('rejects user %s before any earlier TreeRow part can be rewritten', (_name, edit) => {
    const f = stableTreeRowFixture();
    f.refresh();
    writeExportShapeGuard(f.asset);
    edit(f);
    const before = f.parts().map((part) => ({ fills: part.fills, strokes: part.strokes, writes: { ...part.writes } }));
    const painterBefore = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
    f.node.props!['icon_color'] = '#ff0000';
    expect(() => f.refresh()).toThrow(/changed without a ZUI source mapping/);
    for (const [index, part] of f.parts().entries()) {
      expect(part.fills).toBe(before[index]!.fills);
      expect(part.strokes).toBe(before[index]!.strokes);
      expect(part.writes).toEqual(before[index]!.writes);
    }
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(painterBefore);
    expect(() => assertExportShapeGuard(f.asset)).toThrow(/changed|added or removed/);
    expect(f.createShapeFromSvg).toHaveBeenCalledTimes(1);
  });
});
