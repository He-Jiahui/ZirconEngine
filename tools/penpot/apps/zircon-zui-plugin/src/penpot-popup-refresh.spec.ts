import { hasCurrentPopupLayoutGeometry } from './penpot-popup-layout-geometry';
import type { Board, Shape, Text } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import { projectZuiDocument } from './bridge/penpot-projection';
import { refreshPopupOverlays } from './penpot-popup-controls';
import {
  assertExportShapeGuard,
  writeExportShapeGuard,
} from './penpot-export-guard';

function fixture() {
  let sequence = 0;
  function create(type: 'board' | 'text', characters = ''): Board | Text {
    const data = new Map<string, string>();
    const shape = {
      id: `shape-${++sequence}`,
      type,
      name: type,
      x: 0,
      y: 0,
      width: 1,
      height: 1,
      hidden: false,
      opacity: 1,
      rotation: 0,
      flipX: false,
      flipY: false,
      borderRadius: 0,
      fills: [],
      strokes: [],
      children: [] as Shape[],
      parent: null as Board | null,
      characters,
      layoutChild: { absolute: true },
      getSharedPluginData: (_: string, key: string) => data.get(key) ?? '',
      setSharedPluginData: (_: string, key: string, value: string) => {
        data.set(key, value);
      },
      resize(width: number, height: number) {
        this.width = width;
        this.height = height;
      },
      appendChild(child: Shape) {
        const parent = child.parent;
        if (parent && 'children' in parent) {
          const i = parent.children.indexOf(child);
          if (i >= 0) parent.children.splice(i, 1);
        }
        Object.assign(child, { parent: this });
        this.children.push(child);
      },
      remove() {
        const i = this.parent?.children.indexOf(this as unknown as Shape) ?? -1;
        if (i >= 0) this.parent!.children.splice(i, 1);
        this.parent = null;
      },
    };
    return shape as unknown as Board | Text;
  }
  vi.stubGlobal('penpot', {
    createBoard: () => create('board'),
    createText: (value: string) => create('text', value),
    fonts: {
      all: [
        {
          fontFamily: 'Fira Sans',
          variants: [
            { fontWeight: '400', fontStyle: 'normal' },
            { fontWeight: '600', fontStyle: 'normal' },
          ],
          applyToText: (text: Text, variant: { fontWeight: string }) => {
            text.fontFamily = 'Fira Sans';
            text.fontWeight = variant.fontWeight;
          },
        },
      ],
    },
  });
  const asset = create('board') as Board;
  asset.name = 'Asset';
  asset.resize(960, 640);
  asset.setSharedPluginData('zircon-zui', 'role', 'asset');
  const popup = create('board') as Board;
  popup.name = 'popup';
  popup.x = 100;
  popup.y = 80;
  popup.resize(200, 100);
  popup.setSharedPluginData('zircon-zui', 'role', 'node');
  popup.setSharedPluginData('zircon-zui', 'node-id', 'popup');
  asset.appendChild(popup);
  const document: ZuiDocument = {
    asset: { kind: 'view', id: 'popup-test', version: 2 },
    root: { node: 'popup' },
    nodes: {
      popup: {
        component: 'ContextActionMenu',
        props: {
          popup_open: true,
          options: ['one|label=One', '---', 'two|label=Two,selected'],
        },
      },
    },
  };
  const refresh = () =>
    refreshPopupOverlays(asset, document, projectZuiDocument(document));
  refresh();
  const overlay = asset.children.find(
    (child) =>
      child.getSharedPluginData('zircon-zui', 'popup-overlay-for') === 'popup',
  ) as Board;
  const rows = () => overlay.children as Board[];
  const ids = () =>
    rows().map((row) => [row.id, ...row.children.map((child) => child.id)]);
  writeExportShapeGuard(asset);
  return { asset, popup, overlay, rows, ids, document, refresh };
}
afterEach(() => vi.unstubAllGlobals());
it('preserves popup row and label identities across a repeated public refresh', () => {
  const f = fixture(),
    ids = f.ids();
  expect(() => assertExportShapeGuard(f.asset)).not.toThrow();
  f.refresh();
  expect(f.ids()).toEqual(ids);
  expect(() => assertExportShapeGuard(f.asset)).not.toThrow();
});
it('keeps a native popup allocation exportable after its source board moves and resizes', () => {
  const f = fixture(),
    ids = f.ids(),
    guard = f.asset.getSharedPluginData('zircon-zui', 'export-shapes-guard');
  f.popup.x += 24;
  f.popup.y += 8;
  f.popup.resize(230, 112);
  f.refresh();
  expect(f.overlay).toMatchObject({ x: 124, y: 88, width: 230, height: 112 });
  expect(f.rows()[0]).toMatchObject({ x: 132, y: 96, width: 214, height: 28 });
  expect(f.ids()).toEqual(ids);
  expect(() => assertExportShapeGuard(f.asset)).not.toThrow();
  expect(f.asset.getSharedPluginData('zircon-zui', 'export-shapes-guard')).toBe(
    guard,
  );
});
it('continues rejecting an unmapped direct popup row resize before a refresh can overwrite it', () => {
  const f = fixture();
  f.rows()[0].resize(300, 28);
  expect(() => assertExportShapeGuard(f.asset)).toThrow(
    /without a ZUI source mapping/,
  );
});
it('continues rejecting an unmapped direct popup label edit', () => {
  const f = fixture();
  (f.rows()[0].children[0] as Text).characters = 'Unmapped edit';
  expect(() => assertExportShapeGuard(f.asset)).toThrow(
    /without a ZUI source mapping/,
  );
});
it('updates source option state and labels while reusing their existing rows', () => {
  const f = fixture(),
    ids = f.ids();
  f.document.nodes!['popup'].props!['options'] = [
    'one|label=First,selected',
    '---',
    'two|label=Second',
  ];
  f.refresh();
  expect(f.ids()).toEqual(ids);
  expect((f.rows()[0].children[0] as Text).characters).toBe('First');
  expect((f.rows()[2].children[0] as Text).characters).toBe('Second');
});

