import type { LayoutReviewCaseMessage } from '../model';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { LayoutDependencies } from '../../tools/zui-layout-dependencies';
import fixtureSource from './roundtrip-fixture.zui?raw';
import {
  parseZuiDocument,
  normalizeZuiDocument,
  type ZuiDocument,
  type ZuiNode,
} from './zui-document';
import {
  projectZuiDocument,
  cloneProjectionSnapshot,
  reconcileZuiDocument,
} from './penpot-projection';
import {
  reviewBoardNeedsResponsiveRebuild,
  reviewDocumentForCase,
} from './zui-review-case';
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
            'zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui',
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
    const responsiveNode = (props: Record<string, unknown>): ZuiNode => ({
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

  it('returns focus to the invoking control in the popup source instance', () => {
    const sourcePath =
      'zircon_editor/assets/ui/editor/windows/workbench_window.zui';
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
    const returned = reviewDocumentForCase(source, {
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
    });

    expect(returned.nodes?.['trigger_a'].state).toBeUndefined();
    expect(returned.nodes?.['trigger_b'].state).toMatchObject({
      focused: true,
      focus_visible: true,
    });
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
      expect(reviewed.nodes?.root?.state).toMatchObject({
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
    expect(empty.nodes?.tree?.state).toMatchObject({ collection_items: [] });
    expect(empty.nodes?.grid?.state).toMatchObject({ collection_items: [] });
    const open = reviewDocumentForCase(source, {
      ...reviewCase,
      state: 'open',
    });
    expect(open.nodes?.tree?.state).toMatchObject({ expanded: true });
    const selected = reviewDocumentForCase(source, {
      ...reviewCase,
      state: 'selected',
    });
    expect(selected.nodes?.tree?.state).toMatchObject({ selected: true });
    expect(selected.nodes?.grid?.state).toMatchObject({ selected: true });
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
      expect(reviewed.nodes?.popup?.state).toMatchObject({
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

describe('authored Workbench button painter states', () => {
  const root = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../../', import.meta.url)));
  const assetPath =
    'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_button.zui';
  const themePath = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
  const dependencies = new LayoutDependencies();
  beforeAll(async () => {
    await dependencies.load(root, [assetPath, themePath]);
  });
  const authoredButton = (): ZuiDocument => {
    const { document } = parseZuiDocument(
      readFileSync(`${root}/${assetPath}`, 'utf8'),
    );
    dependencies.embed(document, assetPath);
    return document;
  };

  it.each([
    ['default', '#242424', '#484848', '#e8e8e8'],
    ['hover', '#454545', '#484848', '#e8e8e8'],
    ['pressed', '#383838', '#66b2ff', '#e8e8e8'],
    ['selected', '#243f5a', '#66b2ff', '#e8e8e8'],
    ['focused', '#242424', '#66b2ff', '#e8e8e8'],
    ['disabled', '#2b2b2b', '#363636', '#737373'],
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
    [{ hovered: true, selected: true }, '#243f5a', '#66b2ff'],
    [{ hovered: true, selected: true, pressed: true }, '#383838', '#66b2ff'],
    [
      { hovered: true, selected: true, pressed: true, disabled: true },
      '#2b2b2b',
      '#363636',
    ],
    [
      { selected: true, pressed: true, loading: true, focus_visible: true },
      '#2b2b2b',
      '#363636',
    ],
    [{ focused: true, focus_visible: false }, '#242424', '#484848'],
    [{ hovered: true, focus_visible: true }, '#454545', '#66b2ff'],
    [{ checked: true }, '#243f5a', '#66b2ff'],
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
        expect(shape.text?.color).toBe('#737373');
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
    expect(selected.paint.strokeColor).toBe('#66b2ff');
    expect(selected.text?.color).toBe('#e8e8e8');
    const disabled = projectZuiDocument(
      reviewDocumentForCase(document, { ...reviewCase, state: 'disabled' }),
    ).rootNodes[0];
    expect(disabled.text?.color).toBe('#737373');
  });

  it.each([
    [
      { focused: true, focus_visible: false, focusVisible: true },
      '#242424',
      '#484848',
    ],
    [{ open: false, popup_open: true }, '#242424', '#484848'],
    [
      { pressed: true, drop_hovered: false, active_drag_target: true },
      '#383838',
      '#66b2ff',
    ],
    [
      { focused: true, focus_visible: 'false', focusVisible: true },
      '#242424',
      '#66b2ff',
    ],
    [
      { focused: true, focus_visible: 0, focusVisible: 'false' },
      '#242424',
      '#66b2ff',
    ],
    [{ open: 'false', popup_open: true }, '#454545', '#484848'],
    [
      { pressed: true, drop_hovered: 0, active_drag_target: true },
      '#454545',
      '#484848',
    ],
    [{ enabled: false, hovered: true }, '#2b2b2b', '#363636'],
    [{ enabled: 'false', hovered: true }, '#454545', '#484848'],
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
        strokeColor: '#484848',
      });
    },
  );

  it('uses authored focused state as static metadata, and resets enabled for a requested case', () => {
    const document = authoredButton();
    const node = document.nodes!['root'];
    node.props!['enabled'] = false;
    node.state = { enabled: true, focused: true };
    expect(projectZuiDocument(document).rootNodes[0].paint.strokeColor).toBe(
      '#66b2ff',
    );
    node.state['enabled'] = false;
    expect(
      projectZuiDocument(reviewDocumentForCase(document, reviewCase))
        .rootNodes[0].paint.fillColor,
    ).toBe('#454545');
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
      '#454545',
    );
    expect(normalizeZuiDocument(document)).toEqual(before);
  });

  it('retains the runtime icon selection-before-press exception and disabled icon foreground', () => {
    const source = parseZuiDocument(
      readFileSync(
        `${root}/${assetPath.replace('workbench_button.zui', 'workbench_icon_button.zui')}`,
        'utf8',
      ),
    ).document.nodes!['root'];
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
