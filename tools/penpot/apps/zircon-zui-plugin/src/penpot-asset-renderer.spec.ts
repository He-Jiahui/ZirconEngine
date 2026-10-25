import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import type { Board, Shape, Text } from '@penpot/plugin-types';
import { LayoutDependencies } from '../tools/zui-layout-dependencies';
import {
  auditRenderedAssetBoard,
  createAssetBoard,
  refreshAssetTextLayout,
  scrollAxisForNode,
} from './penpot-asset-renderer';
import { ZUI_METADATA_NAMESPACE } from './metadata';
import { projectZuiDocument } from './bridge/penpot-projection';
import { parseZuiDocument, type ZuiDocument } from './bridge/zui-document';
import { capturedHeightMin } from './penpot-capture-validation';
import { captureNodeTexts } from './penpot-text-capture';
import { authoredLabelTextInsets, createSemanticTexts } from './penpot-semantic-text';
import { assertExportShapeGuard, writeExportShapeGuard } from './penpot-export-guard';
import { linearContentMeasurementSnapshot, measuredLinearDesiredSizes } from './penpot-linear-content';
import { applyWeightedFlexLayout } from './penpot-weighted-flex';

function measuredBoard(id: string, metadata: Record<string, string>): Board {
  return {
    id,
    name: id,
    type: 'board',
    x: 0,
    y: 0,
    width: 100,
    height: 100,
    hidden: false,
    clipContent: false,
    children: [],
    getSharedPluginData: (namespace: string, key: string) =>
      namespace === ZUI_METADATA_NAMESPACE ? (metadata[key] ?? '') : '',
    setSharedPluginData: (namespace: string, key: string, value: string) => {
      if (namespace === ZUI_METADATA_NAMESPACE) metadata[key] = value;
    },
    resize(width: number, height: number) {
      this.width = width;
      this.height = height;
    },
  } as unknown as Board;
}

describe('source-owned Button caption width inheritance', () => {
  afterEach(() => vi.unstubAllGlobals());

  it.each([592, 852, 1232])('creates a centered native caption lane at width %i without replacing the source identity', (width) => {
    const board = measuredBoard('canonical-button', { role: 'node', 'node-id': 'button' });
    board.resize(width, 32);
    const text = Object.assign(measuredBoard('caption', {}), {
      type: 'text', characters: '添加组件', fills: [], strokes: [],
    }) as unknown as Text;
    Object.assign(board, { appendChild(child: Shape) {
      (board.children as Shape[]).push(child); Object.assign(child, { parent: board });
    } });
    vi.stubGlobal('penpot', {
      createText: () => text,
      fonts: { all: [{ id: 'firasans', fontFamily: 'Fira Sans', variants: [{ id: 'regular',
        fontWeight: '400', fontStyle: 'normal' }], applyToText(target: Text) {
        Object.assign(target, { fontId: 'firasans', fontFamily: 'Fira Sans',
          fontVariantId: 'regular', fontWeight: '400', fontStyle: 'normal' });
      } }] },
    });
    const node = { component: 'Button', props: { text: '添加组件' } };
    const document: ZuiDocument = { asset: { kind: 'view', id: 'button-width-inheritance', version: 2 },
      root: { node: 'button' }, nodes: { button: node } };
    const projection = { nodeId: 'button', prefabRole: 'button',
      text: { characters: '添加组件', property: 'text', align: 'center',
        fontFamily: 'Fira Sans', fontWeight: '400', fontSize: 14, lineHeight: 1.2 } } as
      Parameters<typeof createSemanticTexts>[1];
    createSemanticTexts(board, projection, document, node);
    expect(text.constraintsHorizontal).toBe('leftright');
    expect(text.align).toBe('center');
    expect(text.x - board.x + text.width / 2).toBeCloseTo(width / 2);
    expect(text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'node-id')).toBe('button');
    expect(text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-property')).toBe('text');
    expect([board.id, text.id]).toEqual(['canonical-button', 'caption']);
  });
});

