import type { Board, Shape, Text } from '@penpot/plugin-types';
import type { FieldProjection } from './bridge/zui-field-projection';
import { refreshFieldDecoration, layoutFieldText, planSearchFieldWidthConstraints } from './penpot-field-controls';
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
  const createShapeFromSvg = vi.fn((_svg: string) => icon);
  vi.stubGlobal('penpot', { createBoard: () => makeShape(), createShapeFromSvg });
  const field = makeField({ iconColor: '#c0c0c0', icons: { search: sourceSvg } });
  return { board, viewport, circle, path, icon, field, sourceSvg, createShapeFromSvg };
}

describe('field icon placement', () => {
  afterEach(() => vi.unstubAllGlobals());

  it.each([592, 852, 1232])('allocates the Search surface and text lane at consumer width %i', (width) => {
    const f = searchFixture();
    f.board.resize(width, 32);
    refreshFieldDecoration(f.board, f.field);
    const text = makeShape({ type: 'text', hidden: false }) as unknown as import('@penpot/plugin-types').Text;
    f.board.appendChild(text);
    layoutFieldText(f.board, text, f.field);
    const surface = f.board.children.find((part) => part.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'field-part') === 'surface')!;
    expect(surface.width).toBe(width);
    expect(surface.constraintsHorizontal).toBe('leftright');
    expect(f.icon.constraintsHorizontal).toBe('left');
    expect(text.constraintsHorizontal).toBe('leftright');
    expect(text.width).toBe(width - 36);
    expect(text.x - f.board.x).toBe(28);
  });

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


describe('canonical SearchField width plan', () => {
  afterEach(() => vi.unstubAllGlobals());

  function fixture() {
    const current = searchFixture();
    refreshFieldDecoration(current.board, current.field);
    const text = makeShape({ type: 'text', constraintsHorizontal: 'left', hidden: false });
    text.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'role', 'text');
    current.board.appendChild(text);
    layoutFieldText(current.board, text as unknown as import('@penpot/plugin-types').Text, current.field);
    // Model the historical native master before the width-policy migration.
    for (const child of current.board.children) child.constraintsHorizontal = 'left';
    return { ...current, text };
  }
  it('plans only surface/text constraints without geometry, paint or identity writes', () => {
    const { board, field, text, icon } = fixture();
    const before = board.children.map((part) => [part.id, part.width, part.height, part.x, part.y,
      part.constraintsHorizontal, part.fills, part.strokes]);
    const plan = planSearchFieldWidthConstraints(board, field);
    expect(plan.map(({ shape, after }) => [shape.id, after])).toEqual([
      [board.children[0].id, 'leftright'], [text.id, 'leftright'],
    ]);
    expect(plan.some(({ shape }) => shape.id === icon.id)).toBe(false);
    expect(board.children.map((part) => [part.id, part.width, part.height, part.x, part.y,
      part.constraintsHorizontal, part.fills, part.strokes])).toEqual(before);
  });
  it.each([832, 1280])('refuses stale 240/204 internals at root width %i before any constraint writes', (width) => {
    const { board, field } = fixture();
    board.width = width;
    expect(() => planSearchFieldWidthConstraints(board, field)).toThrow('geometry differs');
    expect(board.children.every((child) => child.constraintsHorizontal === 'left')).toBe(true);
  });
  it('refuses extra topology, unsupported variants and native constraint overrides', () => {
    const { board, field, text } = fixture();
    expect(() => planSearchFieldWidthConstraints(board, { ...field, clear: true })).toThrow('canonical default');
    text.constraintsHorizontal = 'right';
    expect(() => planSearchFieldWidthConstraints(board, field)).toThrow('constraint override');
    text.constraintsHorizontal = 'left';
    board.appendChild(makeShape());
    expect(() => planSearchFieldWidthConstraints(board, field)).toThrow('topology changed');
  });

  it.each(['stretched icon', 'foreign parent', 'edited paint', 'hidden text', 'non-finite allocation'])(
    'refuses %s without changing any native constraint or guard', (change) => {
      const f = fixture();
      const surface = f.board.children[0];
      if (change === 'stretched icon') f.icon.constraintsHorizontal = 'leftright';
      if (change === 'foreign parent') Object.assign(surface, { parent: makeShape() });
      if (change === 'edited paint') surface.fills = [{ fillColor: '#ff0000', fillOpacity: 1 }];
      if (change === 'hidden text') f.text.hidden = true;
      if (change === 'non-finite allocation') f.board.width = Number.NaN;
      const snapshot = () => JSON.stringify(f.board.children.map((shape) => ({
        id: shape.id, parent: shape.parent?.id, x: shape.x, y: shape.y,
        width: shape.width, height: shape.height, hidden: shape.hidden,
        constraint: shape.constraintsHorizontal, fills: shape.fills, strokes: shape.strokes,
        partGuard: shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'control-part-paint'),
        textGuard: shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'control-text-geometry'),
      })));
      const before = snapshot();
      expect(() => planSearchFieldWidthConstraints(f.board, f.field)).toThrow();
      expect(snapshot()).toBe(before);
    },
  );

  it.each(['geometry', 'constraint'])('keeps the original export guard rejecting a direct %s edit', (change) => {
    const f = fixture();
    writeExportShapeGuard(f.board);
    const guard = f.board.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'export-shapes-guard');
    if (change === 'geometry') f.text.x += 1;
    else f.text.constraintsHorizontal = 'right';
    expect(() => assertExportShapeGuard(f.board)).toThrow('without a ZUI source mapping');
    expect(() => planSearchFieldWidthConstraints(f.board, f.field)).toThrow();
    expect(f.board.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'export-shapes-guard')).toBe(guard);
  });
});

