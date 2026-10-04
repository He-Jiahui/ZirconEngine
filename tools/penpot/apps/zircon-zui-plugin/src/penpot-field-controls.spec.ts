import type { Board, Shape } from '@penpot/plugin-types';
import type { FieldProjection } from './bridge/zui-field-projection';
import { refreshFieldDecoration } from './penpot-field-controls';

function makeShape(initial: Partial<Shape> = {}): Shape & {
  resizeCalls: Array<[number, number]>;
} {
  const data = new Map<string, string>();
  const shape = {
    id: `shape-${Math.random()}`,
    name: '',
    type: 'board',
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
    },
    remove: () => undefined,
    resize(width: number, height: number) {
      this.resizeCalls.push([width, height]);
      this.width = width;
      this.height = height;
    },
    ...initial,
  };
  return shape as unknown as Shape & {
    resizeCalls: Array<[number, number]>;
  };
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

    const field: FieldProjection = {
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
    };

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