describe('initial layoutChild constraint import', () => {
  it('writes hidden child and detached lane minima through the actual SDK parent capability', async () => {
    const created: Board[] = [];
    const createBoard = () => {
      const board = measuredBoard(`board-${created.length}`, {});
      const layoutChild: { minHeight?: number } = {};
      Object.assign(board, {
        parent: null,
        appendChild(child: Board) {
          const previous = child.parent;
          if (previous?.type === 'board') {
            const siblings = previous.children as Board[];
            const index = siblings.indexOf(child);
            if (index >= 0) siblings.splice(index, 1);
          }
          (board.children as Board[]).push(child);
          Object.assign(child, { parent: board });
        },
        addFlexLayout() {
          const flex = {};
          Object.assign(board, { flex });
          return flex;
        },
        remove() {},
      });
      // Match the official getter: the board's own layout does not confer
      // layoutChild capability; only its immediate flex/grid parent does.
      Object.defineProperty(board, 'layoutChild', {
        get: () => board.parent?.type === 'board' &&
          (board.parent.flex || board.parent.grid) ? layoutChild : undefined,
      });
      created.push(board);
      return board;
    };
    const createText = vi.fn(() => { throw new Error('Hidden content must not be created'); });
    vi.stubGlobal('penpot', {
      createBoard,
      createText,
      viewport: { center: { x: 0, y: 0 } },
    });
    try {
      const document: ZuiDocument = {
        asset: { kind: 'view', id: 'height-minimum-import', version: 2 },
        root: { node: 'host' },
        nodes: {
          host: {
            component: 'UiHostWindow',
            layout: {
              container: { kind: 'VerticalBox' },
              width: { preferred: 1280, stretch: 'Fixed' },
              height: { min: 800, preferred: 800, stretch: 'Fixed' },
            },
            children: [{ node: 'project_context' }, { node: 'dock' }],
          },
          project_context: {
            component: 'HorizontalGroup',
            props: { visibility: 'collapsed', visible: false, text: 'Never materialized' },
            layout: {
              container: { kind: 'HorizontalBox' },
              width: { stretch: 'Stretch' },
              height: { min: 32, preferred: 32, max: 32, stretch: 'Fixed' },
            },
            children: [{ node: 'hidden_content' }],
          },
          hidden_content: { component: 'Label', props: { text: 'Not imported' } },
          dock: {
            component: 'DockHost',
            layout: { container: { kind: 'Overlay' } },
            children: [{ node: 'free_child' }, { node: 'floating' }],
          },
          free_child: {
            component: 'HorizontalGroup',
            props: { visibility: 'collapsed', visible: false },
            layout: { height: { min: 14 } },
          },
          floating: {
            component: 'FloatingWindow',
            props: { visibility: 'collapsed', visible: false },
            layout: { height: { min: 28, preferred: 28, stretch: 'Fixed' } },
          },
          template: {
            component: 'HorizontalGroup',
            props: { visibility: 'collapsed', visible: false },
            layout: { height: { min: 24, preferred: 24, stretch: 'Fixed' } },
          },
        },
      };
      const original = structuredClone(document);
      const projection = projectZuiDocument(document);
      const asset = await createAssetBoard('minimum.zui', document, projection);
      const byNodeId = (nodeId: string) => created.find((board) =>
        board.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'node-id') === nodeId)!;
      const host = byNodeId('host');
      const hidden = byNodeId('project_context');
      const detached = byNodeId('template');
      const freeChild = byNodeId('free_child');
      const floating = byNodeId('floating');
      expect(host.parent).toBe(asset);
      expect(host.flex).toBeDefined();
      expect(host.layoutChild).toBeUndefined();
      expect(capturedHeightMin('host', 800, false, host.layoutChild)).toBeUndefined();
      expect(hidden.parent).toBe(host);
      expect(hidden.hidden).toBe(true);
      expect(hidden.layoutChild).toMatchObject({ minHeight: 32, maxHeight: 32, verticalSizing: 'fix' });
      expect(capturedHeightMin('project_context', 32, true, hidden.layoutChild)).toBe(32);
      expect(hidden.flex).toBeUndefined();
      expect(hidden.children).toHaveLength(0);
      expect(created.some((board) =>
        board.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'node-id') === 'hidden_content'))
        .toBe(false);
      expect(detached.parent?.type).toBe('board');
      expect((detached.parent as Board).flex).toBeDefined();
      expect(detached.layoutChild?.minHeight).toBe(24);
      expect(projection.detachedNodes.find(({ nodeId }) => nodeId === 'template')!.heightMin).toBe(24);
      expect(freeChild.parent).toBe(byNodeId('dock'));
      expect(freeChild.layoutChild).toBeUndefined();
      expect(projection.shapes.find(({ nodeId }) => nodeId === 'free_child')!)
        .not.toHaveProperty('heightMin');
      expect(capturedHeightMin('free_child', 14, false, freeChild.layoutChild)).toBeUndefined();
      expect(floating.parent).toBe(detached.parent);
      expect(floating.parent).not.toBe(byNodeId('dock'));
      expect(floating.layoutChild?.minHeight).toBe(28);
      expect(projection.shapes.find(({ nodeId }) => nodeId === 'floating'))
        .toMatchObject({ visualDetached: true, visualDetachedParentId: 'dock', heightMin: 28 });
      expect(createText).not.toHaveBeenCalled();
      expect(document).toEqual(original);
    } finally {
      vi.unstubAllGlobals();
    }
  });
});

describe('initial TreeRow layout refresh', () => {
  function initialRow(visibility: 'collapsed' | 'visible') {
    const asset = measuredBoard('asset', { role: 'asset', 'node-id': 'root' });
    const row = measuredBoard('row', { role: 'node', 'node-id': 'row' });
    Object.assign(asset, { children: [row] });
    Object.assign(row, { parent: asset, hidden: true });
    const document: ZuiDocument = {
      asset: { kind: 'view', id: 'initial-tree-row', version: 2 },
      root: { node: 'root' },
      penpot_host_theme_source: 'editor_tokens.zui',
      nodes: {
        root: { component: 'Overlay', children: [{ node: 'row' }] },
        row: { component: 'TreeRow', props: { text: '', visibility } },
      },
    };
    const projection = projectZuiDocument(document);
    return { row, refresh: () => refreshAssetTextLayout(asset, document, projection) };
  }

  it('keeps collapsed source rows unpainted during initial reflow', () => {
    const { row, refresh } = initialRow('collapsed');
    expect(refresh).not.toThrow();
    expect(row.children).toHaveLength(0);
    expect(row.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'tree-row-authoring-painter')).toBe('');
  });

  it('rejects missing painter metadata for visible source rows even if the board is hidden', () => {
    const { refresh } = initialRow('visible');
    expect(refresh).toThrow('TreeRow has no original painter metadata; reimport it.');
  });
});

