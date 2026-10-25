import type { LayoutReviewCaseMessage } from '../model';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { deflateSync } from 'node:zlib';
import { fileURLToPath } from 'node:url';
import { afterEach, beforeEach, vi } from 'vitest';
import { LayoutDependencies } from '../../tools/zui-layout-dependencies';
import fixtureSource from './roundtrip-fixture.zui?raw';
import {
  parseZuiDocument,
  normalizeZuiDocument,
  serializeZuiDocument,
  encodeZuiMetadata,
  decodeZuiMetadata,
  type ZuiDocument,
  type ZuiNode,
  type ZuiTable,
} from './zui-document';
import {
  projectZuiDocument,
  cloneProjectionSnapshot,
  reconcileZuiDocument,
} from './penpot-projection';
import {
  reviewBoardNeedsResponsiveRebuild,
  reviewDocumentForCase,
  importedActualNormalReviewCase,
  reviewCaseFromSavedMetadata,
  runtimeSceneMediaForCase,
  runtimeSceneMediaReviewTarget,
  reviewUtf8ByteLength,
} from './zui-review-case';
import { reviewHost } from './zui-review-host';
import type { ToolbarSourceNode } from './zui-viewport-toolbar-publication';
import { buttonStatePreviewNode } from './zui-button-state-projection';
import { editorButtonState } from './zui-editor-button-style';

const reviewCase: LayoutReviewCaseMessage = {
  id: 'hover-360x520-dpi1',
  sourcePath: 'test.zui',
  host: 'component',
  viewport: { width: 360, height: 520 },
  dpi: 1,
  locale: 'en-US',
  state: 'hover',
  data: {},
};

describe('actual portable scene bytes UTF-8 limits', () => {
  beforeEach(() => { vi.stubGlobal('TextEncoder', undefined); vi.stubGlobal('crypto', undefined); });
  afterEach(() => vi.unstubAllGlobals());

  it('counts UTF-8 boundaries, multi-byte text, surrogate pairs and lone-surrogate replacements', () => {
    expect(globalThis.TextEncoder).toBeUndefined();
    expect(globalThis.crypto).toBeUndefined();
    const values: Array<[string, number]> = [
      ['', 0], ['ASCII', 5], ['\u007f', 1], ['\u0080', 2], ['\u07ff', 2], ['\u0800', 3], ['\uffff', 3],
      ['\u4e2d\u6587', 6], ['\ud83d\ude80', 4], ['\ud800', 3], ['\udc00', 3],
      ['\ud800\ud83d\ude80', 7], ['\udc00\ud800', 6], ['\ud800a\udc00', 7],
    ];
    for (const [text, expected] of values) {
      expect(reviewUtf8ByteLength(text)).toBe(expected);
      expect(reviewUtf8ByteLength(text)).toBe(Buffer.byteLength(text, 'utf8'));
    }
  });

  it.each([
    ['CJK', '\u4e2d'.repeat(1365) + 'A'],
    ['surrogate pairs', '\ud83d\ude80'.repeat(1024)],
  ])('preserves the 4096-byte authored Workbench override limit for %s', (_name, text) => {
    const { document } = parseZuiDocument(fixtureSource), sourcePath = 'zircon_editor/assets/ui/editor/windows/workbench_window.zui';
    const identity = { sourcePath, sourceNodeId: 'toolbar_menu_title', controlId: 'WorkbenchMenuTitle', instancePath: '[]' };
    const node = document.nodes!['root'];
    node.control_id = identity.controlId;
    node['penpot_review_source_path'] = sourcePath;
    node['penpot_review_source_node_id'] = identity.sourceNodeId;
    node['penpot_review_instance_path'] = identity.instancePath;
    node.props = { ...(node.props ?? {}), text: 'Menu' };
    const value = (override: string): LayoutReviewCaseMessage => ({ ...reviewCase, host: 'editor', locale: 'zh-CN', state: 'default',
      data: { workbenchState: { ...identity, textOverrides: [{ ...identity, property: 'text', value: override }] } } });
    expect(Buffer.byteLength(text, 'utf8')).toBe(4096);
    expect(reviewDocumentForCase(document, value(text)).nodes!['root'].props?.['text']).toBe(text);
    expect(node.props['text']).toBe('Menu');
    expect(() => reviewDocumentForCase(document, value(text + 'A'))).toThrow('exceeds 4096 UTF-8 bytes');
  });
});