it('rejects directly resized popup overlay geometry even when it has renderer provenance', () => {
  const f = fixture();
  f.overlay.resize(260, 100);
  expect(() => assertExportShapeGuard(f.asset)).toThrow(
    /without a ZUI source mapping/,
  );
});
it('rejects a directly moved label while allowing its parent to move through native layout', () => {
  const f = fixture();
  const label = f.rows()[0].children[0];
  label.x += 3;
  expect(() => assertExportShapeGuard(f.asset)).toThrow(
    /without a ZUI source mapping/,
  );
});
it('rejects an unmarked auxiliary node even if it copies a popup geometry record', () => {
  const f = fixture();
  f.rows()[0].setSharedPluginData('zircon-zui', 'popup-content-for', '');
  expect(() => assertExportShapeGuard(f.asset)).toThrow(
    /without a ZUI source mapping/,
  );
});

it('requires an auxiliary explicit popup root before accepting an allocation record', () => {
  const f = fixture();
  expect(hasCurrentPopupLayoutGeometry(f.rows()[0])).toBe(true);
  f.overlay.setSharedPluginData('zircon-zui', 'role', 'node');
  expect(hasCurrentPopupLayoutGeometry(f.rows()[0])).toBe(false);
  f.overlay.setSharedPluginData('zircon-zui', 'role', 'auxiliary');
  f.overlay.setSharedPluginData('zircon-zui', 'explicit-overlay', 'false');
  expect(hasCurrentPopupLayoutGeometry(f.rows()[0])).toBe(false);
});

it.each([
  'fontSize',
  'fontFamily',
  'fontWeight',
  'lineHeight',
  'align',
  'verticalAlign',
])('rejects an unmapped popup label %s edit before refreshing', (property) => {
  const f = fixture();
  const text = f.rows()[0].children[0] as Text;
  Object.assign(text, { [property]: 'unsupported' });
  expect(() => assertExportShapeGuard(f.asset)).toThrow(
    /without a ZUI source mapping/,
  );
});