describe('Penpot rendered source identity', () => {
  it('preserves owner and parent tuples for repeated prefab instances', () => {
    const asset = measuredBoard('asset', { role: 'asset' });
    const parent = measuredBoard('parent', {
      role: 'node',
      'node-id': 'host',
      sourcePath: 'host.zui',
      sourceNodeId: 'host_root',
      instancePath: '[]',
    });
    const instances = ['first', 'second'].map((callSite) => {
      const instancePath = JSON.stringify([
        { sourcePath: 'host.zui', sourceNodeId: callSite },
      ]);
      return measuredBoard(callSite, {
        role: 'node',
        'node-id': `${callSite}/button`,
        sourcePath: 'button.zui',
        sourceNodeId: 'button',
        controlId: 'SharedButton',
        instancePath,
      });
    });
    Object.assign(asset, { children: [parent] });
    Object.assign(parent, { parent: asset, children: instances });
    for (const instance of instances) Object.assign(instance, { parent });
    const measured = auditRenderedAssetBoard(asset).semanticNodes!;
    for (const [index, callSite] of ['first', 'second'].entries()) {
      expect(measured[index + 1]).toMatchObject({
        sourcePath: 'button.zui',
        sourceNodeId: 'button',
        controlId: 'SharedButton',
        instancePath: JSON.stringify([
          { sourcePath: 'host.zui', sourceNodeId: callSite },
        ]),
        parentSourcePath: 'host.zui',
        parentSourceNodeId: 'host_root',
        parentInstancePath: '[]',
      });
    }
  });

  it('leaves missing owner metadata incomplete for strict acceptance', () => {
    const asset = measuredBoard('asset', { role: 'asset' });
    const child = measuredBoard('child', { role: 'node', 'node-id': 'child' });
    Object.assign(asset, { children: [child] });
    Object.assign(child, { parent: asset });
    expect(auditRenderedAssetBoard(asset).semanticNodes![0]).toMatchObject({
      sourcePath: null,
      sourceNodeId: null,
      controlId: null,
      instancePath: '',
      parentSourcePath: null,
      parentSourceNodeId: null,
      parentInstancePath: '',
    });
  });

  it('reads Inspector item keys and field IDs separately from the persisted authored and parent tuples', () => {
    const asset = measuredBoard('asset', { role: 'asset' });
    const owner = measuredBoard('owner', {
      role: 'node', 'node-id': 'owner', sourcePath: 'details.zui',
      sourceNodeId: 'properties', instancePath: '[]',
    });
    const instancePath = JSON.stringify([{ sourcePath: 'details.zui', sourceNodeId: 'prototype' }]);
    const fields = [
      ['zircon_runtime::scene::components::CameraComponent.fov_y_radians',
        '157694992433500694610706526866931910355'],
      ['zircon_runtime::scene::components::CameraComponent.z_far',
        '57620256968940470099537706358547345741'],
    ];
    const rows = fields.map(([propertyFieldId, itemKey], index) => measuredBoard(`row-${index}`, {
      role: 'node', 'node-id': `row-${index}`, component: 'InputField',
      sourcePath: 'component_property_row.zui', sourceNodeId: 'root', instancePath,
      itemKey, propertyFieldId,
    }));
    Object.assign(asset, { children: [owner] });
    Object.assign(owner, { parent: asset, children: rows });
    for (const row of rows) Object.assign(row, { parent: owner });
    const measured = auditRenderedAssetBoard(asset).semanticNodes!;
    expect(measured[0].itemKey).toBeUndefined();
    expect(measured[0].propertyFieldId).toBeUndefined();
    for (const [index, [propertyFieldId, itemKey]] of fields.entries())
      expect(measured[index + 1]).toMatchObject({
        sourcePath: 'component_property_row.zui', sourceNodeId: 'root', instancePath,
        parentSourcePath: 'details.zui', parentSourceNodeId: 'properties', parentInstancePath: '[]',
        itemKey, propertyFieldId,
      });
    rows[0].setSharedPluginData(ZUI_METADATA_NAMESPACE, 'propertyFieldId', '');
    const incomplete = auditRenderedAssetBoard(asset).semanticNodes![1];
    expect(incomplete.itemKey).toBe(fields[0][1]);
    expect(incomplete.propertyFieldId).toBeUndefined();
    expect(incomplete.sourceNodeId).toBe('root');
    expect(incomplete.instancePath).toBe(instancePath);
  });
});

describe('Penpot asset scroll semantics', () => {
  it('checks each nested viewport against its own scroll axes', () => {
    const asset = measuredBoard('asset', { role: 'asset' });
    const horizontalViewport = measuredBoard('outer', {
      role: 'node',
      'node-id': 'outer',
      'container-kind': 'scroll',
      'scroll-axis': 'horizontal',
    });
    const verticalViewport = measuredBoard('inner', {
      role: 'node',
      'node-id': 'inner',
      'container-kind': 'scroll',
      'scroll-axis': 'vertical',
    });
    const content = measuredBoard('content', {
      role: 'node',
      'node-id': 'content',
    });
    Object.assign(asset, { children: [horizontalViewport] });
    Object.assign(horizontalViewport, {
      parent: asset,
      children: [verticalViewport],
    });
    Object.assign(verticalViewport, {
      parent: horizontalViewport,
      children: [content],
    });
    Object.assign(content, {
      parent: verticalViewport,
      width: 120,
      height: 300,
    });

    expect(auditRenderedAssetBoard(asset).overflowNodeIds).toEqual(['content']);
  });

  it('preserves a ScrollBox axis when its child layout uses a VerticalBox', () => {
    expect(
      scrollAxisForNode({
        component: 'ScrollBox',
        props: {
          scroll_axis: 'vertical',
          scroll_x: false,
          scroll_y: true,
        },
        layout: { container: { kind: 'VerticalBox', gap: 10 } },
      }),
    ).toBe('vertical');
  });

  it('combines explicit horizontal and vertical scroll flags', () => {
    expect(
      scrollAxisForNode({
        component: 'ScrollableBox',
        props: { scroll_x: true, scroll_y: true },
        layout: { container: { kind: 'HorizontalBox' } },
      }),
    ).toBe('both');
  });

  it('keeps non-scroll layout containers out of scroll review', () => {
    expect(
      scrollAxisForNode({
        component: 'VerticalBox',
        layout: { container: { kind: 'VerticalBox' } },
      }),
    ).toBeNull();
  });
});