describe('review host state projection', () => {
  it.each([
    [{ ...reviewCase, id: '' }, 'Review case id'],
    [{ ...reviewCase, sourcePath: '' }, 'Review case sourcePath'],
    [{ ...reviewCase, host: 'browser' }, 'Review case host'],
    [{ ...reviewCase, viewport: null }, 'Review case viewport'],
    [
      { ...reviewCase, viewport: { width: 360, height: 520, depth: 1 } },
      'Unknown review viewport field',
    ],
    [{ ...reviewCase, data: null }, 'Review case data'],
    [{ ...reviewCase, hidden: true }, 'Unknown review case field'],
  ])('rejects a malformed review case envelope %j', (value, message) => {
    const { document } = parseZuiDocument(fixtureSource);
    expect(() =>
      reviewDocumentForCase(
        document,
        value as unknown as LayoutReviewCaseMessage,
      ),
    ).toThrow(message);
  });

  it('accepts and validates optional theme and review-host identity fields', () => {
    const { document } = parseZuiDocument(fixtureSource);
    expect(() =>
      reviewDocumentForCase(document, {
        ...reviewCase,
        themeSourcePath:
          'zircon_editor/assets/ui/editor/theme/editor_tokens.zui',
        reviewHost: {
          path: 'evidence/review-host.zui',
          sha256: 'a'.repeat(64),
        },
      }),
    ).not.toThrow();
    expect(() =>
      reviewDocumentForCase(document, {
        ...reviewCase,
        reviewHost: {
          path: '',
          sha256: 'not-a-sha256',
        },
      }),
    ).toThrow('Review case reviewHost');
  });

  it('targets one authored Workbench control using source path and node identity', () => {
    const document: ZuiDocument = {
      asset: { kind: 'view', id: 'review-host', version: 2 },
      root: { node: 'root' },
      nodes: {
        root: { component: 'Overlay', children: [{ node: 'palette' }, { node: 'other' }] },
        palette: {
          component: 'CommandPalette',
          control_id: 'WorkbenchCommandPalette',
          penpot_review_source_path:
            'zircon_editor/assets/ui/editor/windows/workbench_window.zui',
          penpot_review_source_node_id: 'command_palette',
          props: { open: false, popup_open: false },
        },
        other: {
          component: 'CommandPalette',
          control_id: 'WorkbenchCommandPalette',
          penpot_review_source_path:
            'zircon_editor/assets/ui/editor/windows/other_window.zui',
          penpot_review_source_node_id: 'command_palette',
          props: { open: false, popup_open: false },
        },
      },
    };
    const original = normalizeZuiDocument(document);
    const reviewed = reviewDocumentForCase(document, {
      ...reviewCase,
      host: 'editor',
      state: 'open',
      data: {
        workbenchState: {
          sourcePath:
            'zircon_editor/assets/ui/editor/windows/workbench_window.zui',
          controlId: 'WorkbenchCommandPalette',
          sourceNodeId: 'command_palette',
        },
      },
    });

    expect(reviewed.nodes?.['palette'].state).toMatchObject({
      open: true,
      popup_open: true,
    });
    expect(reviewed.nodes?.['other'].state).toBeUndefined();
    expect(normalizeZuiDocument(document)).toEqual(original);
  });

  it('accepts Chinese Workbench locale only through an authored text override', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const node = document.nodes!['root'];
    node.control_id = 'WorkbenchMenuTitle';
    node['penpot_review_source_path'] =
      'zircon_editor/assets/ui/editor/windows/workbench_window.zui';
    node['penpot_review_source_node_id'] = 'toolbar_menu_title';
    node['penpot_review_instance_path'] = '[]';
    node.props = { ...(node.props ?? {}), text: 'Menu' };

    const reviewed = reviewDocumentForCase(document, {
      ...reviewCase,
      host: 'editor',
      locale: 'zh-CN',
      state: 'default',
      data: {
        workbenchState: {
          sourcePath:
            'zircon_editor/assets/ui/editor/windows/workbench_window.zui',
          controlId: 'WorkbenchMenuTitle',
          sourceNodeId: 'toolbar_menu_title',
          instancePath: '[]',
          textOverrides: [
            {
              sourcePath:
                'zircon_editor/assets/ui/editor/windows/workbench_window.zui',
              sourceNodeId: 'toolbar_menu_title',
              controlId: 'WorkbenchMenuTitle',
              instancePath: '[]',
              property: 'text',
              value: '菜单',
            },
          ],
        },
      },
    });

    expect(reviewed.nodes?.['root'].props?.['text']).toBe('菜单');
    expect(() =>
      reviewDocumentForCase(document, {
        ...reviewCase,
        host: 'editor',
        locale: 'zh-CN',
        state: 'default',
        data: {},
      }),
    ).toThrow('zh-CN workbench cases require at least one text override');
  });

  it('scopes a Workbench scroll position to its selected source control', () => {
    const document: ZuiDocument = {
      asset: { kind: 'view', id: 'review-host', version: 2 },
      root: { node: 'root' },
      nodes: {
        root: {
          component: 'Overlay',
          children: [{ node: 'selected_scroll' }, { node: 'other_scroll' }],
        },
        selected_scroll: {
          component: 'ScrollableBox',
          control_id: 'WorkbenchSceneTreeScroll',
          penpot_review_source_path:
            'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui',
          penpot_review_source_node_id: 'tree_scroll',
          layout: { container: { kind: 'ScrollableBox' } },
          props: {},
        },
        other_scroll: {
          component: 'ScrollableBox',
          control_id: 'WorkbenchModuleListScroll',
          penpot_review_source_path:
            'zircon_editor/assets/ui/editor/workbench/modules/core/index/module_workspace.zui',
          penpot_review_source_node_id: 'module_scroll',
          layout: { container: { kind: 'ScrollableBox' } },
          props: {},
        },
      },
    };

    const reviewed = reviewDocumentForCase(document, {
      ...reviewCase,
      host: 'editor',
      state: 'scroll-after',
      data: {
        workbenchState: {
          sourcePath:
            'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui',
          controlId: 'WorkbenchSceneTreeScroll',
          sourceNodeId: 'tree_scroll',
        },
      },
    });

    expect(reviewed.nodes?.['selected_scroll'].props).toMatchObject({
      __zircon_review_scroll_position: 'end',
    });
    expect(reviewed.nodes?.['other_scroll'].props).toEqual({});
  });

  it('combines a popup target with an exact independent scroll target', () => {
    const document: ZuiDocument = {
      asset: { kind: 'view', id: 'review-host', version: 2 },
      root: { node: 'root' },
      nodes: {
        root: {
          component: 'Overlay',
          children: [{ node: 'trigger' }, { node: 'popup' }, { node: 'tree' }],
        },
        trigger: { component: 'Button', control_id: 'OpenMenu' },
        popup: {
          component: 'Popup',
          control_id: 'WorkbenchToolbarMenu',
          widget: {
            popup_anchor: { kind: 'control', control_id: 'OpenMenu' },
          },
          penpot_review_source_path:
            'zircon_editor/assets/ui/editor/windows/workbench_window.zui',
          penpot_review_source_node_id: 'toolbar_main_menu',
          props: { open: false, popup_open: false },
        },
        tree: {
          component: 'ScrollableBox',
          control_id: 'WorkbenchSceneTree',
          penpot_review_source_path:
            'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui',
          penpot_review_source_node_id: 'scene_tree',
          layout: { container: { kind: 'ScrollableBox' } },
          props: {},
        },
      },
    };
    const reviewed = reviewDocumentForCase(document, {
      ...reviewCase,
      host: 'editor',
      state: 'open',
      scrollPosition: 'end',
      data: {
        workbenchState: {
          sourcePath:
            'zircon_editor/assets/ui/editor/windows/workbench_window.zui',
          controlId: 'WorkbenchToolbarMenu',
          sourceNodeId: 'toolbar_main_menu',
          scrollTarget: {
            sourcePath:
              'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui',
            controlId: 'WorkbenchSceneTree',
            sourceNodeId: 'scene_tree',
          },
        },
      },
    });

    expect(reviewed.nodes?.['popup'].state).toMatchObject({
      open: true,
      popup_open: true,
    });
    expect(reviewed.nodes?.['tree'].props).toMatchObject({
      __zircon_review_scroll_position: 'end',
    });
  });

  it.each([
    [{ sourcePath: 'window.zui', controlId: 'Menu', unexpected: true }, 'Unknown workbenchState field'],
    [{ sourcePath: 'window.zui', controlId: '' }, 'Workbench state controlId'],
    [
      {
        sourcePath: 'window.zui',
        controlId: 'Menu',
        scrollTarget: { sourcePath: 'scene.zui', controlId: 'Tree', extra: true },
      },
      'Unknown Workbench scrollTarget field',
    ],
  ])('rejects malformed Workbench state selectors %j', (selector, message) => {
    const { document } = parseZuiDocument(fixtureSource);
    expect(() =>
      reviewDocumentForCase(document, {
        ...reviewCase,
        host: 'editor',
        data: { workbenchState: selector },
      }),
    ).toThrow(message);
  });

  it('replaces authored scalar and keyboard button states only in explicit review states', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const node = document.nodes!['cancel'];
    node.props = {
      ...node.props,
      button_interaction_state: 'disabled',
      enter_pressed: true,
    };
    node.state = { button_interaction_state: 'pressed', enter_pressed: true };
    const original = normalizeZuiDocument(document);
    for (const state of [
      'hover',
      'pressed',
      'focused',
      'disabled',
      'selected',
    ]) {
      const review = reviewDocumentForCase(document, { ...reviewCase, state });
      const result = editorButtonState(review.nodes!['cancel']);
      expect(result.interaction).toBe(state === 'selected' ? 'normal' : state);
      expect(result.selected).toBe(state === 'selected');
    }
    expect(normalizeZuiDocument(document)).toEqual(original);
    expect(
      reviewDocumentForCase(document, { ...reviewCase, state: 'default' }),
    ).toBe(document);
  });

  it('applies the authored state selector without changing the runtime source', () => {
    const { document } = parseZuiDocument(fixtureSource);
    document.stylesheets = [
      {
        rules: [
          {
            selector: 'RoundtripButton:hover',
            set: { self: { background: { color: '#abcdef' } } },
          },
          {
            selector: 'RoundtripButton:disabled',
            set: { self: { background: { color: '#123456' } } },
          },
        ],
      },
    ];
    document.nodes!['cancel'].props!['disabled'] = true;
    const original = normalizeZuiDocument(document);
    const review = reviewDocumentForCase(document, reviewCase);
    const cancel = projectZuiDocument(review).shapes.find(
      ({ nodeId }) => nodeId === 'cancel',
    )!;
    expect(cancel.paint.fillColor).toBe('#abcdef');
    expect(normalizeZuiDocument(document)).toEqual(original);
    expect(
      reviewDocumentForCase(document, { ...reviewCase, state: 'default' }),
    ).toBe(document);
  });

  it('projects Workbench responsive tiers from logical viewport width', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const responsiveNode = (props: ZuiTable): ZuiNode => ({
      component: 'Label',
      props,
      layout: {
        width: { preferred: 120.0, stretch: 'Fixed' },
        height: { preferred: 24.0, stretch: 'Fixed' },
      },
    });
    document.nodes!['narrow_only'] = responsiveNode({
      responsive_min_tier: 'narrow',
    });
    document.nodes!['regular_only'] = responsiveNode({
      responsive_min_tier: 'regular',
    });
    document.nodes!['wide_only'] = responsiveNode({
      responsive_min_tier: 'wide',
    });
    document.nodes!['narrow_to_regular'] = responsiveNode({
      responsive_min_tier: 'narrow',
      responsive_max_tier: 'regular',
    });
    document.nodes!['unknown_tier'] = responsiveNode({
      responsive_min_tier: 'future',
    });
    document.nodes!['root'].props!['responsive_min_tier'] = 'wide';
    const before = normalizeZuiDocument(document);

    const visibilityAt = (width: number) => {
      const review = reviewDocumentForCase(document, {
        ...reviewCase,
        state: 'default',
        viewport: { width, height: 520 },
      });
      return {
        review,
        narrow: review.nodes!['narrow_only'].props?.['visibility'],
        regular: review.nodes!['regular_only'].props?.['visibility'],
        wide: review.nodes!['wide_only'].props?.['visibility'],
        range: review.nodes!['narrow_to_regular'].props?.['visibility'],
        unknown: review.nodes!['unknown_tier'].props?.['visibility'],
        root: review.nodes!['root'].props?.['visibility'],
      };
    };

    expect(visibilityAt(480)).toMatchObject({
      narrow: 'collapsed',
      regular: 'collapsed',
      wide: 'collapsed',
      range: 'collapsed',
      unknown: undefined,
      root: undefined,
    });
    expect(visibilityAt(481)).toMatchObject({
      narrow: undefined,
      regular: 'collapsed',
      wide: 'collapsed',
      range: undefined,
    });
    expect(visibilityAt(640)).toMatchObject({
      narrow: undefined,
      regular: 'collapsed',
      wide: 'collapsed',
      range: undefined,
    });
    expect(visibilityAt(641)).toMatchObject({
      narrow: undefined,
      regular: undefined,
      wide: 'collapsed',
      range: undefined,
    });
    expect(visibilityAt(1259)).toMatchObject({
      narrow: undefined,
      regular: undefined,
      wide: 'collapsed',
      range: undefined,
    });
    expect(visibilityAt(1260)).toMatchObject({
      narrow: undefined,
      regular: undefined,
      wide: undefined,
      range: 'collapsed',
    });
    expect(visibilityAt(480).review).not.toBe(document);
    expect(normalizeZuiDocument(document)).toEqual(before);
  });

  it('materializes responsive layout props for the requested Penpot viewport', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const root = document.nodes!['root'];
    root.component = 'Stack';
    root.props = {
      ...root.props,
      direction: { xs: 'column', sm: 'column', md: 'row' },
      spacing: { xs: 1, md: 2 },
      display: { xs: 'none', md: 'flex' },
    };
    root.layout = {
      ...root.layout,
      container: { kind: 'Stack', gap: { xs: 4, md: 12 } },
    };
    const before = normalizeZuiDocument(document);

    const narrow = reviewDocumentForCase(document, {
      ...reviewCase,
      state: 'default',
      viewport: { width: 640, height: 520 },
    });
    expect(narrow).not.toBe(document);
    expect(narrow.nodes!['root'].props!['direction']).toBe('column');
    expect(narrow.nodes!['root'].props!['spacing']).toBe(1);
    expect(narrow.nodes!['root'].props!['display']).toBe('none');
    expect(narrow.nodes!['root'].layout!['container']).toMatchObject({
      kind: 'Stack',
      gap: 4,
    });

    const regular = reviewDocumentForCase(document, {
      ...reviewCase,
      state: 'default',
      viewport: { width: 900, height: 620 },
    });
    expect(regular.nodes!['root'].props!['direction']).toBe('row');
    expect(regular.nodes!['root'].props!['spacing']).toBe(2);
    expect(regular.nodes!['root'].props!['display']).toBe('flex');
    expect(normalizeZuiDocument(document)).toEqual(before);
  });

  it('rebuilds responsive boards when the viewport changes instead of cloning stale Stack geometry', () => {
    const { document } = parseZuiDocument(fixtureSource);
    document.nodes!['root'].component = 'Stack';
    document.nodes!['root'].props = {
      direction: { xs: 'column', md: 'row' },
    };

    expect(reviewBoardNeedsResponsiveRebuild(document, undefined, 1280)).toBe(
      true,
    );
    expect(reviewBoardNeedsResponsiveRebuild(document, 1280, 640)).toBe(true);
    expect(reviewBoardNeedsResponsiveRebuild(document, 640, 640)).toBe(false);
    document.nodes!['root'].props = { direction: 'row' };
    expect(reviewBoardNeedsResponsiveRebuild(document, undefined, 640)).toBe(
      false,
    );
  });

  it('marks an authored scroll container for start and end review without mutating source content', () => {
    const { document } = parseZuiDocument(fixtureSource);
    document.nodes!['root'].component = 'ScrollableBox';
    document.nodes!['root'].layout = {
      container: { kind: 'ScrollableBox', axis: 'Vertical' },
      height: { preferred: 80, stretch: 'Fixed' },
    };
    const before = normalizeZuiDocument(document);
    const start = reviewDocumentForCase(document, {
      ...reviewCase,
      state: 'scroll-before',
    });
    const end = reviewDocumentForCase(document, {
      ...reviewCase,
      state: 'scroll-after',
    });
    expect(
      start.nodes!['root'].props?.['__zircon_review_scroll_position'],
    ).toBe('start');
    expect(end.nodes!['root'].props?.['__zircon_review_scroll_position']).toBe(
      'end',
    );
    expect(end.nodes!['cancel'].props?.['visibility']).toBeUndefined();
    expect(normalizeZuiDocument(document)).toEqual(before);
  });

  it('composes open interaction with scroll-to-end without changing the source', () => {
    const { document } = parseZuiDocument(fixtureSource);
    document.nodes!['root'].component = 'ScrollableBox';
    document.nodes!['root'].layout = {
      container: { kind: 'ScrollableBox', axis: 'Vertical' },
    };
    document.nodes!['cancel'].component = 'WorkbenchToast';
    document.nodes!['cancel'].props = {
      text: 'Operation completed',
      open: false,
      popup_open: false,
    };
    const original = normalizeZuiDocument(document);
    const openedAtEnd = reviewDocumentForCase(document, {
      ...reviewCase,
      state: 'open',
      scrollPosition: 'end',
    });
    expect(openedAtEnd.nodes!['cancel'].state).toMatchObject({
      open: true,
      popup_open: true,
    });
    expect(
      openedAtEnd.nodes!['root'].props?.['__zircon_review_scroll_position'],
    ).toBe('end');
    expect(normalizeZuiDocument(document)).toEqual(original);
    expect(() =>
      reviewDocumentForCase(document, {
        ...reviewCase,
        state: 'open',
        scrollPosition: 'middle' as 'end',
      }),
    ).toThrow(/scrollPosition/);
  });

  it('injects only native painter popup and drag states into a review clone', () => {
    const dialog: ZuiDocument = {
      asset: { kind: 'component', id: 'res://dialog.zui', version: 2 },
      components: { Dialog: { root: 'root' } },
      nodes: {
        root: {
          component: 'Dialog',
          props: { open: false, popup_open: false, title: 'Settings' },
        },
      },
    };
    const drag: ZuiDocument = {
      asset: { kind: 'component', id: 'res://drag.zui', version: 2 },
      components: { Drag: { root: 'root' } },
      nodes: {
        root: {
          component: 'DragOverlay',
          props: {
            dragging: false,
            drop_allowed: true,
            payload_label: 'StoneWall.mesh',
          },
        },
      },
    };
    const dialogBefore = normalizeZuiDocument(dialog);
    const dragBefore = normalizeZuiDocument(drag);

    const opened = reviewDocumentForCase(dialog, {
      ...reviewCase,
      state: 'open',
    });
    const closed = reviewDocumentForCase(dialog, {
      ...reviewCase,
      state: 'closed',
    });
    const allowed = reviewDocumentForCase(drag, {
      ...reviewCase,
      state: 'drop-allowed',
    });
    const blocked = reviewDocumentForCase(drag, {
      ...reviewCase,
      state: 'drop-blocked',
    });

    expect(opened.nodes?.['root'].state).toMatchObject({
      open: true,
      popup_open: true,
    });
    expect(closed.nodes?.['root'].state).toMatchObject({
      open: false,
      popup_open: false,
    });
    expect(allowed.nodes?.['root'].state).toMatchObject({
      dragging: true,
      drop_hovered: true,
      active_drag_target: true,
      drop_allowed: true,
    });
    expect(blocked.nodes?.['root'].state).toMatchObject({
      dragging: true,
      drop_hovered: true,
      active_drag_target: true,
      drop_allowed: false,
    });
    expect(normalizeZuiDocument(dialog)).toEqual(dialogBefore);
    expect(normalizeZuiDocument(drag)).toEqual(dragBefore);
  });

  it('applies authored generic popup states including focus return', () => {
    const popup: ZuiDocument = {
      asset: { kind: 'component', id: 'res://dropdown-popup.zui', version: 2 },
      components: { DropdownPopup: { root: 'root' } },
      nodes: {
        root: {
          component: 'DropdownPopup',
          widget: {
            popup_anchor: { kind: 'control', control_id: 'DropdownTrigger' },
          },
          props: { open: false, popup_open: false, options: ['Scene'] },
        },
        trigger: {
          component: 'Button',
          control_id: 'DropdownTrigger',
          props: { focused: false, focus_visible: false },
        },
      },
    };
    const before = normalizeZuiDocument(popup);
    const opened = reviewDocumentForCase(popup, {
      ...reviewCase,
      state: 'open',
    });
    const closed = reviewDocumentForCase(popup, {
      ...reviewCase,
      state: 'closed',
    });
    const returned = reviewDocumentForCase(popup, {
      ...reviewCase,
      state: 'focus-return',
    });
    expect(opened.nodes?.['root'].state).toMatchObject({
      open: true,
      popup_open: true,
    });
    expect(closed.nodes?.['root'].state).toMatchObject({
      open: false,
      popup_open: false,
    });
    expect(returned.nodes?.['root'].state).toMatchObject({
      open: false,
      popup_open: false,
    });
    expect(returned.nodes?.['trigger'].state).toMatchObject({
      focused: true,
      focus_visible: true,
    });
    expect(normalizeZuiDocument(popup)).toEqual(before);
  });

  it('limits PopupMenu state to its selected host and returns focus to its control anchor', () => {
    const mainSource = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
    const popupSource = 'zircon_editor/assets/ui/editor/unreal/components/menu_popups.zui';
    const anchorSource = 'zircon_editor/assets/ui/editor/unreal/components/menus.zui';
    const popupInstance = JSON.stringify([
      { sourcePath: mainSource, sourceNodeId: 'menu_popups' },
    ]);
    const anchorInstance = JSON.stringify([
      { sourcePath: mainSource, sourceNodeId: 'menu_bar' },
    ]);
    const source: ZuiDocument = {
      asset: { kind: 'view', id: 'review-host', version: 2 },
      root: { node: 'root' },
      nodes: {
        root: {
          component: 'Overlay',
          children: [
            { node: 'popup_a' },
            { node: 'slot_a' },
            { node: 'popup_b' },
            { node: 'slot_b' },
          ],
        },
        popup_a: {
          component: 'PopupMenu',
          control_id: 'PopupA',
          widget: {
            popup_anchor: { kind: 'control', control_id: 'MenuSlot2' },
          },
          props: {
            open: false,
            popup_open: false,
            menu_items: ['New Level|action=menu.item.new_scene'],
          },
          penpot_review_source_path: popupSource,
          penpot_review_source_node_id: 'popup_a',
          penpot_review_instance_path: popupInstance,
        },
        slot_a: {
          component: 'Button',
          control_id: 'MenuSlot2',
          penpot_review_source_path: anchorSource,
          penpot_review_source_node_id: 'menu_slot2',
          penpot_review_instance_path: anchorInstance,
        },
        popup_b: {
          component: 'PopupMenu',
          control_id: 'PopupB',
          widget: {
            popup_anchor: { kind: 'control', control_id: 'MenuSlot3' },
          },
          props: { open: false, popup_open: false, menu_items: ['Window'] },
          penpot_review_source_path: popupSource,
          penpot_review_source_node_id: 'popup_b',
          penpot_review_instance_path: JSON.stringify([
            { sourcePath: popupSource, sourceNodeId: 'other_menus' },
          ]),
        },
        slot_b: {
          component: 'Button',
          control_id: 'MenuSlot3',
        },
      },
    };
    const review = (state: 'open' | 'closed' | 'focus-return') =>
      reviewDocumentForCase(source, {
        ...reviewCase,
        host: 'editor',
        state,
        data: {
          workbenchState: {
            sourcePath: popupSource,
            sourceNodeId: 'popup_a',
            controlId: 'PopupA',
          },
        },
      });

    const opened = review('open');
    expect(opened.nodes?.['popup_a'].state).toMatchObject({ open: true });
    expect(opened.nodes?.['popup_b'].state).toBeUndefined();

    const closed = review('closed');
    expect(closed.nodes?.['popup_a'].state).toMatchObject({ open: false });
    expect(closed.nodes?.['popup_b'].state).toBeUndefined();

    const returned = review('focus-return');
    expect(returned.nodes?.['popup_a'].state).toMatchObject({ open: false });
    expect(returned.nodes?.['slot_a'].state).toMatchObject({
      focused: true,
      focus_visible: true,
    });
    expect(returned.nodes?.['popup_b'].state).toBeUndefined();
  });

  it('rejects duplicate invoking control IDs even when the selected popup has an exact instance', () => {
    const sourcePath =
      'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
    const instanceA = JSON.stringify([
      { sourcePath, sourceNodeId: 'toolbar_a' },
    ]);
    const instanceB = JSON.stringify([
      { sourcePath, sourceNodeId: 'toolbar_b' },
    ]);
    const source: ZuiDocument = {
      asset: { kind: 'view', id: sourcePath, version: 2 },
      root: { node: 'root' },
      nodes: {
        root: {
          component: 'Overlay',
          children: [
            { node: 'popup_a' },
            { node: 'trigger_a' },
            { node: 'popup_b' },
            { node: 'trigger_b' },
          ],
        },
        popup_a: {
          component: 'DropdownPopup',
          control_id: 'SharedPopup',
          widget: {
            popup_anchor: { kind: 'control', control_id: 'SharedTrigger' },
          },
          props: { open: false, popup_open: false },
          penpot_review_source_path: sourcePath,
          penpot_review_source_node_id: 'popup',
          penpot_review_instance_path: instanceA,
        },
        trigger_a: {
          component: 'Button',
          control_id: 'SharedTrigger',
          props: { focused: false, focus_visible: false },
          penpot_review_source_path: sourcePath,
          penpot_review_source_node_id: 'trigger',
          penpot_review_instance_path: instanceA,
        },
        popup_b: {
          component: 'DropdownPopup',
          control_id: 'SharedPopup',
          widget: {
            popup_anchor: { kind: 'control', control_id: 'SharedTrigger' },
          },
          props: { open: false, popup_open: false },
          penpot_review_source_path: sourcePath,
          penpot_review_source_node_id: 'popup',
          penpot_review_instance_path: instanceB,
        },
        trigger_b: {
          component: 'Button',
          control_id: 'SharedTrigger',
          props: { focused: false, focus_visible: false },
          penpot_review_source_path: sourcePath,
          penpot_review_source_node_id: 'trigger',
          penpot_review_instance_path: instanceB,
        },
      },
    };
    expect(() => reviewDocumentForCase(source, {
      ...reviewCase,
      host: 'editor',
      sourcePath,
      state: 'focus-return',
      data: {
        workbenchState: {
          sourcePath,
          sourceNodeId: 'popup',
          controlId: 'SharedPopup',
          instancePath: instanceB,
        },
      },
    })).toThrow('Workbench focus return anchor SharedTrigger resolved 2 nodes');
  });

  it('requires one uniquely mounted invoking control across component sources', () => {
    const popupSource =
      'zircon_editor/assets/ui/editor/unreal/components/menu_popups.zui';
    const popupInstance = JSON.stringify([
      { sourcePath: popupSource, sourceNodeId: 'toolbar' },
    ]);
    const otherSource =
      'zircon_editor/assets/ui/editor/unreal/components/menus.zui';
    const otherInstance = JSON.stringify([
      { sourcePath: otherSource, sourceNodeId: 'toolbar' },
    ]);
    const makeSource = (anchors: readonly ZuiNode[]): ZuiDocument => {
      const nodes: Record<string, ZuiNode> = {
        root: {
          component: 'Overlay',
          children: [
            { node: 'popup' },
            ...anchors.map((_, index) => ({ node: `trigger_${index}` })),
          ],
        },
        popup: {
          component: 'DropdownPopup',
          control_id: 'ScopedPopup',
          widget: {
            popup_anchor: { kind: 'control', control_id: 'ScopedTrigger' },
          },
          props: { open: false, popup_open: false },
          penpot_review_source_path: popupSource,
          penpot_review_source_node_id: 'popup',
          penpot_review_instance_path: popupInstance,
        },
      };
      anchors.forEach((anchor, index) => {
        nodes[`trigger_${index}`] = anchor;
      });
      return {
        asset: { kind: 'view', id: popupSource, version: 2 },
        root: { node: 'root' },
        nodes,
      };
    };
    const review = (anchors: readonly ZuiNode[]) =>
      reviewDocumentForCase(makeSource(anchors), {
        ...reviewCase,
        host: 'editor',
        sourcePath: popupSource,
        state: 'focus-return',
        data: {
          workbenchState: {
            sourcePath: popupSource,
            sourceNodeId: 'popup',
            controlId: 'ScopedPopup',
            instancePath: popupInstance,
          },
        },
      });
    const anchor = (sourcePath: string, instancePath: string): ZuiNode => ({
      component: 'Button',
      control_id: 'ScopedTrigger',
      penpot_review_source_path: sourcePath,
      penpot_review_source_node_id: 'trigger',
      penpot_review_instance_path: instancePath,
    });

    expect(() => review([])).toThrow(
      'Workbench focus return anchor ScopedTrigger resolved 0 nodes',
    );
    const returned = review([anchor(otherSource, otherInstance)]);
    expect(returned.nodes?.['trigger_0'].state).toMatchObject({
      focused: true, focus_visible: true,
    });
    expect(() => review([
      anchor(popupSource, popupInstance), anchor(otherSource, otherInstance),
    ])).toThrow('Workbench focus return anchor ScopedTrigger resolved 2 nodes');
    expect(() =>
      review([
        anchor(popupSource, popupInstance),
        anchor(popupSource, popupInstance),
      ]),
    ).toThrow('Workbench focus return anchor ScopedTrigger resolved 2 nodes');
  });

  it('returns focus for the authored Unreal Window menu across component mounts', () => {
    const repo = process.env['ZUI_LAYOUT_REPO_ROOT'] ??
      fileURLToPath(new URL('../../../../../../', import.meta.url));
    const mainSource = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
    const popupSource = 'zircon_editor/assets/ui/editor/unreal/components/menu_popups.zui';
    const anchorSource = 'zircon_editor/assets/ui/editor/unreal/components/menus.zui';
    const readDocument = (path: string) =>
      parseZuiDocument(readFileSync(`${repo}/${path}`, 'utf8')).document;
    const popupEntry = Object.entries(readDocument(popupSource).nodes ?? {})
      .find(([, node]) => node.control_id === 'UnrealWindowMenuPopup')!;
    expect(popupEntry).toBeDefined();
    const [popupNodeId, popup] = popupEntry;
    const widget = popup['widget'] as {
      popup_anchor: { kind: string; control_id: string };
    };
    expect(widget.popup_anchor.kind).toBe('control');
    expect(widget.popup_anchor.control_id).toBe('MenuSlot2');
    const anchorEntry = Object.entries(readDocument(anchorSource).nodes ?? {})
      .find(([, node]) => node.control_id === widget.popup_anchor.control_id)!;
    expect(anchorEntry).toBeDefined();
    const [anchorNodeId, anchor] = anchorEntry;
    const popupInstance = JSON.stringify([
      { sourcePath: mainSource, sourceNodeId: 'menu_popups' },
    ]);
    const anchorInstance = JSON.stringify([
      { sourcePath: mainSource, sourceNodeId: 'menu_bar' },
    ]);
    const source: ZuiDocument = {
      asset: { kind: 'view', id: mainSource, version: 2 },
      root: { node: 'root' },
      nodes: {
        root: { component: 'Overlay', children: [{ node: 'menus' }, { node: 'popups' }] },
        menus: { component: 'HorizontalBox', children: [{ node: 'anchor' }] },
        popups: { component: 'Overlay', children: [{ node: 'popup' }] },
        popup: {
          ...popup,
          penpot_review_source_path: popupSource,
          penpot_review_source_node_id: popupNodeId,
          penpot_review_instance_path: popupInstance,
        },
        anchor: {
          ...anchor,
          penpot_review_source_path: anchorSource,
          penpot_review_source_node_id: anchorNodeId,
          penpot_review_instance_path: anchorInstance,
        },
      },
    };
    const before = normalizeZuiDocument(source);
    const returned = reviewDocumentForCase(source, {
      ...reviewCase, host: 'editor', sourcePath: mainSource, state: 'focus-return',
      data: { workbenchState: {
        sourcePath: popupSource, sourceNodeId: popupNodeId,
        controlId: popup.control_id!, instancePath: popupInstance,
      } },
    });
    expect(returned.nodes?.['popup'].state).toMatchObject({ open: false, popup_open: false });
    expect(returned.nodes?.['anchor'].state).toMatchObject({ focused: true, focus_visible: true });
    expect(normalizeZuiDocument(source)).toEqual(before);
  });

  it('uses native painter empty data only as a transient component-state override', () => {
    const source: ZuiDocument = {
      asset: { kind: 'component', id: 'res://commands.zui', version: 2 },
      components: { Commands: { root: 'root' } },
      nodes: {
        root: {
          component: 'CommandPalette',
          props: {
            commands: ['build|label=Build'],
            filtered_commands: ['build'],
            recent_commands: ['build'],
            selected_command_id: 'build',
          },
        },
      },
    };
    const before = normalizeZuiDocument(source);
    const empty = reviewDocumentForCase(source, {
      ...reviewCase,
      state: 'empty',
    });
    expect(empty.nodes?.['root'].state).toMatchObject({
      commands: [],
      filtered_commands: [],
      recent_commands: [],
      selected_command_id: '',
    });
    expect(normalizeZuiDocument(source)).toEqual(before);
  });

  it.each(['CommandPalette', 'NotificationCenter'])(
    'applies the closed native painter state to %s',
    (component) => {
      const source: ZuiDocument = {
        asset: { kind: 'component', id: 'res://popup.zui', version: 2 },
        components: { Popup: { root: 'root' } },
        nodes: {
          root: {
            component,
            props: { open: true, popup_open: true },
          },
        },
      };
      const before = normalizeZuiDocument(source);
      const reviewed = reviewDocumentForCase(source, {
        ...reviewCase,
        state: 'closed',
      });
      expect(reviewed.nodes?.['root']?.state).toMatchObject({
        open: false,
        popup_open: false,
      });
      expect(normalizeZuiDocument(source)).toEqual(before);
    },
  );

  it('applies data-surface states without mutating authored rows or tree items', () => {
    const source: ZuiDocument = {
      asset: { kind: 'view', id: 'res://data-surface.zui', version: 2 },
      root: { node: 'root' },
      nodes: {
        root: {
          component: 'VerticalBox',
          children: [{ node: 'tree' }, { node: 'grid' }],
        },
        tree: {
          component: 'TreeView',
          props: {
            component_role: 'mui-x-tree-view',
            text: 'Folders',
            collection_items: ['expanded|0|Assets', 'selected|1|Meshes'],
            empty_text: 'No folders',
          },
        },
        grid: {
          component: 'DataGrid',
          props: {
            component_role: 'mui-x-data-grid',
            text: 'Assets',
            collection_items: ['selected|Tree.mesh|Mesh|Ready'],
            empty_text: 'No assets',
          },
        },
      },
    };
    const before = normalizeZuiDocument(source);
    const empty = reviewDocumentForCase(source, {
      ...reviewCase,
      state: 'empty',
    });
    expect(empty.nodes?.['tree']?.state).toMatchObject({ collection_items: [] });
    expect(empty.nodes?.['grid']?.state).toMatchObject({ collection_items: [] });
    const open = reviewDocumentForCase(source, {
      ...reviewCase,
      state: 'open',
    });
    expect(open.nodes?.['tree']?.state).toMatchObject({ expanded: true });
    const selected = reviewDocumentForCase(source, {
      ...reviewCase,
      state: 'selected',
    });
    expect(selected.nodes?.['tree']?.state).toMatchObject({ selected: true });
    expect(selected.nodes?.['grid']?.state).toMatchObject({ selected: true });
    expect(normalizeZuiDocument(source)).toEqual(before);
  });

  it('hides closed embedded and standalone dialogs while keeping source nodes addressable', () => {
    const view: ZuiDocument = {
      asset: { kind: 'view', id: 'res://showcase.zui', version: 2 },
      root: { node: 'host' },
      nodes: {
        host: {
          component: 'Overlay',
          children: [{ node: 'dialog' }, { node: 'confirm' }],
        },
        dialog: {
          component: 'Dialog',
          props: { open: true, popup_open: true, title: 'Scene Settings' },
        },
        confirm: {
          component: 'ConfirmDialog',
          props: { open: true, popup_open: true, title: 'Delete scene?' },
        },
      },
    };
    const standalone: ZuiDocument = {
      asset: { kind: 'component', id: 'res://dialog.zui', version: 2 },
      components: { Dialog: { root: 'dialog' } },
      nodes: {
        dialog: {
          component: 'Dialog',
          props: { open: true, popup_open: true, title: 'Scene Settings' },
        },
      },
    };
    const opened = reviewDocumentForCase(view, {
      ...reviewCase,
      state: 'open',
    });
    const closed = reviewDocumentForCase(view, {
      ...reviewCase,
      state: 'closed',
    });
    const standaloneClosed = reviewDocumentForCase(standalone, {
      ...reviewCase,
      state: 'closed',
    });

    expect(
      projectZuiDocument(opened).shapes.find(
        ({ nodeId }) => nodeId === 'dialog',
      )?.previewHidden,
    ).toBe(false);
    expect(
      projectZuiDocument(closed).shapes.find(
        ({ nodeId }) => nodeId === 'dialog',
      )?.previewHidden,
    ).toBe(true);
    expect(
      projectZuiDocument(closed).shapes.find(
        ({ nodeId }) => nodeId === 'confirm',
      )?.previewHidden,
    ).toBe(true);
    const standaloneProjection = projectZuiDocument(standaloneClosed);
    expect(
      standaloneProjection.shapes.find(({ nodeId }) => nodeId === 'dialog')
        ?.previewHidden,
    ).toBe(true);
    expect(
      cloneProjectionSnapshot(standaloneProjection).shapes.find(
        ({ nodeId }) => nodeId === 'dialog',
      ),
    ).toBeDefined();
  });

  it.each([
    ['Dialog', 'focused'],
    ['CommandPalette', 'empty'],
    ['NotificationCenter', 'selected'],
  ])(
    'opens %s for its %s review state without changing the product default',
    (component, state) => {
      const source: ZuiDocument = {
        asset: { kind: 'component', id: 'res://review/popup.zui', version: 2 },
        components: { Popup: { root: 'popup' } },
        nodes: {
          popup: {
            component,
            props: {
              open: false,
              popup_open: false,
              title: 'Review popup',
              ...(component === 'CommandPalette'
                ? { placeholder: 'Search commands' }
                : {}),
            },
          },
        },
      };
      const before = normalizeZuiDocument(source);
      const reviewed = reviewDocumentForCase(source, { ...reviewCase, state });
      expect(reviewed.nodes?.['popup']?.state).toMatchObject({
        open: true,
        popup_open: true,
      });
      expect(projectZuiDocument(reviewed).rootNodes[0].previewHidden).toBe(
        false,
      );
      expect(normalizeZuiDocument(source)).toEqual(before);
    },
  );

  it('honours keep-mounted native painter visibility without hiding static toast specimens', () => {
    const view: ZuiDocument = {
      asset: { kind: 'view', id: 'res://painter-visibility.zui', version: 2 },
      root: { node: 'host' },
      nodes: {
        host: {
          component: 'Overlay',
          children: [{ node: 'notifications' }, { node: 'toast' }],
        },
        notifications: {
          component: 'NotificationCenter',
          props: { open: false, popup_open: false, visibility: 'visible' },
        },
        toast: {
          component: 'WorkbenchToast',
          props: { text: 'Static specimen', severity: 'info' },
        },
      },
    };
    const projection = projectZuiDocument(view);
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'notifications')
        ?.previewHidden,
    ).toBe(true);
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'toast')?.previewHidden,
    ).toBe(false);
  });

  it('refuses unknown states, unhandled data and token-only theme states', () => {
    const { document } = parseZuiDocument(fixtureSource);
    expect(() =>
      reviewDocumentForCase(document, { ...reviewCase, state: 'drop-blocked' }),
    ).toThrow('native painter component');
    expect(() =>
      reviewDocumentForCase(document, {
        ...reviewCase,
        state: 'default',
        data: { title: 'Changed' },
      }),
    ).toThrow('business host adapter');
    expect(() =>
      reviewDocumentForCase(
        { asset: { kind: 'theme_tokens', id: 'theme', version: 2 } },
        reviewCase,
      ),
    ).toThrow('component consumer host');
  });
});

