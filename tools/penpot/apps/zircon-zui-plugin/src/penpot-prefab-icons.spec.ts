import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import type { Shape, Text } from '@penpot/plugin-types';
import { LayoutDependencies } from '../tools/zui-layout-dependencies';
import { trackedZuiPaths } from '../tools/zui-layout-catalog';
import { cloneZuiDocument, parseZuiDocument, type ZuiDocument, type ZuiNode } from './bridge/zui-document';
import { createPrefabIcon, previewIconTint, recolorIconShape, nativeSemanticGlyph,
  nativeSemanticGlyphGeometry, refreshNativeSemanticIcons, layoutNativeSemanticText } from './penpot-prefab-icons';
import { ZUI_METADATA_NAMESPACE } from './metadata';
import { hostButtonPreviewNode } from './bridge/zui-editor-button-projection';
import { isEditorTabCloseIconButton } from './bridge/zui-editor-button-identity';
import { editorTabCloseStyle } from './bridge/zui-editor-button-style';
import { resolveDesignColor } from './bridge/zui-prefab-system';
import type { Board } from '@penpot/plugin-types';

describe('native IconButton design projection', () => {
  const root = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../', import.meta.url)));
  const assetPath =
    'zircon_editor/assets/ui/editor/workbench/primitives/inputs/icon_button.zui';
  const themePath = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
  const headerPath = 'zircon_editor/assets/ui/editor/unreal/components/dock_header.zui';
  const viewportPath = 'zircon_editor/assets/ui/editor/unreal/components/viewport.zui';
  const toolbarPath = 'zircon_editor/assets/ui/editor/unreal/components/toolbar.zui';
  const dependencies = new LayoutDependencies();

  beforeAll(async () => dependencies.load(root, [assetPath, themePath, headerPath,
    'zircon_editor/assets/ui/editor/unreal/starship.zui', viewportPath, toolbarPath,
    'zircon_editor/assets/ui/editor/workbench/primitives/inputs/dropdown.zui']));
  afterEach(() => vi.unstubAllGlobals());

  it.each(['modes', 'settings'])('tints the actual %s Button SVG with its native painter color', (id) => {
    const document = parseZuiDocument(readFileSync(`${root}/${toolbarPath}`, 'utf8')).document;
    dependencies.embed(document, toolbarPath, themePath);
    const node = document.nodes![id];
    const before = structuredClone(node);
    const svg = readFileSync(`${root}/zircon_editor/assets/icons/${String(node.props!['icon'])}`, 'utf8');
    node['penpot_icon_svg'] = svg;
    const icon = { opacity: 1, fills: [{ fillColor: '#26d8d1' }],
      strokes: [{ strokeColor: '#bafff8' }, { strokeColor: 'transparent' }],
      setSharedPluginData: vi.fn(), resize: vi.fn(), x: 0, y: 0, name: '' };
    vi.stubGlobal('penpot', { createShapeFromSvg: vi.fn(() => icon) });
    const tint = resolveDesignColor(document, hostButtonPreviewNode(document, node).props!['icon_color']);
    expect(tint).toBe('#C0C0C0');
    expect(previewIconTint(document, node)).toBe(tint);
    createPrefabIcon({ x: 100, y: 200, width: 160, height: 32,
      appendChild: vi.fn() } as unknown as Board, node, document, false);
    expect(icon.fills[0].fillColor).toBe(tint);
    expect(icon.strokes.map((stroke) => stroke.strokeColor)).toEqual([tint, 'transparent']);
    expect(icon.resize).toHaveBeenCalledWith(20, 20);
    expect(icon.y + 10).toBe(216);
    expect({ ...node, penpot_icon_svg: undefined }).toEqual({ ...before, penpot_icon_svg: undefined });
    delete document['penpot_host_theme_source'];
    expect(previewIconTint(document, node)).toBeNull();
  });

  it.each(['modes', 'settings'])('resolves the actual %s text Button stylesheet before selecting glyph ink', (id) => {
    const document = parseZuiDocument(readFileSync(`${root}/${toolbarPath}`, 'utf8')).document;
    dependencies.embed(document, toolbarPath, themePath);
    const node = document.nodes![id];
    expect(node.props?.['foreground_color']).toBeUndefined();
    for (const [state, color] of [
      [{ hovered: true }, '#FFFFFF'],
      [{ pressed: true }, '#FFFFFF'],
      [{ disabled: true, hovered: true, pressed: true }, '#808080'],
    ] as const) {
      node.state = { ...state };
      const before = structuredClone(node);
      expect(previewIconTint(document, node)).toBe(color);
      expect(node).toEqual(before);
    }
  });

  it.each([
    [{}, '#FFFFFF'],
    [{ selected: false, checked: false }, '#C0C0C0'],
    [{ selected: false, checked: false, hovered: true }, '#FFFFFF'],
    [{ selected: false, checked: false, pressed: true }, '#FFFFFF'],
    [{ disabled: true, selected: true, hovered: true }, '#808080'],
  ])('uses generic Native image foreground for the actual grid snap state %j', (state, color) => {
    const document = parseZuiDocument(readFileSync(`${root}/${viewportPath}`, 'utf8')).document;
    dependencies.embed(document, viewportPath, themePath);
    const node = document.nodes!['move_snap_toggle'];
    node.state = state;
    expect(node.props?.['icon_color']).toBeUndefined();
    expect(node.props?.['foreground_color']).toBeUndefined();
    const before = structuredClone(node);
    expect(previewIconTint(document, node)).toBe(color);
    expect(node).toEqual(before);
  });

  it('keeps generic image severity and disabled ink ahead of styled active foreground', () => {
    const document = parseZuiDocument(readFileSync(`${root}/${viewportPath}`, 'utf8')).document;
    dependencies.embed(document, viewportPath, themePath);
    const node = document.nodes!['move_snap_toggle'];
    node.state = { validation_level: 'warning' };
    expect(previewIconTint(document, node)).toBe('#FFB800');
    node.state = { validation_level: 'error', text_tone: 'warning' };
    expect(previewIconTint(document, node)).toBe('#EF3535');
    node.state = { disabled: true, validation_level: 'error' };
    expect(previewIconTint(document, node)).toBe('#808080');
  });

  const ordinaryIconFixture = (document: ZuiDocument, node: ZuiNode, width: number, height: number,
    firstFillColor = '#26d8d1') => {
    const values: Record<string, string> = {};
    const icon = { id: 'ordinary-svg', type: 'group', x: 0, y: 0, width: 24, height: 24,
      opacity: 0.75, hidden: false,
      fills: [{ fillColor: firstFillColor, fillOpacity: 0.35 }, { fillColor: 'transparent' }],
      strokes: [{ strokeColor: '#bafff8', strokeOpacity: 0.6 }], children: [],
      getSharedPluginData: (_namespace: string, key: string) => values[key] ?? '',
      setSharedPluginData: (_namespace: string, key: string, value: string) => { values[key] = value; },
      resize(nextWidth: number, nextHeight: number) { this.width = nextWidth; this.height = nextHeight; } };
    const resized = vi.spyOn(icon, 'resize');
    const children: Shape[] = [];
    const board = { x: 100, y: 200, width, height, children,
      appendChild: (child: Shape) => children.push(child) } as unknown as Board;
    const create = vi.fn(() => icon);
    vi.stubGlobal('penpot', { createShapeFromSvg: create });
    node['penpot_icon_svg'] = readFileSync(`${root}/zircon_editor/assets/icons/${String(node.props!['icon'])}`, 'utf8');
    createPrefabIcon(board, node, document, node.component === 'IconButton');
    return { board, children, icon, resized, create, values };
  };

  it.each(['modes', 'settings'])('shares authored %s icon size and stylesheet tint through create, refresh and reopen', (id) => {
    const document = parseZuiDocument(readFileSync(`${root}/${toolbarPath}`, 'utf8')).document;
    dependencies.embed(document, toolbarPath, themePath);
    const node = document.nodes![id];
    const { board, children, icon, resized, create, values } = ordinaryIconFixture(document, node, 160, 32);
    const original = structuredClone(node);
    expect([icon.width, icon.height, icon.x, icon.y]).toEqual([20, 20, 110, 206]);
    expect(icon.hidden).toBe(true); // Full label ink is not measured during creation.
    expect(icon.fills[0]!.fillColor).toBe('#C0C0C0');
    expect(values['role']).toBe('auxiliary');
    expect(values['sourceNodeId']).toBeUndefined();
    refreshNativeSemanticIcons(board, node, document, 100);
    expect(icon.hidden).toBe(false);
    expect(resized).toHaveBeenCalledTimes(1);
    board.height = 40;
    refreshNativeSemanticIcons(board, node, document, 100);
    expect([icon.width, icon.height, icon.y]).toEqual([20, 20, 210]);
    expect(resized).toHaveBeenCalledTimes(1);
    expect(node).toEqual(original);

    node.state = { hovered: true };
    refreshNativeSemanticIcons(board, node, document, 100);
    expect(icon.fills[0]!.fillColor).toBe('#FFFFFF');
    const reopened = { ...icon, fills: icon.fills.map((fill) => ({ ...fill })),
      strokes: icon.strokes.map((stroke) => ({ ...stroke })) };
    children[0] = reopened as unknown as Shape;
    node.state = { disabled: true, hovered: true };
    refreshNativeSemanticIcons(board, node, document, 100);
    expect(reopened.fills[0]!.fillColor).toBe('#808080');
    expect(reopened.fills[0]!.fillOpacity).toBe(0.35);
    expect(reopened.fills[1]!.fillColor).toBe('transparent');
    expect(reopened.strokes[0]!.strokeOpacity).toBe(0.6);
    expect(reopened.opacity).toBe(0.75);
    expect(create).toHaveBeenCalledTimes(1);
  });

  it.each([
    ['modes', 152, 92.48697509765625, 16.256512451171875, 98.48697509765625],
    ['settings', 106, 49.602862548828126, 14.698568725585937, 55],
  ] as const)('centers the actual %s authored asset and measured label as one Native Button group',
    (id, width, ink, start, expectedSlot) => {
      const document = parseZuiDocument(readFileSync(`${root}/${toolbarPath}`, 'utf8')).document;
      dependencies.embed(document, toolbarPath, themePath);
      const node = document.nodes![id];
      const { board, children, icon, resized, create, values } = ordinaryIconFixture(document, node, width, 32);
      const before = structuredClone(document);
      expect(nativeSemanticGlyph(document, node)).toBeNull();
      expect(nativeSemanticGlyphGeometry(document, node, width, 32, 16)).toMatchObject({
        icon: null, pendingMeasurement: true });
      expect(icon.hidden).toBe(true);
      const geometry = nativeSemanticGlyphGeometry(document, node, width, 32, 16, ink)!;
      expect(geometry.pendingMeasurement).toBe(false);
      expect(geometry.icon).toMatchObject({ y: 6, width: 20, height: 20 });
      expect(geometry.icon!.x).toBeCloseTo(start);
      expect(geometry.text.x).toBeCloseTo(start + 27); // Authored 20 + actual host gap 7.
      expect(geometry.text.width).toBe(expectedSlot); // Clip guard is clamped to the available text lane.
      refreshNativeSemanticIcons(board, node, document, ink);
      expect(icon.x - board.x).toBeCloseTo(start);
      expect(icon.hidden).toBe(false);
      expect(values['native-semantic-glyph']).toBeUndefined();
      expect(values['sourceNodeId']).toBeUndefined();

      for (const align of ['left', 'center', 'right'] as const) {
        const text = { fontSize: '13.333333', lineHeight: '1.2', align,
          resize(nextWidth: number, nextHeight: number) { Object.assign(this, { width: nextWidth, height: nextHeight }); },
          width: 0, height: 0, x: 0, y: 0 } as unknown as Text;
        expect(layoutNativeSemanticText(board, text, node, document, ink)).toBe(true);
        expect(text.align).toBe(align); // Editable alignment stays authored.
        const alignmentOffset = (text.width - ink) * (align === 'center' ? 0.5 : align === 'right' ? 1 : 0);
        expect(text.x - board.x + alignmentOffset).toBeCloseTo(start + 27);
        expect(text.width).toBe(expectedSlot);
      }
      const stableResizeCount = resized.mock.calls.length;
      const reopened = { ...icon, fills: icon.fills.map((fill) => ({ ...fill })),
        strokes: icon.strokes.map((stroke) => ({ ...stroke })) };
      children[0] = reopened as unknown as Shape;
      refreshNativeSemanticIcons(board, node, document, ink);
      refreshNativeSemanticIcons(board, node, document, ink);
      expect(reopened.x - board.x).toBeCloseTo(start);
      expect(resized).toHaveBeenCalledTimes(stableResizeCount);
      expect(create).toHaveBeenCalledTimes(1);
      expect(document).toEqual(before);
    });

  it('uses resolved Button padding/gap/state without painting an unmeasured or undersized leading group', () => {
    const document = parseZuiDocument(readFileSync(`${root}/${toolbarPath}`, 'utf8')).document;
    dependencies.embed(document, toolbarPath, themePath);
    const node = document.nodes!['modes'];
    node.props = { ...node.props, layout_padding_left: 16, layout_padding_right: 4,
      layout_padding_top: 2, layout_padding_bottom: 4, layout_content_offset_x: 10 };
    const { board, icon } = ordinaryIconFixture(document, node, 160, 40);
    expect(icon.hidden).toBe(true);
    refreshNativeSemanticIcons(board, node, document);
    expect(icon.hidden).toBe(true);
    expect(nativeSemanticGlyphGeometry(document, node, 160, 40, 16, 80)).toMatchObject({
      icon: { x: 31, y: 9, width: 20, height: 20 },
      text: { x: 61, y: 11, width: 86, height: 16 } });
    node.state = { layout_content_offset_x: 6, layout_padding_left: 20, pressed: true };
    const before = structuredClone(node);
    expect(nativeSemanticGlyphGeometry(document, node, 160, 40, 16, 80)).toMatchObject({
      icon: { x: 35, y: 10, width: 20, height: 20 },
      text: { x: 61, y: 12, width: 86, height: 16 } });
    expect(node).toEqual(before);
    board.width = 19;
    refreshNativeSemanticIcons(board, node, document, 80);
    expect(icon.hidden).toBe(true);
    board.width = 160;
    refreshNativeSemanticIcons(board, node, document, 80);
    expect(icon.hidden).toBe(false);
    expect(icon.x - board.x).toBe(35);
    expect(() => nativeSemanticGlyphGeometry(document, node, 160, 40, 16, NaN)).toThrow(/font measurement/);
    expect(() => nativeSemanticGlyphGeometry(document, node, Infinity, 40, 16, 80)).toThrow(/dimensions/);
  });

  it.each([
    (document: ZuiDocument, _node: ZuiNode) => { delete document['penpot_host_theme_source']; },
    (_document: ZuiDocument, node: ZuiNode) => { node.component = 'IconButton'; },
    (_document: ZuiDocument, node: ZuiNode) => { node.control_id = 'PageTab1'; },
    (_document: ZuiDocument, node: ZuiNode) => { node.control_id = 'AssetBrowserKindSceneChip'; },
    (_document: ZuiDocument, node: ZuiNode) => { node.control_id = 'AssetsActivityPreviewButton'; },
    (_document: ZuiDocument, node: ZuiNode) => { node.props = { ...node.props, component_variant: 'code COMPACT_ICON_TEXT' }; },
    (_document: ZuiDocument, node: ZuiNode) => {
      node.control_id = 'GenericContained'; node.props = { ...node.props, button_variant: 'contained' };
    },
    (_document: ZuiDocument, node: ZuiNode) => { node.props = { ...node.props, icon_placement: 'trailing' }; },
    (_document: ZuiDocument, node: ZuiNode) => { node.props = { ...node.props, icon_placement: 'icon_only' }; },
    (_document: ZuiDocument, node: ZuiNode) => {
      node.control_id = 'WorkbenchOpenMenu'; node.props = { ...node.props, button_variant: 'secondary' };
    },
    (_document: ZuiDocument, node: ZuiNode) => { node.props = { ...node.props, overflow: 'shrink' }; },
  ])('keeps other painter families outside regular leading Button geometry (%#)', (mutate) => {
    const document = parseZuiDocument(readFileSync(`${root}/${toolbarPath}`, 'utf8')).document;
    dependencies.embed(document, toolbarPath, themePath);
    const node = document.nodes!['modes'];
    mutate(document, node);
    expect(nativeSemanticGlyphGeometry(document, node, 152, 32, 16, 100)).toBeNull();
  });

  it('uses generic image insets for grid snap without applying them to Workbench or Close slots', () => {
    const document = parseZuiDocument(readFileSync(`${root}/${viewportPath}`, 'utf8')).document;
    dependencies.embed(document, viewportPath, themePath);
    const node = document.nodes!['move_snap_toggle'];
    const { board, icon, resized } = ordinaryIconFixture(document, node, 28, 28);
    expect([icon.width, icon.height, icon.x, icon.y]).toEqual([20, 20, 104, 204]);
    node.state = { layout_icon_size: 10 };
    refreshNativeSemanticIcons(board, node, document);
    expect(icon.width).toBe(20); // Generic node_images does not consume the Button slot property.
    delete node.state;
    board.width = board.height = 24;
    refreshNativeSemanticIcons(board, node, document);
    expect(icon.width).toBeCloseTo(16.32);
    expect(icon.x).toBeCloseTo(103.84);
    expect(icon.y).toBeCloseTo(203.84);
    const stableResizeCount = resized.mock.calls.length;
    refreshNativeSemanticIcons(board, node, document);
    expect(resized).toHaveBeenCalledTimes(stableResizeCount);

    const workbench = { ...node, control_id: 'WorkbenchOrdinaryIcon', classes: [], props: { icon: node.props!['icon'] } };
    const separate = ordinaryIconFixture(document, workbench, 28, 28);
    expect(separate.icon.width).toBe(18);
    separate.values['role'] = 'foreign';
    separate.board.height = 40;
    refreshNativeSemanticIcons(separate.board, workbench, document);
    expect(separate.icon.y).toBe(205);
  });

  it('suppresses an authored icon that cannot fit and keeps invalid declarations on the Native Button density fallback', () => {
    const document = parseZuiDocument(readFileSync(`${root}/${toolbarPath}`, 'utf8')).document;
    dependencies.embed(document, toolbarPath, themePath);
    const node = document.nodes!['modes'];
    const { board, icon, resized } = ordinaryIconFixture(document, node, 160, 32);
    board.height = 19;
    refreshNativeSemanticIcons(board, node, document, 100);
    expect(icon.hidden).toBe(true);
    expect(resized).toHaveBeenCalledTimes(1);
    board.height = 32;
    refreshNativeSemanticIcons(board, node, document, 100);
    expect(icon.hidden).toBe(false);
    expect(icon.width).toBe(20);
    for (const invalid of [0, -1, NaN, Infinity]) {
      node.state = { layout_icon_size: invalid };
      refreshNativeSemanticIcons(board, node, document, 100);
      expect(icon.width).toBe(16); // Native Button density fallback, not Workbench IconButton geometry.
    }
  });

  it.each([[toolbarPath, 'modes'], [viewportPath, 'translate_mode']])(
    'preserves authored non-host %s/%s SVG paint and opacity through initial creation and refresh', (path, id) => {
      const document = parseZuiDocument(readFileSync(`${root}/${path}`, 'utf8')).document;
      dependencies.embed(document, path, themePath);
      delete document['penpot_host_theme_source'];
      const node = document.nodes![id];
      const { board, icon } = ordinaryIconFixture(document, node, 160, 32, '#000000');
      expect(icon.fills[0]!.fillColor).toBe('#000000');
      expect(icon.strokes[0]!.strokeColor).toBe('#bafff8');
      const original = structuredClone({ fills: icon.fills, strokes: icon.strokes, opacity: icon.opacity });
      refreshNativeSemanticIcons(board, node, document);
      expect({ fills: icon.fills, strokes: icon.strokes, opacity: icon.opacity }).toEqual(original);
      expect(icon.width).toBe(18);
    });

  it.each(['select_mode', 'translate_mode', 'rotate_mode', 'scale_mode'])(
    'projects viewport transform %s using the actual deselected state', (id) => {
      const document = parseZuiDocument(readFileSync(`${root}/${viewportPath}`, 'utf8')).document;
      dependencies.embed(document, viewportPath, themePath);
      const node = document.nodes![id];
      node.state = { selected: false, checked: false };
      const before = structuredClone(node);
      expect(previewIconTint(document, node)).toBe('#C8C8C8');
      expect(node).toEqual(before);
    },
  );

  it.each([
    [{}, '#FFFFFF'],
    [{ selected: false, checked: false, hovered: true }, '#FFFFFF'],
    [{ selected: false, checked: false, pressed: true }, '#FFFFFF'],
    [{ selected: true, checked: false }, '#FFFFFF'],
    [{ selected: false, checked: true }, '#FFFFFF'],
    [{ disabled: true, hovered: true, pressed: true, selected: true }, '#808080'],
  ])('projects viewport transform interaction %j from the shared UE stylesheet', (state, color) => {
    const document = parseZuiDocument(readFileSync(`${root}/${viewportPath}`, 'utf8')).document;
    dependencies.embed(document, viewportPath, themePath);
    const node = document.nodes!['translate_mode'];
    node.state = state;
    expect(previewIconTint(document, node)).toBe(color);
  });

  it('tints viewport transform SVG paint while preserving its centered geometry', () => {
    const document = parseZuiDocument(readFileSync(`${root}/${viewportPath}`, 'utf8')).document;
    dependencies.embed(document, viewportPath, themePath);
    const node = document.nodes!['translate_mode'];
    node.state = { selected: false, checked: false };
    node['penpot_icon_svg'] = readFileSync(`${root}/zircon_editor/assets/icons/${String(node.props!['icon'])}`, 'utf8');
    const icon = { opacity: 1, fills: [{ fillColor: '#26d8d1' }],
      strokes: [{ strokeColor: '#bafff8' }], setSharedPluginData: vi.fn(),
      resize: vi.fn(), x: 0, y: 0, name: '' };
    vi.stubGlobal('penpot', { createShapeFromSvg: vi.fn(() => icon) });
    createPrefabIcon({ x: 100, y: 200, width: 24, height: 24,
      appendChild: vi.fn() } as unknown as Board, node, document);
    expect(icon.fills[0].fillColor).toBe('#C8C8C8');
    expect(icon.strokes[0].strokeColor).toBe('#C8C8C8');
    expect(icon.resize.mock.calls[0]![0]).toBeCloseTo(16.32);
    expect(icon.resize.mock.calls[0]![1]).toBeCloseTo(16.32);
    expect(icon.x).toBeCloseTo(103.84);
    expect(icon.y).toBeCloseTo(203.84);
    expect(icon.opacity).toBe(1);
  });

  it.each([
    [{}, '#C0C0C0', 1],
    [{ hovered: true }, '#FFFFFF', 1],
    [{ pressed: true }, '#C0C0C0', 1],
    [{ hovered: true, pressed: true }, '#C0C0C0', 1],
    [{ hovered: true, focused: true }, '#FFFFFF', 1],
    [{ disabled: true, hovered: true }, '#FFFFFF', 0.45],
    [{ disabled: true, hovered: true, pressed: true }, '#C0C0C0', 0.45],
    [{ enter_pressed: true, hovered: true }, '#C0C0C0', 1],
  ])('projects the authored UE Close brush without a surface for %j', (state, glyph, opacity) => {
    const document = parseZuiDocument(readFileSync(`${root}/${headerPath}`, 'utf8')).document;
    dependencies.embed(document, headerPath, themePath);
    const node = document.nodes!['dock_tab_close_0'];
    node.state = state;
    expect(isEditorTabCloseIconButton(node)).toBe(true);
    expect(previewIconTint(document, node)).toBe(glyph);
    expect(editorTabCloseStyle(node).opacity).toBe(opacity);
    expect(hostButtonPreviewNode(document, node).props).toMatchObject({
      background_color: 'transparent', border_color: 'transparent', border_width: 0,
    });
    const svg = readFileSync(`${root}/zircon_editor/assets/icons/${String(node.props!['icon'])}`, 'utf8');
    node['penpot_icon_svg'] = svg;
    const icon = { opacity: 1, fills: [], strokes: [], setSharedPluginData: vi.fn(),
      resize: vi.fn(), x: 0, y: 0, name: '' };
    const fromSvg = vi.fn(() => icon);
    vi.stubGlobal('penpot', { createShapeFromSvg: fromSvg });
    const board = { x: 100, y: 200, width: 20, height: 20, appendChild: vi.fn() } as unknown as Board;
    createPrefabIcon(board, node, document);
    expect(fromSvg).toHaveBeenCalledWith(svg);
    expect(icon.resize).toHaveBeenCalledWith(16, 16);
    expect([icon.x, icon.y, icon.opacity]).toEqual([102, 202, opacity]);
  });

  it('restricts the Close painter to the icon family and established identities', () => {
    expect(isEditorTabCloseIconButton({ component: 'Label', control_id: 'DockTabClose0' })).toBe(false);
    expect(isEditorTabCloseIconButton({ component: 'IconButton', control_id: 'OtherClose' })).toBe(false);
    expect(isEditorTabCloseIconButton({ component: 'IconButton', control_id: 'WorkbenchStatusTabClose' })).toBe(false);
  });

  it.each([
    [{}, '#C0C0C0'],
    [{ hovered: true }, '#C0C0C0'],
    [{ pressed: true }, '#0070E0'],
    [{ focused: true }, '#C0C0C0'],
    [{ selected: true }, '#0070E0'],
    [{ disabled: true, selected: true }, '#808080'],
  ])('tints the authored SVG like the retained host for %j', (state, color) => {
    const document = parseZuiDocument(
      readFileSync(`${root}/${assetPath}`, 'utf8'),
    ).document;
    dependencies.embed(document, assetPath, themePath);
    const node = document.nodes!['root'];
    node.state = state;
    expect(previewIconTint(document, node)).toBe(color);
  });

  it('recolors both authored fills and strokes without painting transparent paths', () => {
    const shape = {
      fills: [{ fillColor: '#26d8d1' }, { fillColor: 'transparent' }],
      strokes: [{ strokeColor: '#bafff8' }],
      children: [{ fills: [{ fillColor: '#ffffff' }], strokes: [] }],
    };
    recolorIconShape(shape as unknown as Shape, '#60aeff');
    expect(shape.fills?.map((fill) => fill.fillColor)).toEqual([
      '#60aeff',
      'transparent',
    ]);
    expect(shape.strokes.map((stroke) => stroke.strokeColor)).toEqual([
      '#60aeff',
    ]);
    expect(
      shape.children[0].fills?.[0]
        .fillColor,
    ).toBe('#60aeff');
  });
});