describe('actual missing semantic icons renderer integration', () => {
  const repo = process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../', import.meta.url));
  const theme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
  const starship = 'zircon_editor/assets/ui/editor/unreal/starship.zui';
  const details = 'zircon_editor/assets/ui/editor/unreal/components/details.zui';
  const button = 'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/button.zui';
  const dependencies = new LayoutDependencies();
  beforeAll(async () => dependencies.load(repo, [theme, starship]));
  afterEach(() => vi.unstubAllGlobals());

  it.each([
    ['add', 'WorkbenchAddComponent', 'zircon_editor_shell/controls/add.svg', 16, 59],
    ['add-localized', 'WorkbenchAddComponent', 'zircon_editor_shell/controls/add.svg', 16, 59],
    ['add-localized-zh', 'WorkbenchAddComponent', 'zircon_editor_shell/controls/add.svg', 16, 59],
    ['transform_title', 'WorkbenchTransformLabel', 'zircon_editor_shell/inspector/transform.svg', 13.333333, 29.333333],
    ['component_title', 'WorkbenchMeshLabel', 'zircon_editor_shell/inspector/mesh-renderer.svg', 13.333333, 29.333333],
  ] as const)('creates and refreshes %s with its actual SVG and one editable text lane', async (id, control, assetPath, iconSize, textLeft) => {
    const isAdd = control === 'WorkbenchAddComponent', sourcePath = isAdd ? button : details, nodeId = isAdd ? 'root' : id;
    const localized = id.startsWith('add-localized');
    const caption = id.endsWith('-zh') ? '添加组件' : isAdd ? 'Add Component' : id === 'component_title' ? 'Camera' : 'Transform';
    const authored = parseZuiDocument(readFileSync(repo + '/' + sourcePath, 'utf8')).document;
    const node = structuredClone(authored.nodes![nodeId]);
    node.control_id = control;
    node.props = { ...node.props, text: localized ? { text_key: 'editor.inspector.component_add.open', table: 'editor' } : caption,
      visibility: 'visible' };
    node.layout = { width: { min: 200, preferred: 200, max: 200, stretch: 'Fixed' },
      height: { min: 32, preferred: 32, max: 32, stretch: 'Fixed' } };
    const document: ZuiDocument = { asset: { id: 'unit-semantic-glyph-' + id, kind: 'view', version: 2 },
      // Retain the source's actual style closure: Details binds its paint to
      // Starship tokens, which are not part of the base editor token document.
      imports: { styles: structuredClone(authored.imports?.['styles'] ?? []) },
      root: { node: nodeId }, nodes: { [nodeId]: node } };
    if (localized) document['penpot_i18n_tables'] = { editor: { translations: {
      'editor.inspector.component_add.open': caption,
    } } };
    dependencies.embed(document, sourcePath, theme);
    const currentNode = document.nodes![nodeId];
    if (isAdd) currentNode['penpot_review_instance_path'] = JSON.stringify([{ sourcePath: details, sourceNodeId: 'add' }]);
    const original = structuredClone(document);
    let sequence = 0;
    const shape = (type: string) => {
      const value = measuredBoard('mock-' + sequence++, {});
      Object.assign(value, { type, opacity: 1, fills: [], strokes: [],
        appendChild(child: Shape) {
          (value.children as Shape[]).push(child);
          Object.assign(child, { parent: value });
        },
        remove() {},
      });
      return value;
    };
    const createText = (characters: string) => {
      const text = shape('text');
      const normalFontSize = projection.shapes.find((shape) => shape.nodeId === nodeId)!.text!.fontSize!;
      let fixedBounds = { width: 100, height: 16 };
      Object.assign(text, { characters, width: 100, height: 16, fontSize: '13.333333', lineHeight: '1.2',
        fontStyle: 'normal', letterSpacing: '0', textTransform: 'none', direction: 'ltr', textDecoration: 'none',
        getRange: () => ({}),
        resize(width: number, height: number) { fixedBounds = { width, height }; },
      });
      // The SDK measures auto-width text at its current font size; fixed text
      // keeps the explicit resize bounds. 100 is this fixture's normal advance.
      const semanticText = text as unknown as Text;
      Object.defineProperties(text, {
        width: { configurable: true, get: () => semanticText.growType === 'auto-width'
          ? Math.ceil(100 * (Number(semanticText.fontSize) / normalFontSize)) : fixedBounds.width },
        height: { configurable: true, get: () => semanticText.growType === 'auto-width'
          ? Math.ceil(Number(semanticText.fontSize) * Number(semanticText.lineHeight)) : fixedBounds.height },
      });
      return text;
    };
    const fromSvg = vi.fn(() => shape('group'));
    vi.stubGlobal('penpot', {
      createBoard: () => shape('board'), createText, createShapeFromSvg: fromSvg,
      viewport: { center: { x: 0, y: 0 } }, fonts: { all: [{ fontFamily: 'Roboto',
        variants: ['400', '700'].map((fontWeight) => ({ fontWeight, fontStyle: 'normal', fontVariantId: fontWeight })),
        applyToText: (text: Text, variant: { fontWeight: string }) => {
          Object.assign(text, { fontFamily: 'Roboto', fontWeight: variant.fontWeight });
        }, applyToRange: vi.fn(),
      }] },
    });
    const projection = projectZuiDocument(document);
    const asset = await createAssetBoard('unit-' + id + '.zui', document, projection);
    const board = asset.children.find((child) => child.type === 'board' &&
      child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'node-id') === nodeId) as Board;
    const icon = board.children.find((child) => child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'native-semantic-glyph'))!;
    const texts = board.children.filter((child) => child.type === 'text' &&
      child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'role') === 'text') as Text[];
    expect(fromSvg).toHaveBeenCalledWith(readFileSync(repo + '/zircon_editor/assets/icons/' + assetPath, 'utf8'));
    expect(texts).toHaveLength(1);
    expect(texts[0].characters).toBe(caption);
    const styleGuard = texts[0].getSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-style-guard');
    if (isAdd) {
      expect(icon.hidden).toBe(true); // Actual font probe has not been consumed yet.
      const probe = asset.children.find((child) => child.type === 'text' &&
        child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'linear-content-measurement') === nodeId) as Text;
      expect(probe.width).toBe(1600);
      expect(linearContentMeasurementSnapshot(asset).find((item) => item.nodeId === nodeId)!.width).toBe(100);
    }
    refreshAssetTextLayout(asset, document, projection);
    expect(icon.hidden).toBe(false);
    expect(icon.width).toBeCloseTo(iconSize);
    expect(texts[0].x - board.x).toBeCloseTo(textLeft);
    if (isAdd) {
      expect(texts[0].width).toBe(106);
      expect(texts[0].x - board.x + (texts[0].width - 100) / 2).toBeCloseTo(62);
    }
    expect(captureNodeTexts(board).text?.align).toBe(projection.shapes.find((shape) => shape.nodeId === nodeId)!.text!.align);
    expect(board.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'has-leading-icon')).toBe('');
    const first = [icon.x, icon.y, icon.width, icon.height, texts[0].x, texts[0].y, texts[0].width];
    refreshAssetTextLayout(asset, document, projection);
    expect([icon.x, icon.y, icon.width, icon.height, texts[0].x, texts[0].y, texts[0].width]).toEqual(first);
    expect(texts[0].getSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-style-guard')).toBe(styleGuard);
    texts[0].characters = 'Still editable';
    refreshAssetTextLayout(asset, document, projection);
    expect(texts[0].characters).toBe('Still editable');
    expect(auditRenderedAssetBoard(asset).semanticNodes).toHaveLength(1);
    expect(document).toEqual(original);
    if (isAdd) {
      texts[0].characters = caption;
      refreshAssetTextLayout(asset, document, projection);
      // A persisted board from the old translation measurement has a centered
      // icon and a six-DIP caption lane. Reflow must keep the immutable guard
      // while validating the new positions against the real owning source.
      icon.x = board.x + (board.width - icon.width) / 2;
      texts[0].resize(6, 16);
      texts[0].x = board.x + (board.width + 16) / 2;
      icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'native-semantic-layout-geometry', '');
      texts[0].setSharedPluginData(ZUI_METADATA_NAMESPACE, 'native-semantic-layout-geometry', '');
      writeExportShapeGuard(asset);
      const guard = asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'export-shapes-guard');
      refreshAssetTextLayout(asset, document, projection);
      expect(() => assertExportShapeGuard(asset)).not.toThrow();
      expect(asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'export-shapes-guard')).toBe(guard);
      // Also reopen an already-reflowed old file that predates layout records.
      icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'native-semantic-layout-geometry', '');
      texts[0].setSharedPluginData(ZUI_METADATA_NAMESPACE, 'native-semantic-layout-geometry', '');
      expect(() => assertExportShapeGuard(asset)).not.toThrow();
      refreshAssetTextLayout(asset, document, projection);
      const correctX = icon.x, correctTextX = texts[0].x;
      icon.x += 0.05;
      expect(() => assertExportShapeGuard(asset)).toThrow(/changed geometry/);
      expect(() => refreshAssetTextLayout(asset, document, projection)).toThrow(/changed geometry/);
      icon.x = correctX;
      texts[0].x += 0.05;
      expect(() => assertExportShapeGuard(asset)).toThrow(/changed geometry/);
      expect(() => refreshAssetTextLayout(asset, document, projection)).toThrow(/changed geometry/);
      texts[0].x = correctTextX;
      const sourceIdentity = board.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'sourceNodeId');
      board.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'sourceNodeId', 'unowned');
      expect(() => assertExportShapeGuard(asset)).toThrow(/changed geometry/);
      board.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'sourceNodeId', sourceIdentity);
      icon.fills = [{ fillColor: '#ff0000' }];
      expect(() => assertExportShapeGuard(asset)).toThrow(/changed fills/);
    }
  });
});

