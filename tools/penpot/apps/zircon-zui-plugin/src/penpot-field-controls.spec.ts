import type { Board, Shape } from '@penpot/plugin-types';
import type { FieldProjection } from './bridge/zui-field-projection';
import { refreshFieldDecoration } from './penpot-field-controls';
import { assertExportShapeGuard, writeExportShapeGuard } from './penpot-export-guard';
import { ZUI_METADATA_NAMESPACE } from './metadata';

function makeShape(initial: Partial<Shape> = {}): Shape & {
  resizeCalls: Array<[number, number]>;
  paintWrites: { fills: number; strokes: number };
  children: Shape[];
  appendChild(child: Shape): void;
} {
  const data = new Map<string, string>();
  const shape = {
    id: `shape-${Math.random()}`,
    name: '',
    type: 'board',
    content: 'M0 0 L1 1',
    x: 0,
    y: 0,
    width: 0,
    height: 0,
    fills: [],
    strokes: [],
    children: [] as Shape[],
    resizeCalls: [] as Array<[number, number]>,
    setSharedPluginData: (_namespace: string, key: string, value: string) =>
      data.set(key, value),
    getSharedPluginData: (_namespace: string, key: string) =>
      data.get(key) ?? '',
    appendChild(child: Shape) {
      this.children.push(child);
      Object.assign(child, { parent: this });
    },
    remove: () => undefined,
    resize(width: number, height: number) {
      this.resizeCalls.push([width, height]);
      this.width = width;
      this.height = height;
    },
    ...initial,
  };
  const paintWrites = { fills: 0, strokes: 0 };
  let fills = shape.fills;
  let strokes = shape.strokes;
  Object.defineProperties(shape, {
    fills: { enumerable: true, get: () => fills, set: (value: typeof fills) => { paintWrites.fills++; fills = value; } },
    strokes: { enumerable: true, get: () => strokes, set: (value: typeof strokes) => { paintWrites.strokes++; strokes = value; } },
  });
  return Object.assign(shape, { paintWrites }) as unknown as Shape & {
    resizeCalls: Array<[number, number]>;
    paintWrites: { fills: number; strokes: number };
    children: Shape[];
    appendChild(child: Shape): void;
  };
}

function makeField(overrides: Partial<FieldProjection> = {}): FieldProjection {
  return {
    paint: {
      fillColor: '#111111',
      fillOpacity: 1,
      strokeColor: '#333333',
      strokeOpacity: 1,
      strokeWidth: 1,
      borderRadius: 4,
      opacity: 1,
    },
    search: true,
    clear: false,
    stepper: false,
    pad: 8,
    gap: 4,
    lineHeight: 20,
    iconSize: 16,
    maxHeight: 32,
    stepperWidth: 0,
    stepperGlyphWidth: 0,
    borderWidth: 1,
    divider: '#333333',
    iconColor: '#ffffff',
    stepperColor: '#ffffff',
    icons: {
      search:
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M0 0" stroke="#fff" stroke-width="1.5"/></svg>',
    },
    ...overrides,
  };
}

function searchFixture() {
  const sourceSvg = '<svg viewBox="0 0 24 24"><circle fill="none" stroke="#cbd2dc" stroke-width="1.8"/><path fill="none" stroke="#cbd2dc" stroke-width="1.9"/></svg>';
  const board = makeShape({ x: 100, y: 50, width: 240, height: 32 }) as unknown as Board & ReturnType<typeof makeShape>;
  const viewport = makeShape({ type: 'rectangle', width: 24, height: 24 });
  const circle = makeShape({ type: 'ellipse', strokes: [{ strokeColor: '#cbd2dc', strokeOpacity: 1, strokeWidth: 1.8 }] });
  const path = makeShape({ type: 'path', strokes: [{ strokeColor: '#cbd2dc', strokeOpacity: 1, strokeWidth: 1.9 }] });
  const icon = makeShape({ type: 'group', width: 24, height: 24 });
  for (const child of [viewport, circle, path]) icon.appendChild(child);
  const createShapeFromSvg = vi.fn(() => icon);
  vi.stubGlobal('penpot', { createBoard: () => makeShape(), createShapeFromSvg });
  const field = makeField({ iconColor: '#c0c0c0', icons: { search: sourceSvg } });
  return { board, viewport, circle, path, icon, field, sourceSvg, createShapeFromSvg };
}