describe('actual missing semantic icons', () => {
  const root = process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../', import.meta.url));
  const details = 'zircon_editor/assets/ui/editor/unreal/components/details.zui';
  const owner = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
  const shell = 'zircon_editor/assets/ui/editor/unreal/editor_shell.zui';
  const dependencies = new LayoutDependencies();
  let source: ZuiDocument;
  beforeAll(async () => {
    await dependencies.load(root, await trackedZuiPaths());
    source = parseZuiDocument(readFileSync(root + '/' + details, 'utf8')).document;
    dependencies.embed(source, details, 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui');
  }, 120_000);
  afterEach(() => vi.unstubAllGlobals());
  const fixture = () => {
    const document = cloneZuiDocument(source);
    const byControl = (control: string) => Object.values(document.nodes!).find((node) => node.control_id === control)!;
    return { document, byControl };
  };

  it('admits the real Add/Transform/Mesh families and complete owner invocation, without a title cube', () => {
    const { document, byControl } = fixture();
    const add = byControl('WorkbenchAddComponent');
    const transform = byControl('WorkbenchTransformLabel'), mesh = byControl('WorkbenchMeshLabel');
    const original = structuredClone(document);
    expect(nativeSemanticGlyph(document, add)?.assetPath).toBe('zircon_editor_shell/controls/add.svg');
    expect(nativeSemanticGlyph(document, transform)?.assetPath).toBe('zircon_editor_shell/inspector/transform.svg');
    expect(nativeSemanticGlyph(document, mesh)?.assetPath).toBe('zircon_editor_shell/inspector/mesh-renderer.svg');
    expect(nativeSemanticGlyph(document, byControl('WorkbenchInspectorTitle'))).toBeNull();
    expect(document).toEqual(original);
    const prefix = [{ sourcePath: shell, sourceNodeId: 'pane_surface' }, { sourcePath: owner, sourceNodeId: 'details' }];
    transform['penpot_review_instance_path'] = JSON.stringify(prefix);
    add['penpot_review_instance_path'] = JSON.stringify([...prefix, { sourcePath: details, sourceNodeId: 'add' }]);
    expect(nativeSemanticGlyph(document, transform)?.family).toBe('section-transform');
    expect(nativeSemanticGlyph(document, add)?.family).toBe('add-component');
    transform['penpot_review_source_path'] = 'res://ui/editor/unreal/components/details.zui';
    expect(nativeSemanticGlyph(document, transform)?.family).toBe('section-transform');
    expect(document['penpot_native_icon_svgs']).toEqual(original['penpot_native_icon_svgs']);
  });

  it.each([
    (node: ZuiNode) => { node.component = 'Label'; },
    (node: ZuiNode) => { node.props = { ...node.props, button_variant: 'text' }; },
    (node: ZuiNode) => { node.props = { ...node.props, text: 'Delete Component' }; },
    (node: ZuiNode) => { node.props = { ...node.props, icon: 'other.svg' }; },
    (node: ZuiNode) => { node['penpot_review_source_path'] = 'other/' + String(node['penpot_review_source_path']); },
    (node: ZuiNode) => { node['penpot_review_source_node_id'] = 'other'; },
    (node: ZuiNode) => { node['penpot_review_instance_path'] = JSON.stringify([{ sourcePath: details, sourceNodeId: 'other' }]); },
    (node: ZuiNode) => { node['penpot_review_instance_path'] = JSON.stringify([
      { sourcePath: owner, sourceNodeId: 'other_details' }, { sourcePath: details, sourceNodeId: 'add' }]); },
  ])('does not infer an Add glyph from a caption/control when admission is invalid (%#)', (mutate) => {
    const { document, byControl } = fixture();
    const node = byControl('WorkbenchAddComponent');
    mutate(node);
    expect(nativeSemanticGlyph(document, node)).toBeNull();
  });

  it('uses current host metrics, actual font ink and Native gap aliases without changing Source', () => {
    const { document, byControl } = fixture();
    const add = byControl('WorkbenchAddComponent');
    add.props = { ...add.props, text: 'Add Component' };
    const before = structuredClone(document);
    expect(nativeSemanticGlyphGeometry(document, add, 200, 32, 16)!.icon).toBeNull();
    const geometry = nativeSemanticGlyphGeometry(document, add, 200, 32, 16, 100)!;
    expect(geometry).toMatchObject({ pendingMeasurement: false,
      icon: { x: 38, y: 8, width: 16, height: 16 }, text: { x: 62, y: 8, width: 106, height: 16 } });
    expect(document).toEqual(before);
    add.props['layout_content_offset_x'] = 10;
    expect(nativeSemanticGlyphGeometry(document, add, 200, 32, 16, 100)!.text.x).toBe(63);
    delete add.props['layout_content_offset_x'];
    add.props['layout_gap'] = 6;
    expect(nativeSemanticGlyphGeometry(document, add, 200, 32, 16, 100)!.text.x).toBe(61);
    delete add.props['layout_gap'];
    add.state = { pressed: true };
    expect(nativeSemanticGlyphGeometry(document, add, 200, 32, 16, 100)!.icon!.y).toBe(9);
    expect(() => nativeSemanticGlyphGeometry(document, add, 200, 32, 16, NaN)).toThrow(/font measurement/);
    const transform = byControl('WorkbenchTransformLabel');
    const full = nativeSemanticGlyphGeometry(document, transform, 160, 24, 16)!;
    expect(full.icon!.width).toBeCloseTo(13.333333);
    expect(full.text.x).toBeCloseTo(29.333333);
    const narrow = nativeSemanticGlyphGeometry(document, transform, 20, 24, 16)!;
    expect(narrow.icon).toBeNull();
    expect(narrow.text.x).toBe(8);
    expect(transform).toEqual(Object.values(before.nodes!).find((node) => node.control_id === 'WorkbenchTransformLabel'));
    document.tokens!['editor.typography.body.size'] = 20;
    expect(nativeSemanticGlyphGeometry(document, transform, 160, 24, 16)!.icon!.width).toBe(20);
  });

  it('lays out localized captions through the same text table as projection without changing their keys', () => {
    const { document, byControl } = fixture();
    const add = byControl('WorkbenchAddComponent');
    add.props = { ...add.props, text: { text_key: 'editor.inspector.component_add.open', table: 'editor' } };
    for (const caption of ['Add Component', '添加组件', '']) {
      document['penpot_i18n_tables'] = { editor: { translations: {
        'editor.inspector.component_add.open': caption,
      } } };
      const original = structuredClone(document);
      const geometry = nativeSemanticGlyphGeometry(document, add, 200, 32, 16, caption ? 100 : 0)!;
      expect(geometry.pendingMeasurement).toBe(false);
      expect(geometry.icon?.x).toBe(caption ? 38 : 92);
      if (caption) {
        expect(geometry.text.x).toBe(62);
        expect(geometry.text.width).toBe(106);
        expect(nativeSemanticGlyphGeometry(document, add, 200, 32, 16)!.pendingMeasurement).toBe(true);
      }
      expect(document).toEqual(original);
    }
    document['penpot_i18n_tables'] = {};
    expect(() => nativeSemanticGlyphGeometry(document, add, 200, 32, 16, 100)).toThrow(/Unresolved localized text/);
  });

  it('creates the actual SVG and refreshes its geometry/tint, hiding stale unowned paint', () => {
    const { document, byControl } = fixture();
    const node = byControl('WorkbenchTransformLabel');
    const metadata: Record<string, string> = {};
    const icon = { type: 'group', x: 0, y: 0, width: 0, height: 0, hidden: false, opacity: 1,
      fills: [{ fillColor: '#123456' }], strokes: [{ strokeColor: '#654321' }], children: [],
      getSharedPluginData: (_namespace: string, key: string) => metadata[key] ?? '',
      setSharedPluginData: (_namespace: string, key: string, value: string) => { metadata[key] = value; },
      resize(width: number, height: number) { this.width = width; this.height = height; } };
    const children: Shape[] = [];
    const board = { x: 100, y: 200, width: 160, height: 24, children,
      appendChild: (child: Shape) => children.push(child) } as unknown as Board;
    const create = vi.fn(() => icon);
    vi.stubGlobal('penpot', { createShapeFromSvg: create });
    const original = structuredClone(document);
    createPrefabIcon(board, node, document);
    expect(create).toHaveBeenCalledWith(readFileSync(root + '/zircon_editor/assets/icons/zircon_editor_shell/inspector/transform.svg', 'utf8'));
    const first = [icon.x, icon.y, icon.width, icon.height];
    refreshNativeSemanticIcons(board, node, document);
    expect([icon.x, icon.y, icon.width, icon.height]).toEqual(first);
    expect(icon.fills[0].fillColor).toBe(previewIconTint(document, node));
    expect(metadata['role']).toBe('auxiliary');
    expect(icon.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'sourceNodeId')).toBe('');
    expect(document).toEqual(original);
    const wrong = { ...node, penpot_review_source_node_id: 'unowned' };
    refreshNativeSemanticIcons(board, wrong, document);
    expect(icon.hidden).toBe(true);
  });

  it.each([
    ['console', 'Console', 'ionicons/terminal-outline.svg', 3],
    ['runtime-diagnostics', 'Runtime Diagnostics', 'ionicons/grid-outline.svg', 4],
    ['build-export', 'Desktop Export', 'ionicons/share-outline.svg', 5],
  ] as const)('scales actual bottom %s SVG strokes once through delayed import, resize and reopen',
    (key, title, assetPath, count) => {
      const { document } = fixture();
      const header = 'zircon_editor/assets/ui/editor/unreal/components/dock_header.zui';
      const tabs = [
        { id: 'console', descriptorId: 'unit-console', title: 'Console', iconKey: 'console' },
        { id: 'runtime-diagnostics', descriptorId: 'unit-diagnostics', title: 'Runtime Diagnostics', iconKey: 'runtime-diagnostics' },
        { id: 'build-export', descriptorId: 'unit-export', title: 'Desktop Export', iconKey: 'build-export' },
      ];
      const index = tabs.findIndex((tab) => tab.id === key), tab = tabs[index]!;
      const lane = { width: 900, height: 31 };
      const node: ZuiNode = { component: 'Label', control_id: 'DockTab' + index,
        props: { text: title, selected: index === 0 },
        penpot_review_source_path: header, penpot_review_source_node_id: 'dock_tab_' + index,
        penpot_review_instance_path: '[]', penpot_review_runtime_surface_key: 'bottom',
        penpot_review_native_tab_icon: { authority: 'actual-workbench-bottom-tab', sourcePath: header,
          sourceNodeId: 'dock_tab_' + index, instancePath: '[]', surfaceKey: 'bottom', drawerSlot: 'bottom',
          tabId: tab.id, descriptorId: tab.descriptorId, iconKey: tab.iconKey, assetPath, title,
          selected: index === 0, sourceUnits: 'physical-pixels', sourceViewport: lane,
          ownerSha256: 'a'.repeat(64), stateFingerprint: 'b'.repeat(64) },
      };
      const presentation = { sourceFingerprint: { sourcePath: owner, sha256: 'a'.repeat(64) },
        stateFingerprint: 'b'.repeat(64), drawers: [{ slot: 'bottom', visible: true, activeTabId: 'console', tabs }],
        dockHeaders: [{ kind: 'bottom', surfaceKey: 'bottom', sourcePath: header, viewport: lane }] };
      const review = { host: 'editor', sourcePath: owner, dpi: 1.25,
        viewport: { width: 720, height: 620 }, data: { workbenchPresentation: presentation } };
      document['penpot_actual_normal_review_case'] = review;
      const svg = readFileSync(root + '/zircon_editor/assets/icons/' + assetPath, 'utf8');
      node['penpot_icon_svg'] = svg; // Actual consumed Header mounts retain this per-node SVG lane.
      expect(svg).toContain('viewBox="0 0 512 512"');
      expect([...svg.matchAll(/stroke-width:32px/g)]).toHaveLength(count);
      const values: Record<string, string> = {};
      const leaves = Array.from({ length: count }, (_, ordinal) => ({
        id: 'native-tab-leaf-' + ordinal, type: 'path', fills: [],
        strokes: [{ strokeColor: '#000000', strokeWidth: 32 }],
      }));
      const viewport = { id: 'native-tab-viewport', type: 'rectangle', fills: [], strokes: [] };
      const icon = { id: 'native-tab-icon', type: 'group', x: 0, y: 0, width: 0, height: 0,
        hidden: false, fills: [], strokes: [], children: [] as typeof leaves,
        getSharedPluginData: (_namespace: string, name: string) => values[name] ?? '',
        setSharedPluginData: (_namespace: string, name: string, value: string) => { values[name] = value; },
        resize(width: number, height: number) { this.width = width; this.height = height; } };
      const children: Shape[] = [];
      const board = { x: 10, y: 20, width: 120, height: 32, children,
        appendChild: (shape: Shape) => children.push(shape) } as unknown as Board;
      const create = vi.fn(() => icon);
      vi.stubGlobal('penpot', { createShapeFromSvg: create });
      const original = structuredClone(document);
      createPrefabIcon(board, node, document);
      expect(create).toHaveBeenCalledWith(svg);
      expect(icon.hidden).toBe(true);
      icon.children = leaves.slice(0, 1);
      refreshNativeSemanticIcons(board, node, document);
      expect(icon.hidden).toBe(true);
      expect(leaves.map((leaf) => leaf.strokes[0]!.strokeWidth)).toEqual(Array(count).fill(32));
      icon.children = [viewport, ...leaves] as typeof leaves;
      refreshNativeSemanticIcons(board, node, document);
      expect(icon.hidden).toBe(false);
      expect(icon.width).toBeCloseTo(15.6);
      for (const leaf of leaves) expect(leaf.strokes[0]!.strokeWidth).toBeCloseTo(0.975, 10);
      expect(leaves.every((leaf) => leaf.fills.length === 0)).toBe(true);
      expect(viewport.strokes).toEqual([]);
      expect(document).toEqual(original);

      for (const dpi of [1, 1.25, 1.5, 1.25]) {
        review.dpi = dpi;
        review.viewport.width = lane.width / dpi;
        for (const height of [40, 32, 32]) {
          board.height = height;
          refreshNativeSemanticIcons(board, node, document);
          const geometry = nativeSemanticGlyphGeometry(document, node, board.width, board.height, 0)!;
          expect(icon.width).toBeCloseTo(geometry.icon!.width);
          for (const leaf of leaves)
            expect(leaf.strokes[0]!.strokeWidth).toBeCloseTo(32 * geometry.icon!.width / 512, 10);
        }
      }
      // Saved plugin metadata, rather than proxy identity/WeakMaps, owns the baseline.
      const reopened = { ...icon, children: icon.children.map((shape) => ({ ...shape,
        strokes: shape.strokes.map((stroke) => ({ ...stroke })) })) };
      children[0] = reopened as unknown as Shape;
      refreshNativeSemanticIcons(board, node, document);
      expect(reopened.children[1]!.strokes[0]!.strokeWidth).toBeCloseTo(0.975);
      const firstWidth = reopened.children[1]!.strokes[0]!.strokeWidth;
      reopened.children.at(-1)!.strokes[0]!.strokeWidth = 32;
      expect(() => refreshNativeSemanticIcons(board, node, document)).toThrow(/stroke changed/);
      expect(reopened.hidden).toBe(true);
      expect(reopened.children[1]!.strokes[0]!.strokeWidth).toBe(firstWidth);
      expect(create).toHaveBeenCalledTimes(1);
    });
});