describe('authored Label text padding at the renderer boundary', () => {
  afterEach(() => vi.unstubAllGlobals());

  function rendererMock(contentWidth: (value: string) => number = () => 100, normalFontSize = 13.333333) {
    let sequence = 0;
    const shape = (type: string) => {
      const value = measuredBoard('label-mock-' + sequence++, {});
      Object.assign(value, {
        type, opacity: 1, fills: [], strokes: [], parent: null,
        layoutChild: { absolute: false, horizontalSizing: 'fix', verticalSizing: 'fix' },
        appendChild(child: Shape) {
          (value.children as Shape[]).push(child);
          Object.assign(child, { parent: value });
        },
        addFlexLayout() {
          const flex = {};
          Object.assign(value, { flex });
          return flex;
        },
        remove() {},
      });
      return value;
    };
    vi.stubGlobal('penpot', {
      createBoard: () => shape('board'),
      createText(characters: string) {
        const text = shape('text');
        let fixedBounds = { width: 100, height: 16 };
        Object.assign(text, { characters, width: 100, height: 16, fontSize: '13.333333', lineHeight: '1.2',
          fontStyle: 'normal', letterSpacing: '0', textTransform: 'none', direction: 'ltr', textDecoration: 'none',
          getRange: () => ({}),
          resize: vi.fn((width: number, height: number) => { fixedBounds = { width, height }; }),
        });
        const semanticText = text as unknown as Text;
        Object.defineProperties(text, {
          width: { configurable: true, get: () => semanticText.growType === 'auto-width'
            ? Math.ceil(contentWidth(semanticText.characters) * Number(semanticText.fontSize) / normalFontSize)
            : fixedBounds.width },
          height: { configurable: true, get: () => semanticText.growType === 'auto-width'
            ? Math.ceil(Number(semanticText.fontSize) * Number(semanticText.lineHeight)) : fixedBounds.height },
        });
        return text;
      },
      viewport: { center: { x: 0, y: 0 } },
      fonts: { all: [{ fontFamily: 'Roboto', variants: [{ fontWeight: '400', fontStyle: 'normal', fontVariantId: '400' }],
        applyToText: (text: Text, variant: { fontWeight: string }) => {
          Object.assign(text, { fontFamily: 'Roboto', fontWeight: variant.fontWeight });
        },
        applyToRange: vi.fn(),
      }] },
    });
  }

  function fixture(props: Record<string, string | number | boolean> = {}) {
    const source = 'res://ui/editor/unreal/components/details.zui';
    const invocation = JSON.stringify([{ sourcePath: 'res://ui/editor/unreal/level_editor_window.zui', sourceNodeId: 'details' }]);
    const document: ZuiDocument = {
      asset: { kind: 'view', id: 'authored-label-column', version: 2 },
      root: { node: 'row' },
      tokens: { 'label.name-inset': 20 },
      nodes: {
        row: { component: 'HorizontalGroup',
          penpot_review_source_path: source, penpot_review_source_node_id: 'position', penpot_review_instance_path: invocation,
          children: [{ node: 'label' }, { node: 'axis' }],
          layout: { container: { kind: 'HorizontalBox', gap: 4 },
            width: { min: 160, preferred: 160, max: 160, stretch: 'Fixed' },
            height: { min: 28, preferred: 28, max: 28, stretch: 'Fixed' } },
        },
        label: { component: 'Label', control_id: 'WorkbenchTransformPositionLabel',
          penpot_review_source_path: source, penpot_review_source_node_id: 'position_label', penpot_review_instance_path: invocation,
          props: { text: 'Location', font_family: 'Roboto', font_size: 13.333333, font_weight: 400, line_height_ratio: 1.2, ...props },
          layout: { width: { min: 72, preferred: 72, max: 72, stretch: 'Fixed' }, height: { stretch: 'Stretch' } },
        },
        axis: { component: 'Space', layout: {
          width: { min: 62, preferred: 62, max: 62, stretch: 'Fixed' }, height: { stretch: 'Stretch' },
        } },
      },
    };
    return document;
  }

  function rendered(asset: Board) {
    const row = asset.children.find((child) => child.type === 'board' &&
      child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'node-id') === 'row') as Board;
    const label = row.children.find((child) => child.type === 'board' &&
      child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'node-id') === 'label') as Board;
    const axis = row.children.find((child) => child.type === 'board' &&
      child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'node-id') === 'axis') as Board;
    const text = label.children.find((child) => child.type === 'text' &&
      child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'role') === 'text') as Text;
    return { row, label, axis, text };
  }

  it.each(['position_label', 'rotation_label', 'scale_label'] as const)(
    'creates and refreshes actual Details %s without moving its column or changing source metadata', async (sourceNodeId) => {
      const repo = process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../', import.meta.url));
      const authored = parseZuiDocument(readFileSync(repo + '/zircon_editor/assets/ui/editor/unreal/components/details.zui', 'utf8')).document;
      const document = fixture();
      document.nodes!.label = { ...structuredClone(authored.nodes![sourceNodeId]),
        penpot_review_source_path: 'res://ui/editor/unreal/components/details.zui',
        penpot_review_source_node_id: sourceNodeId,
        penpot_review_instance_path: document.nodes!.row['penpot_review_instance_path'],
      };
      expect(document.nodes!.label.props!['layout_padding_left']).toBe(20);
      const original = structuredClone(document), projection = projectZuiDocument(document);
      // This positive padding fixture needs full content that fits its 52-DIP
      // lane. A 100-DIP advance correctly grows the authored Fixed column to 88.
      const advance = 51.75;
      rendererMock(() => advance, projection.shapes.find((shape) => shape.nodeId === 'label')!.text!.fontSize!);
      const asset = await createAssetBoard('authored-label.zui', document, projection);
      const { label, axis, text } = rendered(asset);
      expect(label.width).toBe(72);
      expect([text.x - label.x, text.width]).toEqual([20, 52]);
      const measurement = linearContentMeasurementSnapshot(asset).find((item) => item.nodeId === 'label')!;
      expect(measurement.text).toBe(document.nodes!.label.props!['text']);
      expect(measurement.width).toBeCloseTo(advance);
      expect(measurement.width).toBeLessThanOrEqual(label.width - 20);
      const identity = ['sourcePath', 'sourceNodeId', 'instancePath', 'controlId'].map((key) => label.getSharedPluginData(ZUI_METADATA_NAMESPACE, key));
      const originalMetadata = asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'document');
      refreshAssetTextLayout(asset, document, projection);
      expect([label.width, text.x - label.x, text.width]).toEqual([72, 20, 52]);
      const geometry = [label.x, label.y, label.width, label.height, axis.x, axis.y, axis.width, axis.height, text.x, text.y, text.width, text.height];
      const styleGuard = text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-style-guard');
      writeExportShapeGuard(asset);
      refreshAssetTextLayout(asset, document, projection);
      expect([label.x, label.y, label.width, label.height, axis.x, axis.y, axis.width, axis.height, text.x, text.y, text.width, text.height]).toEqual(geometry);
      expect(() => assertExportShapeGuard(asset)).not.toThrow();
      expect(text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-style-guard')).toBe(styleGuard);
      expect(captureNodeTexts(label).text?.characters).toBe(document.nodes!.label.props!['text']);
      expect(['sourcePath', 'sourceNodeId', 'instancePath', 'controlId'].map((key) => label.getSharedPluginData(ZUI_METADATA_NAMESPACE, key))).toEqual(identity);
      expect(identity).toEqual(['res://ui/editor/unreal/components/details.zui', sourceNodeId,
        document.nodes!.row['penpot_review_instance_path'], document.nodes!.label.control_id]);
      expect(asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'document')).toBe(originalMetadata);
      expect(document).toEqual(original);
      text.x += 0.05;
      expect(() => assertExportShapeGuard(asset)).toThrow(/changed geometry/);
    },
  );

  const paddingCases: Array<{ name: string; props: Record<string, string | number>; left: number; width: number }> = [
    { name: 'no declared padding', props: {}, left: 0, width: 72 },
    { name: 'explicit zero', props: { layout_padding_left: 0 }, left: 0, width: 72 },
    { name: 'token and asymmetric right padding', props: { layout_padding_left: '$label.name-inset', layout_padding_right: 12 }, left: 20, width: 40 },
    { name: 'a narrow remaining lane', props: { layout_padding_left: 68, layout_padding_right: 2 }, left: 68, width: 2 },
  ];
  it.each(paddingCases)('preserves $name during creation and repeated refresh', async ({ props, left, width }) => {
    rendererMock();
    const document = fixture(props), original = structuredClone(document), projection = projectZuiDocument(document);
    const asset = await createAssetBoard('label-padding.zui', document, projection);
    const { label, text } = rendered(asset);
    expect([label.width, text.x - label.x, text.width]).toEqual([72, left, width]);
    refreshAssetTextLayout(asset, document, projection);
    refreshAssetTextLayout(asset, document, projection);
    expect([label.width, text.x - label.x, text.width]).toEqual([72, left, width]);
    expect(document).toEqual(original);
  });

  it('keeps state zero authoritative and leaves non-Label control insets to their painter', () => {
    const document = fixture({ layout_padding_left: 20 });
    const node = document.nodes!.label;
    node.state = { layout_padding_left: 0 };
    expect(authoredLabelTextInsets(document, node, 72)).toEqual({ left: 0, right: 0 });
    expect(authoredLabelTextInsets(document, { ...node, component: 'InputField' }, 72)).toBeUndefined();
  });

  it.each([
    { name: 'actual Location fractional advance', value: 'Location', advance: 52.2265625,
      column: 72.25, left: 20, right: 0, align: 'left', ellipsis: false },
    { name: 'actual title fractional advance', value: 'Zircon Editor', advance: 76.23697509765625,
      column: 88.25, left: 0, right: 12, align: 'left', ellipsis: true },
    { name: 'centered asymmetric lane', value: 'Location', advance: 52.2265625,
      column: 72.25, left: 12, right: 8, align: 'center', ellipsis: false },
    { name: 'right aligned asymmetric lane', value: 'Location', advance: 52.2265625,
      column: 72.25, left: 12, right: 8, align: 'right', ellipsis: false },
  ])('keeps $name on one editable measured line without resizing its owner', async ({ value, advance, column, left, right, align, ellipsis }) => {
    const document = fixture({ text: value, text_align: align,
      layout_padding_left: left, layout_padding_right: right,
      ...(ellipsis ? { text_overflow: 'ellipsis' } : {}) });
    document.nodes!.label.layout = {
      width: { min: column, preferred: column, max: column, stretch: 'Fixed' },
      height: { stretch: 'Stretch' },
    };
    const original = structuredClone(document), projection = projectZuiDocument(document);
    const normalFontSize = projection.shapes.find((shape) => shape.nodeId === 'label')!.text!.fontSize!;
    rendererMock(() => advance, normalFontSize);
    const asset = await createAssetBoard('fractional-label.zui', document, projection);
    const { row, label, axis, text } = rendered(asset);
    // Real Penpot arranges Stretch children before text settling. This mock has
    // no SDK scheduler, so use the same native arranger as refresh for that step.
    applyWeightedFlexLayout(row, document,
      projection.shapes.find((shape) => shape.nodeId === 'row')!.container,
      measuredLinearDesiredSizes(asset, document, projection));
    expect([row.height, label.height, axis.height]).toEqual([28, 28, 28]);
    const ownerGeometry = [label.x, label.y, label.width, label.height, axis.x, axis.y, axis.width, axis.height];
    const styleGuard = text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-style-guard');
    const sourceMetadata = asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'document');
    const identities = ['sourcePath', 'sourceNodeId', 'instancePath', 'controlId']
      .map((key) => label.getSharedPluginData(ZUI_METADATA_NAMESPACE, key));
    expect(text.growType).toBe('fixed'); // Text settling has not consumed the full-content measurement yet.
    expect(text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'single-line-overflow')).toBe(ellipsis ? 'ellipsis' : '');
    const measurement = linearContentMeasurementSnapshot(asset).find((item) => item.nodeId === 'label')!;
    expect(measurement.width).toBeCloseTo(Math.ceil(advance * 16) / 16);
    expect(measurement.width).toBeLessThanOrEqual(column - left - right);
    refreshAssetTextLayout(asset, document, projection);
    expect(text.growType).toBe('auto-width');
    expect(text.layoutChild!.absolute).toBe(true);
    expect(text.characters).toBe(value);
    expect(text.width).toBe(Math.ceil(advance));
    expect(Number(text.fontSize)).toBe(normalFontSize);
    expect(text.align).toBe(align);
    const laneWidth = column - left - right;
    const alignment = align === 'right' ? 1 : align === 'center' ? 0.5 : 0;
    expect(text.x - label.x).toBeCloseTo(left + (laneWidth - text.width) * alignment);
    // The SDK's whole-pixel text box does not change the original ink alignment.
    expect(text.x - label.x + (text.width - advance) * alignment)
      .toBeCloseTo(left + (laneWidth - advance) * alignment);
    expect([label.x, label.y, label.width, label.height, axis.x, axis.y, axis.width, axis.height]).toEqual(ownerGeometry);
    const textGeometry = [text.x, text.y, text.width, text.height];
    const resizeCount = vi.mocked(text.resize).mock.calls.length;
    writeExportShapeGuard(asset);
    refreshAssetTextLayout(asset, document, projection);
    refreshAssetTextLayout(asset, document, projection);
    expect([text.x, text.y, text.width, text.height]).toEqual(textGeometry);
    expect(vi.mocked(text.resize).mock.calls).toHaveLength(resizeCount);
    expect(() => assertExportShapeGuard(asset)).not.toThrow();
    expect(captureNodeTexts(label).text).toMatchObject({ characters: value, align });
    expect(text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-style-guard')).toBe(styleGuard);
    expect(asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'document')).toBe(sourceMetadata);
    expect(['sourcePath', 'sourceNodeId', 'instancePath', 'controlId']
      .map((key) => label.getSharedPluginData(ZUI_METADATA_NAMESPACE, key))).toEqual(identities);
    expect(document).toEqual(original);
    text.x += 0.05;
    expect(() => assertExportShapeGuard(asset)).toThrow(/changed geometry/);
  });

  it.each([
    { name: 'long content', value: 'Location', advance: 53, props: {} },
    { name: 'an explicit line break', value: 'Loca\ntion', advance: 40, props: {} },
    { name: 'an authored word wrap', value: 'Location', advance: 40, props: { wrap: 'word' } },
    { name: 'an authored glyph wrap', value: 'Location', advance: 40, props: { wrap: 'glyph' } },
  ])('preserves fixed paragraph layout for $name', async ({ value, advance, props }) => {
    const document = fixture({ text: value, layout_padding_left: 20, ...props });
    const projection = projectZuiDocument(document);
    rendererMock(() => advance, projection.shapes.find((shape) => shape.nodeId === 'label')!.text!.fontSize!);
    const asset = await createAssetBoard('wrapped-label.zui', document, projection);
    const { label, text } = rendered(asset);
    refreshAssetTextLayout(asset, document, projection);
    refreshAssetTextLayout(asset, document, projection);
    expect(text.growType).toBe('fixed');
    expect([label.width, text.x - label.x, text.width, text.height]).toEqual([72, 20, 52, 28]);
    expect(text.characters).toBe(value);
  });

  it('requires a matching full-content measurement and restores fixed layout when the lane narrows', async () => {
    const document = fixture({ layout_padding_left: 20 });
    const projection = projectZuiDocument(document);
    rendererMock(() => 40, projection.shapes.find((shape) => shape.nodeId === 'label')!.text!.fontSize!);
    const asset = await createAssetBoard('changing-label.zui', document, projection);
    const { label, text } = rendered(asset);
    refreshAssetTextLayout(asset, document, projection);
    expect(text.growType).toBe('auto-width');
    text.characters = 'A changed value has no matching measurement';
    refreshAssetTextLayout(asset, document, projection);
    expect(text.growType).toBe('fixed');
    expect(text.width).toBe(52);
    text.characters = 'Location';
    refreshAssetTextLayout(asset, document, projection);
    expect(text.growType).toBe('auto-width');
    document.nodes!.label.state = { layout_padding_left: 60 };
    refreshAssetTextLayout(asset, document, projection);
    expect([text.growType, text.x - label.x, text.width]).toEqual(['fixed', 60, 12]);
  });

  it.each([-1, Number.NaN, Number.POSITIVE_INFINITY, '$missing.label-inset', 72])(
    'rejects unsupported or empty Label text lanes for padding %s', (padding) => {
      const document = fixture({ layout_padding_left: padding });
      expect(() => authoredLabelTextInsets(document, document.nodes!.label, 72)).toThrow(/Label text padding/);
    },
  );
});