describe('actual managed scene fingerprint review adapter', () => {
  const owner = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
  const viewport = 'res://ui/editor/unreal/components/viewport.zui';
  const projectRoot = '\\\\?\\E:\\managed-editor-project';
  const cursor = { structure: 6, geometry: 6, interaction: 1, viewport: 1, hitTest: 6, theme: 2, diagnostics: 1 };
  type RecordValue = Record<string, unknown>;
  const evidence = (value: LayoutReviewCaseMessage) => value.data['normal_window_evidence'] as RecordValue;
  const publication = (value: LayoutReviewCaseMessage) => evidence(value)['workbenchStatePublication'] as RecordValue;
  const fingerprint = (value: LayoutReviewCaseMessage) => value.data['managedSceneFingerprint'] as RecordValue;
  const presentation = (value: LayoutReviewCaseMessage) => value.data['workbenchPresentation'] as RecordValue;
  function fixture() {
    // Reduced unit inputs retain the real R52 packet shape, nulls, device path,
    // source order and same-frame relationship. They are not native evidence.
    const document: ZuiDocument = {
      asset: { kind: 'view', id: 'res://ui/editor/unreal/level_editor_window.zui#review-host', version: 2 },
      root: { node: 'host' },
      nodes: {
        host: { component: 'Overlay', control_id: 'WorkbenchWindowRoot', children: [{ node: 'toolbar' }] },
        toolbar: { component: 'Overlay', control_id: 'UnrealViewportToolbar',
          props: { toolbar_overflow_row_control: 'UnrealViewportToolbarRow', toolbar_overflow_group_controls: ['SnapGroup'] },
          children: [{ node: 'row' }, { node: 'popup' }] },
        row: { component: 'HorizontalGroup', control_id: 'UnrealViewportToolbarRow',
          children: [{ node: 'group', slot: { kind: 'LinearSlot' } }, { node: 'more' }] },
        group: { component: 'HorizontalGroup', control_id: 'SnapGroup',
          children: [{ node: 'translate' }, { node: 'rotate' }, { node: 'scale' }] },
        popup: { component: 'ScrollableBox', control_id: 'UnrealViewportToolbarOverflow',
          props: { toolbar_overflow_role: 'toolbar-overflow', popup_open: false }, children: [] },
        more: { component: 'IconButton', control_id: 'UnrealViewportToolbarMore' },
        translate: { component: 'IconButton', control_id: 'ToggleTranslateSnap' },
        rotate: { component: 'IconButton', control_id: 'ToggleRotateSnap' },
        scale: { component: 'IconButton', control_id: 'ToggleScaleSnap' },
      },
    };
    const parents = new Map<string, string>();
    for (const [id, node] of Object.entries(document.nodes!)) {
      node['penpot_review_source_path'] = id === 'host' ? owner : viewport;
      node['penpot_review_source_node_id'] = id === 'host' ? 'root' : id;
      node['penpot_review_instance_path'] = '[]';
      for (const child of node.children ?? []) parents.set(child.node, id);
    }
    const nodes: ToolbarSourceNode[] = Object.entries(document.nodes!).filter(([id]) => id !== 'host').map(([id, node]) => {
      const parent = parents.get(id)!;
      const snap = ['translate', 'rotate', 'scale'].includes(id);
      const enabled = id === 'rotate';
      return {
        nodeId: id, parentNodeId: parent === 'host' ? 'native-host' : parent,
        sourcePath: viewport, sourceNodeId: id, instancePath: '[]',
        parentSourcePath: parent === 'host' ? owner : viewport,
        parentSourceNodeId: parent === 'host' ? 'root' : parent, parentInstancePath: '[]',
        controlId: node.control_id!, component: node.component, visible: !['more', 'popup'].includes(id),
        bounds: { x: 0, y: 0, width: 40, height: 35 }, clip: { x: 0, y: 0, width: 1600, height: 1000 },
        state: { selected: snap ? enabled : null, checked: snap ? enabled : null,
          popupOpen: id === 'popup' ? false : null, value: snap ? enabled : null,
          text: null, valueText: null, disabled: false }, scroll: null,
      };
    });
    const managedSceneFingerprint = { projectRoot, sources: [
      { sourcePath: 'zircon-project.toml', sha256: 'a'.repeat(64) },
      { sourcePath: 'assets/scenes/main.scene.toml', sha256: 'b'.repeat(64) },
      { sourcePath: 'assets/scenes/workbench-review-empty.scene.toml', sha256: 'c'.repeat(64) },
    ] };
    const workbenchPresentation = {
      schema: 'dev.zircon.editor.workbench-presentation', version: 1,
      sourceFingerprint: { sourcePath: owner, sha256: 'd'.repeat(64) },
      viewportSnap: { translationEnabled: false, rotationEnabled: true, scaleEnabled: false },
      status: { projectPath: projectRoot },
    };
    const actual: LayoutReviewCaseMessage = {
      id: 'actual-managed-unit', sourcePath: owner, host: 'editor',
      viewport: { width: 1280, height: 800 }, dpi: 1.25, locale: 'en-US', state: 'default',
      data: {
        workbenchState: { sourcePath: owner, sourceNodeId: 'root', controlId: 'WorkbenchWindowRoot' },
        managedSceneFingerprint, workbenchPresentation,
        normal_window_evidence: {
          presentationCursor: { ...cursor }, presentedFrameCount: 3,
          workbenchStatePublication: { authority: 'normal-retained-workbench-projection', observed: true,
            presentationCursor: { ...cursor }, presentedFrameCount: 3,
            managedSceneFingerprint: structuredClone(managedSceneFingerprint),
            workbenchPresentation: structuredClone(workbenchPresentation) },
          viewportToolbarPublication: { schema: 'dev.zircon.editor.viewport-toolbar-publication', version: 1,
            authority: 'normal-prepared-source-surface', observed: true, coordinateSpace: 'physical-pixels',
            surfaceGeneration: 1, surfaceTreeId: 'unit-normal-surface', rootNodeId: 'toolbar', rowNodeId: 'row',
            moreNodeId: 'more', popupNodeId: 'popup', groupNodeIds: ['group'], nodes,
            presentationCursor: { ...cursor }, presentedFrameCount: 3 },
        },
      },
    };
    document['penpot_actual_normal_review_case'] = actual;
    return { document, actual };
  }

  function sceneFixture() {
    const { document, actual } = fixture();
    const nativeWindow = 'res://ui/editor/unreal/level_editor_window.zui';
    const invocation = [{ sourcePath: nativeWindow, sourceNodeId: 'viewport' }];
    const prefix = [{ sourcePath: 'zircon_editor/assets/ui/editor/unreal/editor_shell.zui', sourceNodeId: 'pane_surface' }];
    document.nodes!['host'].children!.push({ node: 'viewport_root' });
    document.nodes!['viewport_root'] = { component: 'VerticalGroup', control_id: 'WorkbenchMainBandViewportPanel',
      penpot_review_source_path: viewport, penpot_review_source_node_id: 'root', children: [{ node: 'viewport' }] };
    document.nodes!['viewport'] = { component: 'Overlay', control_id: 'UnrealSceneViewViewport',
      penpot_review_source_path: viewport, penpot_review_source_node_id: 'viewport', children: [{ node: 'scene_media' }] };
    document.nodes!['scene_media'] = { component: 'SceneViewViewport', control_id: 'WorkbenchViewportSurface',
      penpot_review_source_path: viewport, penpot_review_source_node_id: 'scene_media',
      layout: { width: { stretch: 'Stretch' }, height: { stretch: 'Stretch' } },
      events: [{ id: 'ViewportToolbar/FrameSelection', event: 'Click', route: 'workbench.viewport.frame_selection' }] };
    for (const [id, node] of Object.entries(document.nodes!)) node['penpot_review_instance_path'] = JSON.stringify(
      ['viewport_root','viewport','scene_media'].includes(id) ? [...prefix, ...invocation.map((step) => ({ ...step, sourcePath: owner }))] : prefix);
    const parents = new Map<string, string>();
    for (const [id, node] of Object.entries(document.nodes!)) for (const child of node.children ?? []) parents.set(child.node, id);
    const ids = new Map(Object.keys(document.nodes!).map((id, index) => [id, index + 1]));
    const nativeIdentity = (id: string) => ({ nodeId: id === 'host' ? 'native-host' : id,
      sourcePath: id === 'host' ? nativeWindow : viewport, sourceNodeId: ['host','viewport_root'].includes(id) ? 'root' : id,
      instancePath: ['viewport_root','viewport','scene_media'].includes(id) ? invocation : [] });
    const rows = Object.entries(document.nodes!).map(([id, node]) => {
      const parent = parents.get(id), identity = nativeIdentity(id);
      return { ...identity, component: node.component, controlId: node.control_id, surfaceNodeId: ids.get(id),
        parent: parent ? nativeIdentity(parent) : null,
        projected: { ...identity, component: node.component, controlId: node.control_id, surfaceNodeId: ids.get(id), parentId: parent ? nativeIdentity(parent).nodeId : null } };
    });
    evidence(actual)['arrangedInventoryPublication'] = { schema: 'dev.zircon.editor.normal-arranged-inventory', version: 1,
      authority: 'normal-retained-runtime-surfaces', observed: true, complete: false, presentationCursor: { ...cursor }, presentedFrameCount: 3,
      surfaces: [{ schema: 'dev.zircon.editor.normal-surface-arranged-inventory', version: 1, scope: 'workbench-window',
        documentId: nativeWindow, coordinateSpace: 'surface-logical', frameAuthority: 'retained-host-projection', sourceFrameGeneration: 13,
        rootIds: [ids.get('host')], nodes: rows }] };
    const pngChunk = (kind: string, data: Buffer): Buffer => {
      const content = Buffer.concat([Buffer.from(kind),data]); let crc = 0xffffffff;
      for (const value of content) { crc ^= value; for (let bit = 0; bit < 8; bit++) crc = crc & 1 ? (crc >>> 1) ^ 0xedb88320 : crc >>> 1; }
      const chunk = Buffer.alloc(data.length + 12); chunk.writeUInt32BE(data.length); content.copy(chunk,4); chunk.writeUInt32BE((crc ^ 0xffffffff) >>> 0,chunk.length-4); return chunk;
    };
    const png = Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),pngChunk('IHDR',Buffer.from([0,0,0,1,0,0,0,1,8,6,0,0,0])),
      pngChunk('IDAT',deflateSync(Buffer.from([0,0,0,0,255]))),pngChunk('IEND',Buffer.alloc(0))]);
    const hash = (bytes: Buffer) => createHash('sha256').update(bytes).digest('hex');
    const rawHash = hash(Buffer.from([0,0,0,255])), pngHash = hash(png), ticket = { device_id: 1, generation: 1, queue_class: 'Graphics', sequence: 4 };
    const scene = { schema: 'dev.zircon.editor.normal-scene-image-readback', version: 1, authority: 'normal-actual-submitted-runtime-image-readback',
      owner: { sourcePath: viewport, sourceNodeId: 'scene_media', instancePath: JSON.stringify(invocation) },
      viewportRoot: { sourcePath: viewport, sourceNodeId: 'root', instancePath: JSON.stringify(invocation) },
      paneKind: 'Scene', surfaceKey: 'actual-unit-scene-surface', atlasUv: null, overlayPresent: true,
      commandIndex: 4, commandBounds: { x: 0, y: 50, width: 1, height: 1 }, commandClip: { x: 0, y: 50, width: 1, height: 1 },
      sourceRef: { authority: 'surface-node-owner', primitiveIdentity: false, treeId: `template.v2.${nativeWindow}`, generation: 11, nodeId: ids.get('scene_media'), nodeCommandIndex: 0, fragmentIndex: 0 },
      physicalSize: [1,1], resourceKey: 'viewport:unit', resourceGeneration: 1, readbackCompleted: true, failure: null,
      sourceFormat: 'Rgba8UnormSrgb', sourceByteLayout: 'rgba8', colorEncoding: 'srgb', alphaMode: 'opaque', opaquePixels: true,
      rawRowBytes: 4, rawByteLength: 4, rawSha256: rawHash, rgba8Sha256: rawHash, pngColorEncoding: 'srgb', pngByteLength: png.length, pngSha256: pngHash,
      conversion: { algorithm: 'identity-rgba8-srgb', inputSha256: rawHash, outputSha256: rawHash },
      receipt: { kind: 'Texture', terminal: 'Succeeded', request: 2, frame_key: { submission: ticket } } };
    const command = { commandIndex: scene.commandIndex,
      source: { ...scene.owner, controlId: 'WorkbenchViewportSurface', parentNodeId: 'viewport',
        parentSourcePath: viewport, parentSourceNodeId: 'viewport', parentInstancePath: JSON.stringify(invocation),
        sourceRef: structuredClone(scene.sourceRef) }, bounds: structuredClone(scene.commandBounds), clip: structuredClone(scene.commandClip),
      paint: { kind: 'image', resourceKey: scene.resourceKey, generation: scene.resourceGeneration, width: 1, height: 1,
        residentAfterSuccessfulPresent: true, hasUploadBytes: false } };
    // These Source/indices reproduce R57's unrelated brushes, not GPU proof.
    const dockImages = [[78,6,0,0],[81,9,0,0],[84,12,0,0],[94,6,1,1],[98,9,1,1],[102,12,1,1],[104,6,2,0],[106,9,2,0],[108,12,2,0]]
      .map(([commandIndex,nodeId,nodeCommandIndex,fragmentIndex], index) => ({ ...command, commandIndex,
        source: { sourcePath: null, sourceNodeId: null, instancePath: null, controlId: 'DockTab' + index % 3,
          sourceRef: { authority: 'surface-node-owner', primitiveIdentity: false, treeId: 'host.bottom.dock.header',
            generation: 3, nodeId, nodeCommandIndex, fragmentIndex } } }));
    evidence(actual)['commands'] = [command, ...dockImages, ...[284,286,288].map((commandIndex) => ({ ...command, commandIndex, source: null }))];
    const observed = { schema: 'dev.zircon.editor.normal-frame-pixel-association', version: 1, nonce: 'unit-nonce1',
      pixelFrameVerified: true, wholeUiAccepted: false, caseJsonSha256: '1'.repeat(64), caseSha256: '2'.repeat(64),
      normalReadbackReceiptSha256: '3'.repeat(64), normalSourceProfileSha256: '4'.repeat(64),
      normalReadbackReceipt: { schema: 'dev.zircon.editor.normal-frame-readback', version: 1, authority: 'normal-actual-gpu-packet-and-readback-receipt',
        nonce: 'unit-nonce1', caseSha256: '2'.repeat(64), gpuReadbackCompleted: true, failure: null, submission: ticket, receipt: { request: 1 }, sceneImage: scene } };
    evidence(actual)['runtimeSceneMedia'] = { schema: 'dev.zircon.editor.runtime-scene-media', version: 1,
      bootstrapCase: { path: 'E:/unit/bootstrap.json', caseJsonSha256: '1'.repeat(64), caseSha256: '2'.repeat(64) },
      bootstrapPixelAssociation: { path: 'E:/unit/association.json', sha256: '5'.repeat(64), observed },
      image: { mime: 'image/png', byteLength: png.length, pngSha256: pngHash, base64: png.toString('base64') } };
    return { document, actual, scene };
  }

  describe('actual portable scene bytes managed case admission', () => {
    beforeEach(() => { vi.stubGlobal('TextEncoder', undefined); vi.stubGlobal('crypto', undefined); });
    afterEach(() => vi.unstubAllGlobals());

    it('imports and reopens the complete scene case without changing its original metadata or bytes', () => {
      const { document, actual } = sceneFixture(), metadata = encodeZuiMetadata(document), originalCase = JSON.stringify(actual);
      const target = runtimeSceneMediaReviewTarget(document, actual)!;
      expect(reviewDocumentForCase(document, actual).nodes!['scene_media']['penpot_image']).toEqual(target.image);
      const reopened = decodeZuiMetadata(metadata), saved = reviewCaseFromSavedMetadata(reopened, originalCase)!;
      expect(runtimeSceneMediaForCase(saved)).toEqual(evidence(actual)['runtimeSceneMedia']);
      expect(reviewDocumentForCase(reopened, saved).nodes!['scene_media']['penpot_image']).toEqual(target.image);
      expect(encodeZuiMetadata(document)).toBe(metadata);
      expect(encodeZuiMetadata(reopened)).toBe(metadata);
      expect(JSON.stringify(actual)).toBe(originalCase);
    });

    it('accepts exactly 8 MiB of serialized UTF-8 case data and rejects one extra byte despite a shorter UTF-16 length', () => {
      const { actual } = sceneFixture(), budget = 8 * 1024 * 1024;
      actual.id = '';
      const available = budget - Buffer.byteLength(JSON.stringify(actual), 'utf8');
      actual.id = '\u4e2d'.repeat(Math.floor((available - 4) / 3)) + '\ud83d\ude80';
      actual.id += 'a'.repeat(budget - Buffer.byteLength(JSON.stringify(actual), 'utf8'));
      const original = JSON.stringify(actual);
      expect(Buffer.byteLength(original, 'utf8')).toBe(budget);
      expect(original.length).toBeLessThan(budget);
      expect(runtimeSceneMediaForCase(actual)).toBe(evidence(actual)['runtimeSceneMedia']);
      expect(JSON.stringify(actual)).toBe(original);
      actual.id += 'a';
      expect(Buffer.byteLength(JSON.stringify(actual), 'utf8')).toBe(budget + 1);
      expect(() => runtimeSceneMediaForCase(actual)).toThrow('complete case exceeds 8 MiB');
    });
  });

  it('actual scene command selection preserves unrelated R57 DockTab and source-null image commands', () => {
    const { document, actual } = sceneFixture(), before = JSON.stringify(actual);
    const commands = evidence(actual)['commands'] as RecordValue[];
    expect(commands.filter((command) => {
      const reference = (command['source'] as RecordValue | null)?.['sourceRef'] as RecordValue | undefined;
      return reference?.['treeId'] === 'host.bottom.dock.header';
    }).map((command) => command['commandIndex'])).toEqual([78,81,84,94,98,102,104,106,108]);
    expect(commands.filter((command) => command['source'] === null).map((command) => command['commandIndex'])).toEqual([284,286,288]);
    expect(runtimeSceneMediaReviewTarget(document, actual)?.nodeId).toBe('scene_media');
    expect(reviewDocumentForCase(document, actual).nodes!['scene_media']['penpot_image']).toBeDefined();
    expect(JSON.stringify(actual)).toBe(before);
  });

  it.each(['missing invocation','null invocation','malformed invocation','partial invocation step','wrong alias','wrong node','wrong invocation',
    'partial candidate beside target','duplicate owner'])(
    'actual scene command selection rejects %s despite unrelated images', (kind) => {
      const { document, actual } = sceneFixture(), commands = evidence(actual)['commands'] as RecordValue[];
      const source = commands[0]!['source'] as RecordValue;
      if (kind === 'missing invocation') delete source['instancePath'];
      if (kind === 'null invocation') source['instancePath'] = null;
      if (kind === 'malformed invocation') source['instancePath'] = '[';
      if (kind === 'partial invocation step') source['instancePath'] = JSON.stringify([{ sourcePath: 'res://ui/editor/unreal/level_editor_window.zui' }]);
      if (kind === 'wrong alias') source['sourcePath'] = 'res://other/editor/unreal/components/viewport.zui';
      if (kind === 'wrong node') source['sourceNodeId'] = 'toolbar';
      if (kind === 'wrong invocation') source['instancePath'] = '[]';
      if (kind === 'partial candidate beside target' || kind === 'duplicate owner') {
        const extra = structuredClone(commands[0]!); extra['commandIndex'] = 5;
        const extraSource = extra['source'] as RecordValue;
        extraSource['sourcePath'] = 'zircon_editor/assets/ui/editor/unreal/components/viewport.zui';
        if (kind === 'partial candidate beside target') delete extraSource['instancePath'];
        commands.push(extra);
      }
      expect(() => reviewDocumentForCase(document, actual)).toThrow();
    },
  );

  it('actual scene root identity joins cached generation 11 within arranged generation 13 and survives saved reopen without changing source/case', () => {
    const { document, actual } = sceneFixture(), before = encodeZuiMetadata(document), originalCase = JSON.stringify(actual);
    const target = runtimeSceneMediaReviewTarget(document, actual)!;
    expect(target.nodeId).toBe('scene_media');
    const reviewed = reviewDocumentForCase(document, actual);
    expect(reviewed.nodes!['scene_media']['penpot_image']).toEqual(target.image);
    expect(reviewed.nodes!['viewport']).not.toHaveProperty('penpot_image');
    expect(reviewed.nodes!['viewport_root']).not.toHaveProperty('penpot_image');
    expect(document.nodes!['viewport_root']).toMatchObject({ component: 'VerticalGroup', control_id: 'WorkbenchMainBandViewportPanel' });
    expect(reviewed.nodes!['scene_media'].layout).toEqual(document.nodes!['scene_media'].layout);
    expect(reviewed.nodes!['scene_media'].events).toEqual(document.nodes!['scene_media'].events);
    expect(encodeZuiMetadata(document)).toBe(before);
    expect(JSON.stringify(actual)).toBe(originalCase);
    const reopened = decodeZuiMetadata(before), saved = reviewCaseFromSavedMetadata(reopened, originalCase)!;
    expect(runtimeSceneMediaForCase(saved)).toEqual(evidence(actual)['runtimeSceneMedia']);
    const reopenedReview = reviewDocumentForCase(reopened,saved);
    expect(reopenedReview.nodes!['scene_media']['penpot_image']).toEqual(target.image);
    expect(reopenedReview.nodes!['scene_media'].events).toEqual(document.nodes!['scene_media'].events);
  });

  it('actual scene root identity accepts only the documented canonical source aliases across the complete invocation', () => {
    const { document, actual, scene } = sceneFixture();
    const canonical = (path: string) => path.startsWith('res://ui/editor/') ? 'zircon_editor/assets/' + path.slice(6) : path;
    const surface = ((evidence(actual)['arrangedInventoryPublication'] as RecordValue)['surfaces'] as RecordValue[])[0]!;
    for (const row of surface['nodes'] as RecordValue[]) for (const value of [row, row['projected'] as RecordValue]) {
      value['sourcePath'] = canonical(value['sourcePath'] as string);
      const rawSteps = value['instancePath'];
      const steps = (typeof rawSteps === 'string' ? JSON.parse(rawSteps) : rawSteps) as Array<{ sourcePath: string; sourceNodeId: string }>;
      value['instancePath'] = JSON.stringify(steps.map((step) => ({ ...step, sourcePath: canonical(step.sourcePath) })));
    }
    for (const id of ['viewport_root','viewport','scene_media']) document.nodes![id]['penpot_review_source_path'] = canonical(viewport);
    const originalCase = JSON.stringify(actual), originalMetadata = encodeZuiMetadata(document);
    expect(runtimeSceneMediaReviewTarget(document, actual)?.nodeId).toBe('scene_media');
    expect(scene.viewportRoot.sourceNodeId).toBe('root');
    expect(JSON.stringify(actual)).toBe(originalCase);
    expect(encodeZuiMetadata(document)).toBe(originalMetadata);
  });

  it.each(['root is inner viewport','root invocation','root control','root kind','viewport control','viewport kind',
    'media control','media kind','native root control','native root kind','native viewport control','native viewport kind',
    'native media control','native media kind','projected control','viewport skips root','native viewport skips root','source suffix','invocation alias'])(
    'actual scene root identity rejects %s', (kind) => {
      const { document, actual, scene } = sceneFixture();
      const surface = ((evidence(actual)['arrangedInventoryPublication'] as RecordValue)['surfaces'] as RecordValue[])[0]!;
      const rows = surface['nodes'] as RecordValue[];
      const native = (id: string) => rows.find((row) => row['nodeId'] === id)!;
      if (kind === 'root is inner viewport') scene.viewportRoot.sourceNodeId = 'viewport';
      if (kind === 'root invocation') scene.viewportRoot.instancePath = '[]';
      if (kind === 'root control') document.nodes!['viewport_root'].control_id = 'UnrealSceneViewViewport';
      if (kind === 'root kind') document.nodes!['viewport_root'].component = 'Overlay';
      if (kind === 'viewport control') document.nodes!['viewport'].control_id = 'WorkbenchMainBandViewportPanel';
      if (kind === 'viewport kind') document.nodes!['viewport'].component = 'VerticalGroup';
      if (kind === 'media control') document.nodes!['scene_media'].control_id = 'another';
      if (kind === 'media kind') document.nodes!['scene_media'].component = 'Overlay';
      if (kind === 'native root control') native('viewport_root')['controlId'] = 'UnrealSceneViewViewport';
      if (kind === 'native root kind') native('viewport_root')['component'] = 'Overlay';
      if (kind === 'native viewport control') native('viewport')['controlId'] = 'WorkbenchMainBandViewportPanel';
      if (kind === 'native viewport kind') native('viewport')['component'] = 'VerticalGroup';
      if (kind === 'native media control') native('scene_media')['controlId'] = 'another';
      if (kind === 'native media kind') native('scene_media')['component'] = 'Overlay';
      if (kind === 'projected control') (native('viewport')['projected'] as RecordValue)['controlId'] = 'another';
      if (kind === 'viewport skips root') {
        document.nodes!['host'].children = document.nodes!['host'].children!.map((child) => child.node === 'viewport_root' ? { node: 'viewport' } : child);
        document.nodes!['viewport_root'].children = [];
      }
      if (kind === 'native viewport skips root') {
        native('viewport')['parent'] = native('native-host');
        (native('viewport')['projected'] as RecordValue)['parentId'] = 'native-host';
      }
      if (kind === 'source suffix') scene.viewportRoot.sourcePath = 'foreign/ui/editor/unreal/components/viewport.zui';
      if (kind === 'invocation alias') scene.viewportRoot.instancePath = scene.viewportRoot.instancePath.replace('res://ui/editor/', 'res://other/editor/');
      expect(() => reviewDocumentForCase(document,actual)).toThrow();
    },
  );

  it.each(['missing receipt','premultiplied','wrong PNG','wrong owner','similar invocation','outside owner','duplicate target','stale inventory','wrong surface node','view conversion mismatch','unknown format','partial command','duplicate actual id','wrong root invocation',
    'missing commands','duplicate command index','receipt generation rollback','command generation rollback','foreign matching tree',
    'future matching source frame','wrong physical owner','wrong physical parent','wrong full parent','wrong command index','partial submitted reference'])(
    'rejects runtime scene %s instead of uploading a guessed viewport image', (kind) => {
      const { document, actual, scene } = sceneFixture();
      if (kind === 'missing receipt') delete (scene as RecordValue)['receipt'];
      if (kind === 'premultiplied') scene.alphaMode = 'premultiplied';
      if (kind === 'wrong PNG') (evidence(actual)['runtimeSceneMedia'] as RecordValue)['image'] = { mime: 'image/png', byteLength: 1, pngSha256: 'a'.repeat(64), base64: 'AA==' };
      if (kind === 'wrong owner') scene.owner.sourceNodeId = 'toolbar';
      if (kind === 'similar invocation') document.nodes!['scene_media']['penpot_review_instance_path'] = JSON.stringify([{ sourcePath: owner, sourceNodeId: 'viewport_extra' }]);
      if (kind === 'outside owner') document.nodes!['host'].children = [{ node: 'toolbar' }];
      if (kind === 'duplicate target') document.nodes!['duplicate_scene'] = structuredClone(document.nodes!['scene_media']);
      if (kind === 'stale inventory') (evidence(actual)['arrangedInventoryPublication'] as RecordValue)['presentedFrameCount'] = 2;
      if (kind === 'wrong surface node') scene.sourceRef.nodeId = 999;
      if (kind === 'view conversion mismatch') scene.colorEncoding = 'linear';
      if (kind === 'unknown format') scene.sourceFormat = 'constructor';
      if (kind === 'partial command') delete (scene as RecordValue)['commandBounds'];
      const commands = evidence(actual)['commands'] as RecordValue[];
      const command = commands[0]!, source = command['source'] as RecordValue;
      const reference = source['sourceRef'] as RecordValue;
      if (kind === 'missing commands') delete evidence(actual)['commands'];
      if (kind === 'duplicate command index') commands.push(structuredClone(command));
      if (kind === 'receipt generation rollback') scene.sourceRef.generation = 10;
      if (kind === 'command generation rollback') reference['generation'] = 10;
      if (kind === 'foreign matching tree') { scene.sourceRef.treeId = 'template.v2.other'; reference['treeId'] = scene.sourceRef.treeId; }
      if (kind === 'future matching source frame') { scene.sourceRef.generation = 14; reference['generation'] = 14; }
      if (kind === 'wrong physical owner') { scene.sourceRef.nodeId = 999; reference['nodeId'] = 999; }
      if (kind === 'wrong physical parent') source['parentNodeId'] = 'other';
      if (kind === 'wrong full parent') source['parentInstancePath'] = '[]';
      if (kind === 'wrong command index') command['commandIndex'] = 5;
      if (kind === 'partial submitted reference') delete reference['fragmentIndex'];
      if (kind === 'duplicate actual id') {
        const surface = ((evidence(actual)['arrangedInventoryPublication'] as RecordValue)['surfaces'] as RecordValue[])[0]!;
        const rows = surface['nodes'] as RecordValue[]; rows[1]!['surfaceNodeId'] = rows[0]!['surfaceNodeId'];
      }
      if (kind === 'wrong root invocation') {
        const surface = ((evidence(actual)['arrangedInventoryPublication'] as RecordValue)['surfaces'] as RecordValue[])[0]!;
        (surface['nodes'] as RecordValue[]).find((row) => row['controlId'] === 'WorkbenchWindowRoot')!['instancePath'] =
          JSON.stringify([{ sourcePath: owner, sourceNodeId: 'another' }]);
      }
      expect(() => reviewDocumentForCase(document,actual)).toThrow();
    },
  );

  it('retains the complete actual imported case and saved reopen metadata while applying its business state', () => {
    const { document, actual } = fixture();
    const originalCase = JSON.stringify(actual);
    const wrapper: ZuiDocument = {
      asset: { kind: 'view', id: 'res://ui/editor/unreal/level_editor_window.zui', version: 2 },
      root: { node: 'root' }, nodes: { root: { component: 'Overlay' } }, penpot_review_host: document,
    };
    const serialized = serializeZuiDocument(wrapper);
    const imported = reviewHost(parseZuiDocument(serialized).document)!;
    const metadata = encodeZuiMetadata(imported);
    const importedCase = importedActualNormalReviewCase(imported)!;
    expect(importedCase).toEqual(actual);
    const reviewed = reviewDocumentForCase(imported, importedCase);
    expect(reviewed.nodes!['rotate'].props?.['checked']).toBe(true);
    expect(reviewed.nodes!['translate'].props?.['checked']).toBe(false);
    expect(encodeZuiMetadata(imported)).toBe(metadata);
    const savedCaseMetadata = JSON.stringify(importedCase);
    expect(savedCaseMetadata).toBe(originalCase);
    const reopened = decodeZuiMetadata(metadata);
    const saved = reviewCaseFromSavedMetadata(reopened, savedCaseMetadata)!;
    expect(saved).toEqual(actual);
    expect(reviewDocumentForCase(reopened, saved).nodes!['scale'].props?.['checked']).toBe(false);
    expect(encodeZuiMetadata(reopened)).toBe(metadata);
    expect(JSON.stringify(actual)).toBe(originalCase);
  });

  const invalid: Array<[string, (value: LayoutReviewCaseMessage) => void]> = [
    ['orphan fingerprint', (value) => { value.data = { managedSceneFingerprint: fingerprint(value) }; }],
    ['noneditor host', (value) => { value.host = 'component'; delete value.data['workbenchState']; }],
    ['wrong Source owner', (value) => { value.sourcePath = 'different-owner.zui'; }],
    ['wrong presentation owner', (value) => {
      (presentation(value)['sourceFingerprint'] as RecordValue)['sourcePath'] = 'different-owner.zui';
    }],
    ['wrong source hash', (value) => { (presentation(value)['sourceFingerprint'] as RecordValue)['sha256'] = 'invalid'; }],
    ['unobserved publication', (value) => { publication(value)['observed'] = false; }],
    ['wrong publication authority', (value) => { publication(value)['authority'] = 'fixture'; }],
    ['stale business frame', (value) => { publication(value)['presentedFrameCount'] = 2; }],
    ['missing normal frame', (value) => { delete evidence(value)['presentedFrameCount']; }],
    ['zero normal and business frame', (value) => {
      evidence(value)['presentedFrameCount'] = 0; publication(value)['presentedFrameCount'] = 0;
    }],
    ['changed fingerprint', (value) => {
      (fingerprint(value)['sources'] as RecordValue[])[0]['sha256'] = 'e'.repeat(64);
    }],
    ['changed publication fingerprint', (value) => {
      (publication(value)['managedSceneFingerprint'] as RecordValue)['projectRoot'] = 'E:/other-project';
    }],
    ['relative project root on both sides', (value) => {
      fingerprint(value)['projectRoot'] = 'relative-project';
      publication(value)['managedSceneFingerprint'] = structuredClone(fingerprint(value));
    }],
    ['reordered sources on both sides', (value) => {
      (fingerprint(value)['sources'] as RecordValue[]).reverse();
      publication(value)['managedSceneFingerprint'] = structuredClone(fingerprint(value));
    }],
    ['invalid source hash on both sides', (value) => {
      (fingerprint(value)['sources'] as RecordValue[])[0]['sha256'] = 'invalid';
      publication(value)['managedSceneFingerprint'] = structuredClone(fingerprint(value));
    }],
    ['unbound presentation', (value) => { (presentation(value)['status'] as RecordValue)['projectPath'] = 'E:/other-project'; }],
    ['different bound project root', (value) => {
      fingerprint(value)['projectRoot'] = 'E:/other-project';
      publication(value)['managedSceneFingerprint'] = structuredClone(fingerprint(value));
    }],
    ['unknown fingerprint field on both sides', (value) => {
      fingerprint(value)['extra'] = true;
      publication(value)['managedSceneFingerprint'] = structuredClone(fingerprint(value));
    }],
    ['unknown source field on both sides', (value) => {
      (fingerprint(value)['sources'] as RecordValue[])[0]['extra'] = true;
      publication(value)['managedSceneFingerprint'] = structuredClone(fingerprint(value));
    }],
    ['unknown business field', (value) => { publication(value)['extra'] = true; }],
    ['unknown normal evidence field', (value) => { evidence(value)['extra'] = true; }],
    ['missing canonical toolbar publication', (value) => { delete evidence(value)['viewportToolbarPublication']; }],
    ['unknown review data', (value) => { value.data['extra'] = true; }],
  ];
  for (const key of Object.keys(cursor)) {
    invalid.push(['different business cursor ' + key, (value) => {
      (publication(value)['presentationCursor'] as RecordValue)[key] = 99;
    }]);
    invalid.push(['missing cursor ' + key + ' on both sides', (value) => {
      delete (evidence(value)['presentationCursor'] as RecordValue)[key];
      delete (publication(value)['presentationCursor'] as RecordValue)[key];
    }]);
  }
  it.each(invalid)('rejects %s at review, actual import and saved reopen', (_name, mutate) => {
    const { document, actual } = fixture();
    const bad = structuredClone(actual);
    mutate(bad);
    expect(() => reviewDocumentForCase(document, bad)).toThrow();
    expect(() => importedActualNormalReviewCase({ ...document, penpot_actual_normal_review_case: bad })).toThrow();
    expect(() => reviewCaseFromSavedMetadata(document, JSON.stringify(bad))).toThrow();
  });

  it('rejects saved evidence replaced with a self-consistent different managed fingerprint', () => {
    const { document, actual } = fixture();
    const changed = structuredClone(actual);
    (fingerprint(changed)['sources'] as RecordValue[])[0]['sha256'] = 'e'.repeat(64);
    publication(changed)['managedSceneFingerprint'] = structuredClone(fingerprint(changed));
    expect(() => importedActualNormalReviewCase({ ...document, penpot_actual_normal_review_case: changed }))
      .not.toThrow();
    expect(() => reviewCaseFromSavedMetadata(document, JSON.stringify(changed)))
      .toThrow('differs from its imported managed evidence');
    const missing = structuredClone(actual);
    delete missing.data['managedSceneFingerprint'];
    expect(() => reviewCaseFromSavedMetadata(document, JSON.stringify(missing)))
      .toThrow('differs from its imported managed evidence');
  });
});