describe('field icon placement', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('uses intrinsic SVG dimensions while a newly imported Penpot shape is unsettled', () => {
    const board = makeShape({
      x: 100,
      y: 50,
      width: 240,
      height: 32,
    }) as unknown as Board & ReturnType<typeof makeShape>;
    const importedIcon = makeShape({
      // Penpot creates this shape asynchronously. Its proxy can report the
      // initial empty bounds before the SVG importer has populated geometry.
      width: 0,
      height: 0,
      strokes: [{ strokeColor: '#ffffff', strokeWidth: 1.5 }],
    });
    vi.stubGlobal('penpot', {
      createBoard: () => makeShape(),
      createShapeFromSvg: () => importedIcon,
    });

    const field = makeField();

    refreshFieldDecoration(board, field);

    expect(importedIcon.resizeCalls.at(-1)).toEqual([16, 16]);
    expect([importedIcon.x, importedIcon.y]).toEqual([108, 58]);
    expect(importedIcon.strokes[0]?.strokeWidth).toBeCloseTo(1);
    expect(
      [importedIcon.x, importedIcon.y, importedIcon.width, importedIcon.height].every(
        Number.isFinite,
      ),
    ).toBe(true);
  });
});

describe('field icon paint refresh', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('does not write paint during repeated refreshes with imprecise native bounds', () => {
    const { board, icon, circle, path, viewport, field, sourceSvg, createShapeFromSvg } = searchFixture();
    const sourceBefore = JSON.stringify(field);
    refreshFieldDecoration(board, field);
    const paints = [icon, viewport, circle, path].map((shape) => ({
      fills: shape.fills, strokes: shape.strokes, writes: { ...shape.paintWrites },
    }));
    for (let i = 0; i < 3; i++) {
      icon.width = 16.000000476836362;
      icon.height = 15.99999570846478;
      refreshFieldDecoration(board, field);
    }
    expect([circle.strokes[0]?.strokeWidth, path.strokes[0]?.strokeWidth]).toEqual([1.2, 1.2666666666666666]);
    for (const [index, shape] of [icon, viewport, circle, path].entries()) {
      expect(shape.fills).toBe(paints[index]!.fills);
      expect(shape.strokes).toBe(paints[index]!.strokes);
      expect(shape.paintWrites).toEqual(paints[index]!.writes);
    }
    expect(icon.children).toEqual([viewport, circle, path]);
    expect(icon.children[0]).toBe(viewport);
    expect(icon.children[1]).toBe(circle);
    expect(icon.children[2]).toBe(path);
    expect(createShapeFromSvg).toHaveBeenCalledTimes(1);
    expect(field.icons.search).toBe(sourceSvg);
    expect(JSON.stringify(field)).toBe(sourceBefore);
  });

  it('uses immutable per-child widths when the desired size changes and returns', () => {
    const { board, circle, path, field } = searchFixture();
    refreshFieldDecoration(board, field);
    refreshFieldDecoration(board, { ...field, iconSize: 20 });
    expect([circle.strokes[0]?.strokeWidth, path.strokes[0]?.strokeWidth]).toEqual([1.5, 1.5833333333333333]);
    refreshFieldDecoration(board, field);
    expect([circle.strokes[0]?.strokeWidth, path.strokes[0]?.strokeWidth]).toEqual([1.2, 1.2666666666666666]);
  });

  it('keeps empty and partial SVG imports pending until all original strokes arrive', () => {
    const { board, icon, viewport, circle, path, field, createShapeFromSvg } = searchFixture();
    icon.children = [];
    icon.width = 0;
    icon.height = 0;
    refreshFieldDecoration(board, field);
    expect(icon.paintWrites).toEqual({ fills: 0, strokes: 0 });
    icon.appendChild(viewport);
    icon.appendChild(circle);
    refreshFieldDecoration(board, field);
    expect(circle.strokes[0]?.strokeWidth).toBe(1.8);
    expect(circle.paintWrites).toEqual({ fills: 0, strokes: 0 });
    icon.appendChild(path);
    refreshFieldDecoration(board, field);
    expect([circle.strokes[0]?.strokeWidth, path.strokes[0]?.strokeWidth]).toEqual([1.2, 1.2666666666666666]);
    expect(icon.children).toEqual([viewport, circle, path]);
    expect(createShapeFromSvg).toHaveBeenCalledTimes(1);
  });

  it('reads inline pixel widths from the original SVG', () => {
    const { board, icon, circle, path, field } = searchFixture();
    icon.width = 512;
    icon.height = 512;
    circle.strokes = [{ strokeColor: '#000000', strokeWidth: 32 }];
    path.strokes = [{ strokeColor: '#000000', strokeWidth: 32 }];
    field.icons.search = '<svg viewBox="0 0 512 512"><line style="fill:none;stroke:#000;stroke-width:32px"/><line style="fill:none;stroke:#000;stroke-width:32px"/></svg>';
    refreshFieldDecoration(board, field);
    expect([circle.strokes[0]?.strokeWidth, path.strokes[0]?.strokeWidth]).toEqual([1, 1]);
    refreshFieldDecoration(board, field);
    expect([circle.strokes[0]?.strokeWidth, path.strokes[0]?.strokeWidth]).toEqual([1, 1]);
  });

  const edits: Array<[string, (shape: ReturnType<typeof makeShape>) => void]> = [
    ['stroke width', (shape) => { shape.strokes = [{ ...shape.strokes[0]!, strokeWidth: 2 }]; }],
    ['stroke color', (shape) => { shape.strokes = [{ ...shape.strokes[0]!, strokeColor: '#ff0000' }]; }],
    ['stroke opacity', (shape) => { shape.strokes = [{ ...shape.strokes[0]!, strokeOpacity: 0.5 }]; }],
    ['stroke cap', (shape) => { shape.strokes = [{ ...shape.strokes[0]!, strokeCapEnd: 'square' }]; }],
    ['fill', (shape) => { shape.fills = [{ fillColor: '#ff0000', fillOpacity: 1 }]; }],
  ];
  it.each(edits)('preserves a user %s edit for strict export rejection', (_name, edit) => {
    const { board, circle, field } = searchFixture();
    refreshFieldDecoration(board, field);
    writeExportShapeGuard(board);
    assertExportShapeGuard(board);
    edit(circle);
    const paint = { fills: circle.fills, strokes: circle.strokes };
    const writes = { ...circle.paintWrites };
    refreshFieldDecoration(board, field);
    expect(() => refreshFieldDecoration(board, { ...field, iconSize: 20 })).toThrow(/paint changed without a ZUI source mapping/);
    expect(circle.fills).toBe(paint.fills);
    expect(circle.strokes).toBe(paint.strokes);
    expect(circle.paintWrites).toEqual(writes);
    expect(() => assertExportShapeGuard(board)).toThrow(/changed (?:strokes|fills) without a ZUI source mapping/);
  });

  it('preserves mixed group fills while tinting leaves and rejecting user paint edits', () => {
    const { board, icon, circle, path, viewport, field, sourceSvg } = searchFixture();
    icon.fills = 'mixed';
    circle.fills = [{ fillColor: '#cbd2dc', fillOpacity: 1 }];
    field.icons.search = sourceSvg.replace('fill="none"', 'fill="#cbd2dc"');
    const groupFillWrites = icon.paintWrites.fills;
    refreshFieldDecoration(board, field);
    expect(icon.fills).toBe('mixed');
    expect(icon.paintWrites.fills).toBe(groupFillWrites);
    expect(circle.fills).toEqual([{ fillColor: '#c0c0c0', fillOpacity: 1 }]);
    expect(path.strokes[0]?.strokeColor).toBe('#c0c0c0');

    const nextField = { ...field, iconColor: '#ff0000', iconSize: 20 };
    refreshFieldDecoration(board, nextField);
    expect(icon.fills).toBe('mixed');
    expect(icon.paintWrites.fills).toBe(groupFillWrites);
    expect(circle.fills).toEqual([{ fillColor: '#ff0000', fillOpacity: 1 }]);
    expect([circle.strokes[0]?.strokeWidth, path.strokes[0]?.strokeWidth]).toEqual([1.5, 1.5833333333333333]);
    writeExportShapeGuard(board);
    assertExportShapeGuard(board);

    circle.fills = [{ fillColor: '#00ff00', fillOpacity: 1 }];
    const shapes = [icon, viewport, circle, path];
    const before = shapes.map((shape) => ({ fills: shape.fills, strokes: shape.strokes, writes: { ...shape.paintWrites } }));
    refreshFieldDecoration(board, nextField);
    expect(() => assertExportShapeGuard(board)).toThrow(/changed fills without a ZUI source mapping/);
    expect(() => refreshFieldDecoration(board, { ...nextField, iconSize: 16 })).toThrow(/paint changed without a ZUI source mapping/);
    for (const [index, shape] of shapes.entries()) {
      expect(shape.fills).toBe(before[index]!.fills);
      expect(shape.strokes).toBe(before[index]!.strokes);
      expect(shape.paintWrites).toEqual(before[index]!.writes);
    }
    expect(icon.fills).toBe('mixed');
    expect(icon.paintWrites.fills).toBe(groupFillWrites);
  });

  it('ignores paint object key order while comparing every value exactly', () => {
    const { board, circle, path, field } = searchFixture();
    refreshFieldDecoration(board, field);
    for (const shape of [circle, path]) {
      shape.strokes = shape.strokes.map((stroke) =>
        Object.fromEntries(Object.entries(stroke).reverse()));
    }
    refreshFieldDecoration(board, { ...field, iconColor: '#ff0000', iconSize: 20 });
    expect([circle.strokes[0]?.strokeWidth, path.strokes[0]?.strokeWidth]).toEqual([1.5, 1.5833333333333333]);
    expect([circle.strokes[0]?.strokeColor, path.strokes[0]?.strokeColor]).toEqual(['#ff0000', '#ff0000']);
  });

  it('checks the whole tree before a changed desired paint can write earlier siblings', () => {
    const { board, icon, viewport, circle, path, field } = searchFixture();
    refreshFieldDecoration(board, field);
    path.strokes = [{ ...path.strokes[0]!, strokeWidth: 3 }];
    const shapes = [icon, viewport, circle, path];
    const before = shapes.map((shape) => ({ fills: shape.fills, strokes: shape.strokes, writes: { ...shape.paintWrites } }));
    expect(() => refreshFieldDecoration(board, { ...field, iconColor: '#ff0000' })).toThrow(/paint changed without a ZUI source mapping/);
    for (const [index, shape] of shapes.entries()) {
      expect(shape.fills).toBe(before[index]!.fills);
      expect(shape.strokes).toBe(before[index]!.strokes);
      expect(shape.paintWrites).toEqual(before[index]!.writes);
    }
  });

  it('refuses to rebase an existing icon without original paint metadata', () => {
    const { board, icon, circle, field, createShapeFromSvg } = searchFixture();
    icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'field-icon', 'search');
    board.appendChild(icon);
    circle.strokes = [{ ...circle.strokes[0]!, strokeWidth: 1.7 }];
    const strokes = circle.strokes;
    const writes = { ...circle.paintWrites };
    expect(() => refreshFieldDecoration(board, field)).toThrow(/no original paint metadata; reimport it/);
    expect(circle.strokes).toBe(strokes);
    expect(circle.paintWrites).toEqual(writes);
    expect(createShapeFromSvg).not.toHaveBeenCalled();
  });
});