describe('authored footer semantic status signals', () => {
  const root = process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../', import.meta.url));
  const status = 'zircon_editor/assets/ui/editor/unreal/components/status_bar.zui';
  const owner = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
  const shell = 'zircon_editor/assets/ui/editor/unreal/editor_shell.zui';
  const dependencies = new LayoutDependencies();
  let source: ZuiDocument;
  beforeAll(async () => {
    await dependencies.load(root, await trackedZuiPaths());
    source = parseZuiDocument(readFileSync(root + '/' + status, 'utf8')).document;
    dependencies.embed(source, status, 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui');
  }, 120_000);
  afterEach(() => vi.unstubAllGlobals());
  const fixture = (id = 'messages') => {
    const document = cloneZuiDocument(source);
    document['penpot_host_palette'] = { ...document['penpot_host_palette'] as object,
      success: '#12ab34', warning: '#bc5600', info: '#23aaff', text_disabled: '#808080' };
    return { document, node: document.nodes![id] };
  };

  it.each([
    ['ready', 'success', '#12ab34'], ['errors', 'success', '#12ab34'],
    ['warnings', 'warning', '#bc5600'], ['messages', 'info', '#23aaff'],
  ])('admits the authored %s signal and consumes semantic palette/state rather than caption or declared ink',
    (id, signal, color) => {
      const { document, node } = fixture(id);
      const original = structuredClone(document);
      expect(nativeSemanticGlyph(document, node)).toMatchObject({ family: 'status-signal', signal, assetPath: null });
      expect(previewIconTint(document, node)).toBe(color);
      expect(document).toEqual(original);
      node.props = { ...node.props, text: '999 arbitrary text', foreground_color: '#ff00ff' };
      expect(previewIconTint(document, node)).toBe(color);
      node.state = { disabled: true, selected: true, hovered: true };
      expect(previewIconTint(document, node)).toBe('#808080');
      node.state = { loading: true };
      expect(previewIconTint(document, node)).toBe('#808080');
    });

  it('requires the exact footer Source, component, semantic variant and complete supported invocation', () => {
    const { document, node } = fixture();
    for (const steps of [[], [{ sourcePath: owner, sourceNodeId: 'status_bar' }],
      [{ sourcePath: shell, sourceNodeId: 'pane_surface' }, { sourcePath: owner, sourceNodeId: 'status_bar' }]]) {
      node['penpot_review_instance_path'] = JSON.stringify(steps);
      expect(nativeSemanticGlyph(document, node)?.family).toBe('status-signal');
    }
    for (const mutate of [
      (value: ZuiNode) => { value.component = 'Button'; },
      (value: ZuiNode) => { value.control_id = 'OtherStatusMessages'; },
      (value: ZuiNode) => { value['penpot_review_source_node_id'] = 'other'; },
      (value: ZuiNode) => { value['penpot_review_source_path'] = 'other/' + status; },
      (value: ZuiNode) => { value.props = { ...value.props, component_variant: 'other' }; },
      (value: ZuiNode) => { value['penpot_review_instance_path'] = JSON.stringify([
        { sourcePath: owner, sourceNodeId: 'other_status_bar' }]); },
      (value: ZuiNode) => { value['penpot_review_instance_path'] = JSON.stringify([
        { sourcePath: shell, sourceNodeId: 'other' }, { sourcePath: owner, sourceNodeId: 'status_bar' }]); },
    ]) {
      const changed = structuredClone(node);
      mutate(changed);
      expect(nativeSemanticGlyph(document, changed)).toBeNull();
    }
  });

  it('creates an auxiliary ellipse and refreshes marker/text lanes from host metrics without changing Source', () => {
    const { document, node } = fixture();
    const original = structuredClone(document), values: Record<string, string> = {};
    const icon = { id: 'status-circle', type: 'ellipse', x: 0, y: 0, width: 0, height: 0, hidden: false,
      fills: [] as Array<{ fillColor: string }>, strokes: [], layoutChild: { absolute: false },
      getSharedPluginData: (_namespace: string, key: string) => values[key] ?? '',
      setSharedPluginData: (_namespace: string, key: string, value: string) => { values[key] = value; },
      resize(width: number, height: number) { this.width = width; this.height = height; } };
    const children: Shape[] = [];
    const board = { x: 100, y: 200, width: 100, height: 32, children,
      appendChild: (shape: Shape) => children.push(shape) } as unknown as Board;
    const create = vi.fn(() => icon), svg = vi.fn();
    vi.stubGlobal('penpot', { createEllipse: create, createShapeFromSvg: svg });
    createPrefabIcon(board, node, document);
    expect(create).toHaveBeenCalledTimes(1);
    expect(svg).not.toHaveBeenCalled();
    expect([icon.x, icon.y, icon.width, icon.height, icon.hidden]).toEqual([108, 212, 8, 8, false]);
    expect(icon.fills).toEqual([{ fillColor: '#23aaff' }]);
    expect(icon.strokes).toEqual([]);
    expect(icon.layoutChild.absolute).toBe(true);
    expect(values['role']).toBe('auxiliary');
    expect(values['sourceNodeId']).toBeUndefined();
    const text = { fontSize: 10.666667, lineHeight: '1.2', align: 'left', x: 0, y: 0, hidden: false,
      resize: vi.fn(), verticalAlign: 'center' } as unknown as Text;
    expect(layoutNativeSemanticText(board, text, node, document)).toBe(true);
    expect(text.x).toBe(124);
    expect(text.y).toBeCloseTo(209.6);
    expect(text.resize).toHaveBeenLastCalledWith(76, 10.666667 * 1.2);
    board.height = 40;
    refreshNativeSemanticIcons(board, node, document);
    expect(icon.y).toBe(216);
    expect(document).toEqual(original);
    const reopened = { ...icon };
    children[0] = reopened as unknown as Shape;
    refreshNativeSemanticIcons(board, node, document);
    expect([reopened.x, reopened.y, reopened.width, reopened.height]).toEqual([108, 216, 8, 8]);
    refreshNativeSemanticIcons(board, { ...node, penpot_review_source_node_id: 'unowned' }, document);
    expect(reopened.hidden).toBe(true);
  });

  it('uses the shared logical gap once, honors authored offsets/gap and suppresses an undersized signal lane', () => {
    const { document, node } = fixture();
    document.tokens!['editor.density.gap.medium'] = 10;
    document['penpot_actual_normal_review_case'] = { dpi: 1.25 };
    node.props = { ...node.props, layout_offset_x: 1, layout_offset_y: 2, layout_content_offset_x: 4 };
    expect(nativeSemanticGlyphGeometry(document, node, 100, 40, 12)).toMatchObject({
      icon: { x: 11, y: 17, width: 10, height: 10 }, text: { x: 25, y: 16, width: 75, height: 12 } });
    const board = { x: 0, y: 0, width: 15, height: 40 } as Board;
    const text = { fontSize: 10, lineHeight: '1.2', align: 'left', resize: vi.fn(), hidden: false } as unknown as Text;
    expect(nativeSemanticGlyphGeometry(document, node, board.width, board.height, 12)!.icon).toBeNull();
    layoutNativeSemanticText(board, text, node, document);
    expect(text.hidden).toBe(true);
    board.width = 100;
    layoutNativeSemanticText(board, text, node, document);
    expect(text.hidden).toBe(false);
  });
});
