import type { Board, LibraryComponent, Shape } from '@penpot/plugin-types';
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

function stableTreeRowFixture(options: { svg?: string; svgSize?: number; strokeWidths?: number[]; emptyViewport?: boolean; depth?: number; freshProxyChildren?: boolean; normalizeSdkColors?: boolean; fullContent?: boolean } = {}) {
  let nextId = 0;
  function shape(type: 'board' | 'group' | 'rectangle' | 'path' | 'text') {
    const metadata = new Map<string, string>();
    const writes = { fills: 0, strokes: 0, resize: 0 };
    const paintInputs = { fills: [] as Shape['fills'][], strokes: [] as Shape['strokes'][] };
    let fills: Shape['fills'] = [], strokes: Shape['strokes'] = [];
    let x = 0, y = 0, width = options.svgSize ?? 24, height = options.svgSize ?? 24;
    const children: Shape[] = [];
    const part = {
      id: 'tree-shape-' + nextId++, type, name: type,
      hidden: false, opacity: 1, borderRadius: 0, rotation: 0, flipX: false, flipY: false,
      parent: undefined as Shape | undefined,
      get children() {
        return options.freshProxyChildren ? children.map((child) => new Proxy(child, {})) : children;
      },
      get x() { return x; },
      set x(value: number) { const delta = value - x; x = value; for (const child of this.children) child.x += delta; },
      get y() { return y; },
      set y(value: number) { const delta = value - y; y = value; for (const child of this.children) child.y += delta; },
      get width() { return width; }, set width(value: number) { width = value; },
      get height() { return height; }, set height(value: number) { height = value; },
      get content() { return 'M' + x + ' ' + y + ' L' + (x + width) + ' ' + (y + height); },
      characters: 'Camera', writes, paintInputs,
      get fills() { return fills; },
      set fills(value: Shape['fills']) {
        writes.fills++; paintInputs.fills.push(value);
        fills = options.normalizeSdkColors && Array.isArray(value) ? value.map((fill) =>
          typeof fill.fillColor === 'string' ? { ...fill, fillColor: fill.fillColor.toLowerCase() } : fill) : value;
      },
      get strokes() { return strokes; },
      set strokes(value: Shape['strokes']) {
        writes.strokes++; paintInputs.strokes.push(value);
        strokes = options.normalizeSdkColors ? value.map((stroke) =>
          typeof stroke.strokeColor === 'string' ? { ...stroke, strokeColor: stroke.strokeColor.toLowerCase() } : stroke) : value;
      },
      getSharedPluginData: (_namespace: string, key: string) => metadata.get(key) ?? '',
      setSharedPluginData: (_namespace: string, key: string, value: string) => metadata.set(key, value),
      appendChild(child: Shape) { children.push(child); child.parent = this as unknown as Board; },
      resize(nextWidth: number, nextHeight: number) {
        writes.resize++;
        // Like SDK change-dimensions-modifiers: propagate each axis using the
        // actual current root bounds, including any imported fractional size.
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
  const node: ZuiNode = { component: 'TreeRow', props: { text: 'Camera',
    ...(options.fullContent ? {} : { component_variant: 'content' }), tree_depth: options.depth ?? 0 } };
  if (options.svg) node.penpot_icon_svg = options.svg;
  const document: ZuiDocument = { asset: { kind: 'view', id: 'tree-row', version: 2 },
    nodes: { row: node }, penpot_host_theme_source: 'editor_tokens.zui' };
  function svgIcon(drawables = 3, color = '#b3b3b3') {
    const icon = shape('group'), viewport = shape('rectangle');
    icon.fills = 'mixed';
    if (!options.emptyViewport) viewport.strokes = [{ strokeColor: color, strokeWidth: 2, strokeOpacity: 1 }];
    icon.appendChild(viewport as unknown as Shape);
    const leaves = Array.from({ length: drawables }, (_, index) => {
      const leaf = shape('path');
      leaf.strokes = [{ strokeColor: color, strokeWidth: options.strokeWidths?.[index] ?? 2, strokeOpacity: 1 }];
      icon.appendChild(leaf as unknown as Shape);
      return leaf;
    });
    return { icon, viewport, leaves };
  }
  const firstIcon = svgIcon(), { icon, viewport, leaves } = firstIcon;
  const importedIcons: ReturnType<typeof svgIcon>[] = [];
  const panels: ReturnType<typeof shape>[] = [];
  const createBoard = vi.fn(() => { const panel = shape('board'); panels.push(panel); return panel; });
  const createShapeFromSvg = vi.fn((svg: string) => {
    const imported = options.fullContent ? svgIcon(
      [...svg.matchAll(/<(?:path|line|polyline|polygon|circle|ellipse|rect)\b/gi)].length,
      /\bstroke="([^"]*)"/.exec(svg)?.[1] ?? '#b3b3b3',
    ) : firstIcon;
    importedIcons.push(imported);
    return imported.icon;
  });
  vi.stubGlobal('penpot', { createShapeFromSvg, createBoard });
  createControlContent(row as unknown as Board, node, document);
  const text = shape('text');
  text.setSharedPluginData('zircon-zui', 'role', 'text');
  row.appendChild(text as unknown as Shape);
  const refresh = () => refreshTreeRowContent(row as unknown as Board, node, document);
  const parts = () => [row, ...panels,
    ...(options.fullContent ? importedIcons : [firstIcon]).flatMap((part) => [part.icon, part.viewport, ...part.leaves]), text];
  return { asset: asset as unknown as Board, row, icon, viewport, leaves, panels, importedIcons, text, node, document, allocate, refresh, parts, shape, createShapeFromSvg };
}

function linkedTreeRowFixture() {
  const f = stableTreeRowFixture({ fullContent: true });
  f.refresh();
  const inherited = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
  const sourcePath = 'zircon_editor/assets/ui/editor/components/workbench/primitives/data/tree_row.zui';
  const source = 'res://ui/editor/components/workbench/primitives/data/tree_row.zui#WorkbenchTreeRow';
  const originalNodes = new Map<string, Shape>();
  let component: LibraryComponent;
  const clone = (copy: Shape, parent: Shape): Shape => {
    const values = { 'tree-row-authoring-painter': inherited, 'node-id': 'outliner-row' } as Record<string, string>;
    const original = { id: copy.id, type: copy.type, parent, children: [] as Shape[],
      component: () => component, isComponentMainInstance: () => true,
      getSharedPluginData: (_: string, key: string) => values[key] ?? '',
    } as unknown as Shape;
    originalNodes.set(copy.id, original);
    (original as Board).children.push(...('children' in copy ? copy.children.map((child) => clone(child, original)) : []));
    Object.assign(copy, { id: 'copy-' + copy.id, component: () => component,
      isComponentCopyInstance: () => true, componentRefShape: () => new Proxy(original, {}) });
    return original;
  };
  const main = clone(f.row as unknown as Board, { id: f.asset.id } as Shape);
  const record = { version: 1, source, sourcePath, libraryId: 'shared-library', componentId: 'tree-row-component',
    mainInstanceId: main.id, assetBoardId: f.asset.id };
  component = { id: record.componentId, libraryId: record.libraryId, mainInstance: () => main,
    getSharedPluginData: (_: string, key: string) => key === 'canonical-source' ? source :
      key === 'canonical-record' ? JSON.stringify(record) : '',
  } as unknown as LibraryComponent;
  Object.assign(f.asset, { id: 'consumer-' + f.asset.id });
  f.node['penpot_prefab_source'] = sourcePath + '#WorkbenchTreeRow';
  f.node['penpot_review_source_path'] = sourcePath;
  f.node['penpot_review_source_node_id'] = 'outliner-row';
  return { ...f, main, inherited, originalNodes };
}

describe('canonical linked TreeRow painter reflow', () => {
  afterEach(() => vi.unstubAllGlobals());

  it.each([318, 223, 158])('reflows a source-admitted copy at %i without replacing immutable baselines', (width) => {
    const f = linkedTreeRowFixture();
    f.allocate(width, 28);
    expect(f.refresh()).toBe(true);
    const state = expectTreeRowGeneratedGeometry(f);
    const baseline = JSON.parse(f.inherited);
    for (const name of ['originalPaint', 'originalGeometry', 'originalBounds', 'originalWidths'])
      expect(state[name]).toEqual(Object.fromEntries(Object.entries(baseline[name]).map(([id, value]) => ['copy-' + id, value])));
    expect(f.main.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(f.inherited);
    expect(f.row.id).toBe('copy-' + f.main.id);
    expect(f.parts().every((part) => (part as unknown as Shape).componentRefShape()?.id === part.id.slice(5))).toBe(true);
  });

  it('commits only derived IDs when source content and allocation are unchanged', () => {
    const f = linkedTreeRowFixture();
    f.allocate(300, 28);
    const writes = f.parts().map((part) => ({ ...part.writes }));
    expect(f.refresh()).toBe(true);
    expect(f.parts().map((part) => ({ ...part.writes }))).toEqual(writes);
    const raw = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
    expect(JSON.parse(raw).parts.every((id: string) => id.startsWith('copy-'))).toBe(true);
    expect(f.refresh()).toBe(false);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(raw);
  });

  it.each(['geometry', 'paint', 'allocation', 'owner'])('still rejects %s before metadata or painter writes', (edit) => {
    const f = linkedTreeRowFixture();
    if (edit !== 'allocation') f.allocate(223, 28);
    if (edit === 'geometry') f.importedIcons[0]!.icon.x += 1;
    if (edit === 'paint') f.importedIcons[0]!.leaves[0]!.strokes = [{ strokeColor: '#ff0000', strokeWidth: 3 }];
    if (edit === 'owner') f.node['penpot_review_source_path'] = 'other-owner.zui';
    const writes = f.parts().map((part) => ({ ...part.writes }));
    const expected = edit === 'allocation' ? /allocation belongs to a different parent/ :
      edit === 'owner' ? /no canonical source owner/ : /changed without a ZUI source mapping/;
    expect(() => f.refresh()).toThrow(expected);
    expect(f.parts().map((part) => ({ ...part.writes }))).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(f.inherited);
    expect(f.main.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(f.inherited);
  });

  it('never initializes a copy baseline even when its part IDs were already remapped', () => {
    const f = linkedTreeRowFixture();
    f.allocate(223, 28);
    const state = JSON.parse(f.inherited);
    state.parts = state.parts.map((id: string) => 'copy-' + id);
    delete state.generated;
    const raw = JSON.stringify(state);
    f.row.setSharedPluginData('zircon-zui', 'tree-row-authoring-painter', raw);
    const writes = f.parts().map((part) => ({ ...part.writes }));
    expect(() => f.refresh()).toThrow('immutable original painter baseline');
    expect(f.parts().map((part) => ({ ...part.writes }))).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(raw);
  });
});

function expectTreeRowGeneratedGeometry(
  f: ReturnType<typeof stableTreeRowFixture>,
  extra: Array<ReturnType<typeof stableTreeRowFixture>['icon']> = [],
) {
  const state = JSON.parse(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter'));
  const round = (value: number) => Math.round(value * 100) / 100;
  for (const part of [...f.parts(), ...extra]) {
    if (part.id === f.row.id || part.id === f.text.id) continue;
    const predicted = state.generated[part.id]?.geometry;
    expect(predicted).toBeDefined();
    expect(predicted).toEqual({ x: round(part.x - (part.parent?.x ?? 0)),
      y: round(part.y - (part.parent?.y ?? 0)), width: round(part.width), height: round(part.height) });
  }
  return state;
}

function rejectedTreeRowSvgDiagnostic(f: ReturnType<typeof stableTreeRowFixture>, prefix: string) {
  const parts = f.parts(), before = parts.map((part) => ({ ...part.writes }));
  const metadata = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
  let caught: unknown;
  try { f.refresh(); } catch (error) { caught = error; }
  expect(caught).toBeInstanceOf(Error);
  const message = (caught as Error).message;
  expect(message).toContain(prefix + ' SVG diagnostic: ');
  expect(parts.map((part) => part.writes)).toEqual(before);
  expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(metadata);
  return JSON.parse(message.slice(message.indexOf(' SVG diagnostic: ') + ' SVG diagnostic: '.length));
}

describe('stable TreeRow authoring reflow', () => {
  afterEach(() => vi.unstubAllGlobals());

  it.each([592, 852, 1232])('keeps leading glyphs fixed and trailing actions source-aligned at width %i', (width) => {
    const f = stableTreeRowFixture({ fullContent: true });
    const trailing = f.panels.map((panel) => ({ panel,
      rightInset: f.row.x + f.row.width - panel.x - panel.width }));
    expect(trailing).toHaveLength(2);
    expect(f.panels.every((panel) => (panel as unknown as Shape).constraintsHorizontal === 'right')).toBe(true);
    expect(f.importedIcons.slice(0, 2).every(({ icon }) =>
      (icon as unknown as Shape).constraintsHorizontal === 'left')).toBe(true);
    expect(f.importedIcons.slice(2).every(({ icon }) =>
      (icon as unknown as Shape).constraintsHorizontal === 'right')).toBe(true);
    f.allocate(width, 28);
    f.refresh();
    for (const { panel, rightInset } of trailing)
      expect(f.row.x + f.row.width - panel.x - panel.width).toBeCloseTo(rightInset);
    expect(f.row.width).toBe(width);
  });

  it('accepts SDK absent paint references becoming null without repainting or rewriting the captured state', () => {
    const f = stableTreeRowFixture({fullContent: true});
    f.refresh();
    const metadata = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
    for (const part of f.parts()) {
      if (Array.isArray(part.fills)) part.fills = part.fills.map(fill => ({...fill,
        fillColorGradient: null, fillColorRefFile: null, fillColorRefId: null, fillImage: null})) as Shape['fills'];
      part.strokes = part.strokes.map(stroke => ({...stroke, strokeColorRefFile: null,
        strokeColorRefId: null, strokeCapStart: null, strokeCapEnd: null, strokeColorGradient: null, strokeImage: null})) as Shape['strokes'];
    }
    const writes = f.parts().map(part => ({...part.writes}));
    expect(() => f.refresh()).not.toThrow();
    expect(f.parts().map(part => ({...part.writes}))).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(metadata);
  });

  it('accepts SDK nil readback becoming absent after reopening without replacing the saved state', () => {
    const f = stableTreeRowFixture({fullContent: true});
    f.refresh();
    const state = JSON.parse(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter'));
    // Model the archived SDK-produced fixture, not a new production baseline.
    for (const part of Object.values(state.generated) as Array<{paint?: {fills: unknown; strokes: object[]}}>) {
      if (!part.paint) continue;
      if (Array.isArray(part.paint.fills)) for (const fill of part.paint.fills)
        Object.assign(fill, {fillColorGradient: null, fillColorRefFile: null, fillColorRefId: null, fillImage: null});
      for (const stroke of part.paint.strokes)
        Object.assign(stroke, {strokeColorRefFile: null, strokeColorRefId: null, strokeCapStart: null,
          strokeCapEnd: null, strokeColorGradient: null, strokeImage: null});
    }
    const saved = JSON.stringify(state);
    f.row.setSharedPluginData('zircon-zui', 'tree-row-authoring-painter', saved);
    const writes = f.parts().map(part => ({...part.writes}));
    expect(() => f.refresh()).not.toThrow();
    expect(f.parts().map(part => ({...part.writes}))).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(saved);
  });

  const optionalPaintEdits: Array<[string, 'fill' | 'stroke', unknown]> = [
    ['fillColorGradient', 'fill', {type: 'linear', stops: []}],
    ['fillColorRefFile', 'fill', 'a0000000-0000-4000-8000-000000000001'],
    ['fillColorRefId', 'fill', 'a0000000-0000-4000-8000-000000000002'],
    ['fillImage', 'fill', {id: 'a0000000-0000-4000-8000-000000000003', width: 1, height: 1}],
    ['strokeColorRefFile', 'stroke', 'a0000000-0000-4000-8000-000000000001'],
    ['strokeColorRefId', 'stroke', 'a0000000-0000-4000-8000-000000000002'],
    ['strokeCapStart', 'stroke', 'round'], ['strokeCapEnd', 'stroke', 'round'],
    ['strokeColorGradient', 'stroke', {type: 'linear', stops: []}],
    ['strokeImage', 'stroke', {id: 'a0000000-0000-4000-8000-000000000003', width: 1, height: 1}],
  ];
  it.each(optionalPaintEdits)('rejects nonempty optional SDK paint field %s before overwriting native paint', (field, kind, value) => {
    const f = stableTreeRowFixture();
    f.refresh();
    if (kind === 'fill') {
      const part = f.panels[0]!;
      expect(Array.isArray(part.fills) && part.fills.length > 0).toBe(true);
      part.fills = (part.fills as Exclude<Shape['fills'], string>).map(fill => ({...fill, [field]: value}));
    } else {
      const part = f.leaves[0]!;
      expect(part.strokes.length).toBeGreaterThan(0);
      part.strokes = part.strokes.map(stroke => ({...stroke, [field]: value}));
    }
    const writes = f.parts().map(part => ({...part.writes}));
    const metadata = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
    f.node.props!['icon_color'] = '#ff0000';
    expect(() => f.refresh()).toThrow('TreeRow subtree, geometry or paint changed');
    expect(f.parts().map(part => ({...part.writes}))).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(metadata);
  });

  it('retains rejection of unknown paint fields even when their value is null', () => {
    const f = stableTreeRowFixture();
    f.refresh();
    f.leaves[0]!.strokes = f.leaves[0]!.strokes.map(stroke => ({...stroke, unknownPaintField: null}));
    const writes = f.parts().map(part => ({...part.writes}));
    const metadata = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
    expect(() => f.refresh()).toThrow('TreeRow subtree, geometry or paint changed');
    expect(f.parts().map(part => ({...part.writes}))).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(metadata);
  });

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

  it('plans canonical SDK panel and SVG fill and stroke colors before writing or guarding them', () => {
    const f = stableTreeRowFixture({ depth: 1, fullContent: true, normalizeSdkColors: true });
    f.node.props!['icon_color'] = '#C0C0C0';
    f.document['penpot_host_palette'] = { track: '#AbCdEf', surface_hover: '#A1B2C3', border: '#D4E5F6' };
    const filledPath = f.importedIcons[1]!.leaves[0]!;
    filledPath.fills = [{ fillColor: '#ABCDEF', fillOpacity: 0.42 }];
    const source = JSON.stringify(f.document);
    f.refresh();
    const state = expectTreeRowGeneratedGeometry(f);
    const guide = f.panels.find((part) => part.name === 'Penpot · tree-indent-guide')!;
    const action = f.panels.find((part) => part.name === 'Penpot · tree-action-slot')!;
    expect(guide.fills).toEqual([{ fillColor: '#abcdef', fillOpacity: 0.78 }]);
    expect(action.fills).toEqual([{ fillColor: '#a1b2c3', fillOpacity: 1 }]);
    expect(action.strokes[0]?.strokeColor).toBe('#d4e5f6');
    expect(filledPath.fills).toEqual([{ fillColor: '#c0c0c0', fillOpacity: 0.42 }]);
    const guideInput = guide.paintInputs.fills.at(-1), pathInput = filledPath.paintInputs.fills.at(-1);
    expect(Array.isArray(guideInput) ? guideInput[0]?.fillColor : undefined).toBe('#abcdef');
    expect(action.paintInputs.strokes.at(-1)?.[0]?.strokeColor).toBe('#d4e5f6');
    expect(Array.isArray(pathInput) ? pathInput[0]?.fillColor : undefined).toBe('#c0c0c0');
    for (const part of f.parts()) {
      const planned = state.generated[part.id]?.paint;
      if (!planned) continue;
      expect(planned.fills).toEqual(part.fills);
      expect(planned.strokes).toEqual(part.strokes);
    }
    for (const imported of f.importedIcons) {
      expect(imported.icon.fills).toBe('mixed');
      for (const part of [imported.viewport, ...imported.leaves]) {
        expect(part.strokes[0]?.strokeColor).toBe('#c0c0c0');
        expect(part.paintInputs.strokes.at(-1)?.[0]?.strokeColor).toBe('#c0c0c0');
      }
    }
    expect(state.originalPaint[filledPath.id].fills).toEqual([{ fillColor: '#abcdef', fillOpacity: 0.42 }]);
    expect(JSON.parse(state.desired).icons[0].color).toBe('#C0C0C0');
    expect(JSON.stringify(f.document)).toBe(source);
    writeExportShapeGuard(f.asset);
    const writes = f.parts().map((part) => ({ ...part.writes }));
    const metadata = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
    f.refresh();
    expect(f.parts().map((part) => part.writes)).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(metadata);
    expect(() => assertExportShapeGuard(f.asset)).not.toThrow();
  });

  it('normalizes a captured 16 by 15.945591439306043 root once and still rejects a later 0.05 geometry edit', () => {
    const f = stableTreeRowFixture({ freshProxyChildren: true });
    const importedHeight = 15.945591439306043;
    f.leaves[0]!.resize(12, 40 / 3);
    f.leaves[0]!.x = f.icon.x + 2; f.leaves[0]!.y = f.icon.y + 4 / 3;
    f.icon.resize(16, importedHeight);
    const beforeResize = f.icon.writes.resize;
    f.refresh();
    expect(f.icon.writes.resize).toBe(beforeResize + 1);
    expect([f.icon.width, f.icon.height]).toEqual([16, 16]);
    expect(f.viewport.width).toBeCloseTo(16, 10);
    expect(f.viewport.height).toBeCloseTo(16, 10);
    const original = expectTreeRowGeneratedGeometry(f);
    expect(original.originalBounds[f.icon.id]).toEqual({ width: 16, height: importedHeight });
    expect(original.originalGeometry[f.icon.id].height).toBe(importedHeight);
    expect(f.leaves[0]!.height).toBeCloseTo(40 / 3, 10);
    writeExportShapeGuard(f.asset);
    const writes = f.parts().map((part) => ({ ...part.writes }));
    const metadata = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
    f.refresh();
    expect(f.parts().map((part) => part.writes)).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(metadata);
    expect(() => assertExportShapeGuard(f.asset)).not.toThrow();
    f.viewport.height += 0.05;
    expect(() => f.refresh()).toThrow(/TreeRow subtree, geometry or paint changed without a ZUI source mapping/);
    expect(f.parts().map((part) => part.writes)).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(metadata);
    expect(() => assertExportShapeGuard(f.asset)).toThrow(/changed geometry/);
    expect(f.createShapeFromSvg).toHaveBeenCalledTimes(1);
  });

  it('preserves captured nonuniform panel and nested SVG bounds through a 16 to 20 to 16 round trip', () => {
    const f = stableTreeRowFixture({ depth: 1 });
    const nested = f.shape('group'), line = f.leaves[0]!;
    f.icon.children.splice(f.icon.children.findIndex(({ id }) => id === line.id), 1);
    nested.width = 8; nested.height = 8;
    nested.x = f.icon.x + 2; nested.y = f.icon.y + 1;
    f.icon.appendChild(nested as unknown as Shape);
    nested.appendChild(line as unknown as Shape);
    line.resize(0, 4);
    line.x = nested.x + 3; line.y = nested.y + 2;
    f.icon.resize(17, 15.945591439306043);
    const panel = f.panels[0]!;
    panel.resize(0.75, 30.5);
    const imported = { icon: { width: f.icon.width, height: f.icon.height },
      panel: { width: panel.width, height: panel.height } };
    f.refresh();
    const original = expectTreeRowGeneratedGeometry(f, [nested]);
    expect(original.originalBounds[f.icon.id]).toEqual(imported.icon);
    expect(original.originalBounds[panel.id]).toEqual(imported.panel);
    expect(nested.width).toBeCloseTo(8, 10);
    expect(nested.height).toBeCloseTo(8, 10);
    expect(line.width).toBe(0);
    expect(line.height).toBeCloseTo(4, 10);
    const immutable = { geometry: original.originalGeometry, bounds: original.originalBounds, widths: original.originalWidths };
    const ids = [...f.parts(), nested].map(({ id }) => id);
    for (const [extent, width, height] of [[20, 400, 40], [16, 300, 28]] as const) {
      f.allocate(width, height);
      f.document.tokens = { 'editor.density.row_height': extent + 12 };
      f.refresh();
      const state = expectTreeRowGeneratedGeometry(f, [nested]);
      expect([f.icon.width, f.icon.height]).toEqual([extent, extent]);
      expect({ geometry: state.originalGeometry, bounds: state.originalBounds, widths: state.originalWidths }).toEqual(immutable);
      expect([...f.parts(), nested].map(({ id }) => id)).toEqual(ids);
      expect(line.width).toBe(0);
      expect(f.leaves.map((part) => part.strokes[0]?.strokeWidth)).toEqual(Array(3).fill(2 * extent / 24));
      const writes = [...f.parts(), nested].map((part) => ({ ...part.writes }));
      f.refresh();
      expect([...f.parts(), nested].map((part) => part.writes)).toEqual(writes);
    }
    expect(f.createShapeFromSvg).toHaveBeenCalledTimes(1);
  });

  it.each([
    { root: 'icon', axis: 'width', value: 0 },
    { root: 'icon', axis: 'height', value: Number.NaN },
    { root: 'icon', axis: 'height', value: Number.POSITIVE_INFINITY },
    { root: 'panel', axis: 'width', value: 0 },
    { root: 'panel', axis: 'height', value: Number.NaN },
    { root: 'panel', axis: 'height', value: Number.POSITIVE_INFINITY },
  ] as const)('does not capture or paint an invalid $root $axis root bound ($value)', ({ root, axis, value }) => {
    const f = stableTreeRowFixture({ depth: 1 });
    const part = root === 'icon' ? f.icon : f.panels[0]!;
    part[axis] = value;
    const writes = f.parts().map((item) => ({ ...item.writes }));
    const metadata = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
    if (root === 'icon') expect(() => f.refresh()).not.toThrow();
    else expect(() => f.refresh()).toThrow(/invalid original or desired bounds/);
    expect(f.parts().map((item) => item.writes)).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(metadata);
    expect(JSON.parse(metadata).generated).toBeUndefined();
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

  it('matches the original viewport by stable ID through fresh SDK children proxies', () => {
    const f = stableTreeRowFixture({ freshProxyChildren: true });
    f.viewport.hidden = true;
    f.viewport.name = 'base-background';
    const first = f.icon.children[0]!, next = f.icon.children[0]!;
    expect(first).not.toBe(next);
    expect(first.id).toBe(next.id);
    expect(first.id).toBe(f.viewport.id);
    const ids = f.parts().map(({ id }) => id);
    f.refresh();
    expect(JSON.parse(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).generated)
      .toBeDefined();
    writeExportShapeGuard(f.asset);
    const writes = f.parts().map((part) => ({ ...part.writes }));
    f.refresh();
    expect(f.parts().map(({ id }) => id)).toEqual(ids);
    expect(f.parts().map((part) => part.writes)).toEqual(writes);
    expect(() => assertExportShapeGuard(f.asset)).not.toThrow();
    expect(f.createShapeFromSvg).toHaveBeenCalledTimes(1);
  });

  it.each([false, true])('reports the original viewport and owner when initial SVG bounds prevent its exclusion (fresh proxies: %s)', (freshProxyChildren) => {
    const f = stableTreeRowFixture({ freshProxyChildren });
    f.viewport.hidden = true;
    f.viewport.name = 'base-background';
    f.viewport.width += 1;
    const diagnostic = rejectedTreeRowSvgDiagnostic(f, 'TreeRow SVG has added or unsupported children.');
    expect(diagnostic).toMatchObject({
      owner: { id: f.row.id, nodeId: 'outliner-row' },
      source: { width: 24, height: 24, drawables: 3, strokeWidths: [2, 2, 2], viewportStrokeWidths: [2] },
      observed: { stage: 'children', actualDrawables: 4, geometryReady: true },
      firstChildId: f.viewport.id, firstChildIsRectangle: true, viewportMatched: false,
      viewportId: null, truncated: false,
    });
    expect(diagnostic.topology[0]).toMatchObject({ id: f.icon.id,
      relativeBounds: { width: f.icon.width, height: f.icon.height } });
    expect(diagnostic.topology.find((part: { id: string }) => part.id === f.viewport.id))
      .toMatchObject({ parent: f.icon.id, type: 'rectangle', name: 'base-background', hidden: true,
        relativeBounds: { x: 0, y: 0, width: f.viewport.width, height: f.viewport.height } });
  });

  it.each([false, true])('reports added initial SVG leaves while preserving the source drawable rejection (fresh proxies: %s)', (freshProxyChildren) => {
    const f = stableTreeRowFixture({ freshProxyChildren }), extra = f.shape('path');
    extra.name = 'unexpected-path';
    f.icon.appendChild(extra as unknown as Shape);
    const writes = { ...extra.writes };
    const diagnostic = rejectedTreeRowSvgDiagnostic(f, 'TreeRow SVG has added or unsupported children.');
    expect(diagnostic).toMatchObject({ source: { drawables: 3 },
      observed: { stage: 'children', actualDrawables: 4 }, viewportMatched: true, viewportId: f.viewport.id });
    expect(diagnostic.topology.find((part: { id: string }) => part.id === extra.id))
      .toMatchObject({ parent: f.icon.id, type: 'path', name: 'unexpected-path' });
    expect(extra.writes).toEqual(writes);
  });

  it('bounds SVG rejection diagnostics without accepting excess children', () => {
    const f = stableTreeRowFixture();
    const extras = Array.from({ length: 40 }, () => {
      const extra = f.shape('path');
      extra.name = 'x'.repeat(200);
      f.icon.appendChild(extra as unknown as Shape);
      return extra;
    });
    const before = extras.map((part) => ({ ...part.writes }));
    const diagnostic = rejectedTreeRowSvgDiagnostic(f, 'TreeRow SVG has added or unsupported children.');
    expect(diagnostic).toMatchObject({ source: { drawables: 3 },
      observed: { actualDrawables: 43 }, truncated: true, limits: { nodes: 32, depth: 8, text: 160 } });
    expect(diagnostic.topology).toHaveLength(32);
    expect(diagnostic.topology[0].childCount).toBe(44);
    expect(diagnostic.topology[0].childIds).toHaveLength(32);
    expect(diagnostic.topology.at(-1).name).toHaveLength(160);
    expect(extras.map((part) => part.writes)).toEqual(before);
  });

  it('reports source and observed stroke widths before capturing the first SVG baseline', () => {
    const f = stableTreeRowFixture();
    f.leaves[2]!.strokes = [{ ...f.leaves[2]!.strokes[0]!, strokeWidth: 3 }];
    const diagnostic = rejectedTreeRowSvgDiagnostic(f, 'TreeRow original strokes do not match its SVG source.');
    expect(diagnostic).toMatchObject({ source: { drawables: 3, strokeWidths: [2, 2, 2] },
      observed: { stage: 'strokes', actualStrokeWidths: [2, 2, 2, 3], expectedStrokeWidths: [2, 2, 2, 2] },
      viewportMatched: true, viewportId: f.viewport.id });
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
    let caught: unknown;
    try { f.refresh(); } catch (error) { caught = error; }
    expect(caught).toBeInstanceOf(Error);
    const message = (caught as Error).message;
    expect(message).toContain('TreeRow subtree, geometry or paint changed without a ZUI source mapping. State diagnostic: ');
    const diagnostic = JSON.parse(message.split(' State diagnostic: ')[1]!);
    expect(diagnostic).toMatchObject({ row: { id: f.row.id, nodeId: 'outliner-row' }, truncated: false,
      limits: { differences: 32, depth: 8, text: 160, visits: 4096 } });
    expect(diagnostic.differences).toContainEqual({ shape: { id: panel.id, type: 'board', parent: f.row.id },
      category: 'geometry', path: ['geometry', 'width'],
      before: JSON.parse(metadata).generated[panel.id].geometry.width, current: Math.round(panel.width * 100) / 100 });
    expect(f.parts().map((part) => part.writes)).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(metadata);
    expect(() => assertExportShapeGuard(f.asset)).toThrow(/changed geometry/);
  });

  it('bounds generated-state field diagnostics without capturing changed paint', () => {
    const f = stableTreeRowFixture();
    f.refresh();
    f.leaves[0]!.strokes = Array.from({ length: 40 }, () => ({ strokeColor: 'x'.repeat(200), strokeWidth: 3 }));
    const writes = f.parts().map((part) => ({ ...part.writes }));
    const metadata = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
    let caught: unknown;
    try { f.refresh(); } catch (error) { caught = error; }
    expect(caught).toBeInstanceOf(Error);
    const diagnostic = JSON.parse((caught as Error).message.split(' State diagnostic: ')[1]!);
    expect(diagnostic.truncated).toBe(true);
    expect(diagnostic.differences).toHaveLength(32);
    expect(diagnostic.differences).toContainEqual(expect.objectContaining({ category: 'paint',
      path: ['paint', 'strokes', '0', 'strokeColor'], current: 'x'.repeat(160) }));
    expect(f.parts().map((part) => part.writes)).toEqual(writes);
    expect(f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter')).toBe(metadata);
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
    ['stroke opacity', (f) => { f.leaves[2]!.strokes = [{ ...f.leaves[2]!.strokes[0]!, strokeOpacity: 0.25 }]; }],
    ['fill', (f) => { f.leaves[2]!.fills = [{ fillColor: '#00ff00', fillOpacity: 1 }]; }],
    ['fill opacity', (f) => { f.leaves[2]!.fills = [{ ...(f.leaves[2]!.fills as Exclude<Shape['fills'], string>)[0]!, fillOpacity: 0.25 }]; }],
    ['shape opacity', (f) => { f.leaves[2]!.opacity = 0.25; }],
    ['rectangle x', (f) => { f.viewport.x += 1; }],
    ['rectangle y', (f) => { f.viewport.y += 1; }],
    ['rectangle width', (f) => { f.viewport.width += 1; }],
    ['rectangle height', (f) => { f.viewport.height += 1; }],
    ['SDK icon width', (f) => { f.icon.width += 1; }],
    ['SDK icon height', (f) => { f.icon.height += 1; }],
  ];
  it.each(edits)('rejects user %s before any earlier TreeRow part can be rewritten', (name, edit) => {
    const f = stableTreeRowFixture({ normalizeSdkColors: true });
    f.leaves[2]!.fills = [{ fillColor: '#b3b3b3', fillOpacity: 1 }];
    f.refresh();
    writeExportShapeGuard(f.asset);
    edit(f);
    const before = f.parts().map((part) => ({ fills: part.fills, strokes: part.strokes, writes: { ...part.writes } }));
    const painterBefore = f.row.getSharedPluginData('zircon-zui', 'tree-row-authoring-painter');
    f.node.props!['icon_color'] = '#ff0000';
    let caught: unknown;
    try { f.refresh(); } catch (error) { caught = error; }
    expect(caught).toBeInstanceOf(Error);
    const message = (caught as Error).message;
    expect(message).toMatch(/changed without a ZUI source mapping/);
    if (!name.startsWith('direct')) {
      const diagnostic = JSON.parse(message.split(' State diagnostic: ')[1]!);
      expect(diagnostic.row).toMatchObject({ id: f.row.id, nodeId: 'outliner-row' });
      expect(diagnostic.differences.length).toBeGreaterThan(0);
      expect(diagnostic.differences.length).toBeLessThanOrEqual(32);
      const category = name.startsWith('descendant') ? 'topology' :
        name.startsWith('stroke') || name.startsWith('fill') || name === 'shape opacity' ? 'paint' : 'geometry';
      expect(diagnostic.differences.some((part: { category: string }) => part.category === category)).toBe(true);
    }
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