describe('authored Workbench button painter states', () => {
  const root = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../../', import.meta.url)));
  const assetPath =
    'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/button.zui';
  const iconButtonPath =
    'zircon_editor/assets/ui/editor/workbench/primitives/inputs/icon_button.zui';
  const themePath = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
  const dependencies = new LayoutDependencies();
  beforeAll(async () => {
    await dependencies.load(root, [assetPath, iconButtonPath, themePath]);
  });
  const authoredButton = (): ZuiDocument => {
    const { document } = parseZuiDocument(
      readFileSync(`${root}/${assetPath}`, 'utf8'),
    );
    dependencies.embed(document, assetPath);
    return document;
  };
  const authoredIconButton = (): ZuiNode => {
    const { document } = parseZuiDocument(
      readFileSync(`${root}/${iconButtonPath}`, 'utf8'),
    );
    dependencies.embed(document, iconButtonPath);
    return document.nodes!['root'];
  };

  it.each([
    ['default', '#242424', '#383838', '#C0C0C0'],
    ['hover', '#575757', '#383838', '#C0C0C0'],
    ['pressed', '#383838', '#0070E0', '#C0C0C0'],
    ['selected', '#0070E0', '#0070E0', '#C0C0C0'],
    ['focused', '#242424', '#0070E0', '#C0C0C0'],
    ['disabled', '#242424', '#383838', '#808080'],
  ])(
    'paints %s from authored runtime properties without selector rules',
    (state, fill, stroke, text) => {
      const source = authoredButton();
      const before = normalizeZuiDocument(source);
      const review = reviewDocumentForCase(source, { ...reviewCase, state });
      const projection = projectZuiDocument(review);
      expect(projection.rootNodes[0].paint).toMatchObject({
        fillColor: fill,
        strokeColor: stroke,
      });
      expect(projection.rootNodes[0].text?.color).toBe(text);
      expect(normalizeZuiDocument(source)).toEqual(before);
      expect(
        normalizeZuiDocument(
          reconcileZuiDocument(source, cloneProjectionSnapshot(projection))
            .document,
        ),
      ).toEqual(before);
    },
  );

  it.each([
    [{ hovered: true, selected: true }, '#0070E0', '#0070E0'],
    [{ hovered: true, selected: true, pressed: true }, '#383838', '#0070E0'],
    [
      { hovered: true, selected: true, pressed: true, disabled: true },
      '#242424',
      '#383838',
    ],
    [
      { selected: true, pressed: true, loading: true, focus_visible: true },
      '#242424',
      '#383838',
    ],
    [{ focused: true, focus_visible: false }, '#242424', '#383838'],
    [{ hovered: true, focus_visible: true }, '#575757', '#0070E0'],
    [{ checked: true }, '#0070E0', '#0070E0'],
  ])(
    'resolves overlapping state %j using runtime precedence',
    (state, fill, stroke) => {
      const document = authoredButton();
      Object.assign(document.nodes!['root'].props!, state);
      const shape = projectZuiDocument(document).rootNodes[0];
      expect(shape.paint).toMatchObject({
        fillColor: fill,
        strokeColor: stroke,
      });
      if (
        ('loading' in state && state.loading) ||
        ('disabled' in state && state.disabled)
      )
        expect(shape.text?.color).toBe('#808080');
    },
  );

  it('uses focus_border_color and disabled_foreground_color rather than similarly named unused props', () => {
    const document = authoredButton();
    Object.assign(document.nodes!['root'].props!, {
      selected_border_color: '#ff0000',
      selected_foreground_color: '#ff0000',
      disabled_text_color: '#ff0000',
    });
    const selected = projectZuiDocument(
      reviewDocumentForCase(document, { ...reviewCase, state: 'selected' }),
    ).rootNodes[0];
    expect(selected.paint.strokeColor).toBe('#0070E0');
    expect(selected.text?.color).toBe('#C0C0C0');
    const disabled = projectZuiDocument(
      reviewDocumentForCase(document, { ...reviewCase, state: 'disabled' }),
    ).rootNodes[0];
    expect(disabled.text?.color).toBe('#808080');
  });

  it.each([
    [
      { focused: true, focus_visible: false, focusVisible: true },
      '#242424',
      '#383838',
    ],
    [{ open: false, popup_open: true }, '#242424', '#383838'],
    [
      { pressed: true, drop_hovered: false, active_drag_target: true },
      '#383838',
      '#0070E0',
    ],
    [
      { focused: true, focus_visible: 'false', focusVisible: true },
      '#242424',
      '#0070E0',
    ],
    [
      { focused: true, focus_visible: 0, focusVisible: 'false' },
      '#242424',
      '#0070E0',
    ],
    [{ open: 'false', popup_open: true }, '#575757', '#383838'],
    [
      { pressed: true, drop_hovered: 0, active_drag_target: true },
      '#575757',
      '#383838',
    ],
    [{ enabled: false, hovered: true }, '#242424', '#383838'],
    [{ enabled: 'false', hovered: true }, '#575757', '#383838'],
  ])('uses the first boolean metadata alias for %j', (state, fill, stroke) => {
    const document = authoredButton();
    Object.assign(document.nodes!['root'].props!, state);
    expect(projectZuiDocument(document).rootNodes[0].paint).toMatchObject({
      fillColor: fill,
      strokeColor: stroke,
    });
  });

  it.each(['disabled', 'loading', 'selected', 'pressed', 'hovered', 'checked'])(
    'lets authored state false override prop %s true',
    (key) => {
      const document = authoredButton();
      const node = document.nodes!['root'];
      node.props![key] = true;
      node.state = { [key]: false };
      expect(projectZuiDocument(document).rootNodes[0].paint).toMatchObject({
        fillColor: '#242424',
        strokeColor: '#383838',
      });
    },
  );

  it('uses authored focused state as static metadata, and resets enabled for a requested case', () => {
    const document = authoredButton();
    const node = document.nodes!['root'];
    node.props!['enabled'] = false;
    node.state = { enabled: true, focused: true };
    expect(projectZuiDocument(document).rootNodes[0].paint.strokeColor).toBe(
      '#0070E0',
    );
    node.state['enabled'] = false;
    expect(
      projectZuiDocument(reviewDocumentForCase(document, reviewCase))
        .rootNodes[0].paint.fillColor,
    ).toBe('#575757');
  });

  it('applies stylesheet state palette overrides before the painter selects its color', () => {
    const document = authoredButton();
    document.stylesheets = [
      {
        rules: [
          {
            selector: 'Button:hover',
            set: { self: { hover_background_color: '#123456' } },
          },
        ],
      },
    ];
    const shape = projectZuiDocument(
      reviewDocumentForCase(document, reviewCase),
    ).rootNodes[0];
    expect(shape.paint.fillColor).toBe('#123456');
  });

  it('preserves business state and events while isolating the requested transient state', () => {
    const document = authoredButton();
    const node = document.nodes!['root'];
    node.state = {
      loading: true,
      checked: true,
      open: true,
      counter: 7,
      binding: '$session.selection',
    };
    node.events = [
      { id: 'Test/Button', event: 'Click', route: 'test.button.click' },
    ];
    const before = normalizeZuiDocument(document);
    const review = reviewDocumentForCase(document, reviewCase);
    expect(review.nodes!['root'].state).toEqual({
      counter: 7,
      binding: '$session.selection',
      hovered: true,
    });
    expect(review.nodes!['root'].events).toEqual(node.events);
    expect(projectZuiDocument(review).rootNodes[0].paint.fillColor).toBe(
      '#575757',
    );
    expect(normalizeZuiDocument(document)).toEqual(before);
  });

  it('retains the runtime icon selection-before-press exception and disabled icon foreground', () => {
    const source = authoredIconButton();
    expect(source.component).toBe('IconButton');
    Object.assign(source.props!, {
      selected: true,
      pressed: true,
      icon_color: '#111111',
      selected_icon_color: '#222222',
    });
    const selected = buttonStatePreviewNode(source);
    expect(selected.props!['background_color']).toBe(
      '$editor.surface.selected',
    );
    expect(selected.props!['icon_color']).toBe('#222222');
    source.props!['loading'] = true;
    expect(buttonStatePreviewNode(source).props!['icon_color']).toBe(
      '$editor.text.disabled',
    );
  });

  it('keeps runtime secondary hover fallback for icon buttons regardless of button kind', () => {
    const node = authoredButton().nodes!['root'];
    node.component = 'IconButton';
    Object.assign(node.props!, { hovered: true, button_color: 'primary' });
    delete node.props!['hover_background_color'];
    expect(buttonStatePreviewNode(node).props!['background_color']).toBe(
      '#454545',
    );
  });
});