describe('field native width policy', () => {
  afterEach(() => vi.unstubAllGlobals());

  it.each([
    { property: 'query', characters: '角色搜索' },
    { property: 'placeholder', characters: 'Search World Settings' },
  ])('preserves $property content, font and source metadata while reusing stretched parts', ({ property, characters }) => {
    const f = searchFixture();
    const text = Object.assign(makeShape({ type: 'text', hidden: false }), {
      characters, fontId: 'custom-2ddd793a-5f89-40b0-aec1-fbe879e37a57',
      fontVariantId: 'normal-400', fontFamily: 'Noto Sans SC', fontSize: '13.333333',
    }) as unknown as Text;
    text.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'role', 'text');
    text.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-property', property);
    text.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'sourceNodeId', 'root');
    f.board.appendChild(text);
    refreshFieldDecoration(f.board, f.field);
    layoutFieldText(f.board, text, f.field);
    const surface = f.board.children.find((shape) =>
      shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'field-part') === 'surface')!;
    const ids = f.board.children.map((shape) => shape.id);
    // Assert the native descriptor before resizing, not just a recomputed formula.
    expect(surface.constraintsHorizontal).toBe('leftright');
    expect(text.constraintsHorizontal).toBe('leftright');
    expect(f.icon.constraintsHorizontal).toBe('left');
    // These mocks do not implement Penpot native resizing. This loop covers the
    // existing source refresh path; Assets resize/save/reopen is a separate gate.
    for (const width of [1280, 832, 900, 640, 240]) {
      f.board.resize(width, 32);
      refreshFieldDecoration(f.board, f.field);
      layoutFieldText(f.board, text, f.field);
      expect(f.board.children.map((shape) => shape.id)).toEqual(ids);
      expect(surface.width).toBe(width);
      expect(surface.x - f.board.x).toBe(0);
      expect(text.x - f.board.x).toBe(28);
      expect(text.x + text.width).toBe(f.board.x + width - 8);
      expect([f.icon.width, f.icon.height]).toEqual([16, 16]);
      expect(f.icon.x - f.board.x).toBe(8);
      expect(text.characters).toBe(characters);
      expect(text.fontId).toBe('custom-2ddd793a-5f89-40b0-aec1-fbe879e37a57');
      expect(text.fontVariantId).toBe('normal-400');
      expect(text.fontFamily).toBe('Noto Sans SC');
      expect(text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-property')).toBe(property);
      expect(text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'sourceNodeId')).toBe('root');
    }
    expect(f.createShapeFromSvg).toHaveBeenCalledTimes(1);
  });

  it('reapplies the surface width policy when a source refresh reuses an existing part', () => {
    const f = searchFixture();
    refreshFieldDecoration(f.board, f.field);
    const surface = f.board.children.find((shape) =>
      shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'field-part') === 'surface')!;
    surface.constraintsHorizontal = 'left';
    refreshFieldDecoration(f.board, f.field);
    expect(f.board.children.find((shape) => shape.id === surface.id)).toBe(surface);
    expect(surface.constraintsHorizontal).toBe('leftright');
  });

  it('keeps a separate field label fixed while the value lane stretches', () => {
    const f = searchFixture();
    const field = { ...f.field, search: false, labelWidth: 96 };
    const value = makeShape({ type: 'text', hidden: false }) as unknown as Text;
    const label = makeShape({ type: 'text', hidden: false }) as unknown as Text;
    label.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-fragment', 'field-label');
    f.board.appendChild(value);
    f.board.appendChild(label);
    for (const width of [240, 832]) {
      f.board.resize(width, 32);
      refreshFieldDecoration(f.board, field);
      layoutFieldText(f.board, value, field);
      layoutFieldText(f.board, label, field);
      expect(value.constraintsHorizontal).toBe('leftright');
      expect(value.x - f.board.x).toBe(104);
      expect(value.width).toBe(width - 112);
      expect(label.constraintsHorizontal).toBe('left');
      expect(label.x - f.board.x).toBe(8);
      expect(label.width).toBe(80);
    }
  });

  it('keeps clear and search icons fixed and preserves the clear text reserve', () => {
    const f = searchFixture();
    const clear = makeShape({ type: 'group', width: 24, height: 24,
      strokes: [{ strokeColor: '#ffffff', strokeOpacity: 1, strokeWidth: 1.5 }] });
    const clearSvg = '<svg viewBox="0 0 24 24"><path stroke="#fff" stroke-width="1.5" d="M0 0 L24 24"/></svg>';
    f.createShapeFromSvg.mockImplementation((svg) => svg === f.sourceSvg ? f.icon : clear);
    const field = { ...f.field, clear: true, icons: { ...f.field.icons, clear: clearSvg } };
    const text = makeShape({ type: 'text', hidden: false }) as unknown as Text;
    f.board.appendChild(text);
    for (const width of [240, 1280, 640]) {
      f.board.resize(width, 32);
      refreshFieldDecoration(f.board, field);
      layoutFieldText(f.board, text, field);
      expect([f.icon.width, f.icon.height, clear.width, clear.height]).toEqual([16, 16, 16, 16]);
      expect(f.icon.constraintsHorizontal).toBe('left');
      expect(clear.constraintsHorizontal).toBe('right');
      expect(clear.x - f.board.x).toBe(width - 24);
      expect(text.constraintsHorizontal).toBe('leftright');
      expect(text.x + text.width).toBe(clear.x - 8);
    }
    expect(f.createShapeFromSvg).toHaveBeenCalledTimes(2);
  });
});
