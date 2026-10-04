import { defaultReviewCases } from '../../tools/zui-layout-review-contract';
import { normalizeZuiDocument, type ZuiDocument } from './zui-document';
import {
  componentReviewStates,
  nativePainterReviewStates,
} from './zui-component-review-data';
import { reviewDocumentForCase } from './zui-review-case';

const field = (component = 'InputField'): ZuiDocument => ({
  asset: { kind: 'component', id: 'res://ui/input.zui', version: 2 },
  components: { Field: { root: 'root' } },
  nodes: {
    root: {
      component,
      props: {
        value: '',
        query: '',
        input_interactive: true,
        input_focusable: true,
        input_hoverable: true,
        input_clickable: true,
      },
    },
  },
});

const nativePainter = (component: string): ZuiDocument => ({
  asset: { kind: 'component', id: `res://ui/${component}.zui`, version: 2 },
  components: { Painter: { root: 'root' } },
  nodes: {
    root: {
      component,
      props:
        component === 'AgentChat'
          ? {
              messages: ['agent|Ready'],
              composer_text: 'Continue',
              streaming: false,
              error: false,
            }
          : component === 'ChatComposer'
            ? { composer_text: 'Continue', streaming: false }
            : { open: false, popup_open: false },
    },
  },
});

describe('declared component data scenarios', () => {
  it('covers valid, empty, long English, long Chinese and validation error content', () => {
    const source = field();
    const before = normalizeZuiDocument(source);
    const cases = defaultReviewCases(
      'zircon_editor/assets/ui/renamed.zui',
      'component',
      source,
    );
    expect(
      cases.find((item) => item.id === 'default-360x520-dpi1')?.data,
    ).toEqual({ componentInput: { value: 'Player' } });
    for (const prefix of ['empty', 'long-en', 'long-zh', 'error'])
      expect(
        cases.filter((item) => item.id.startsWith(`${prefix}-`)),
      ).toHaveLength(4);
    const chinese = cases.find((item) => item.id.startsWith('long-zh-'))!;
    expect(chinese.locale).toBe('zh-CN');
    expect(
      reviewDocumentForCase(source, chinese).nodes?.['root'].props?.['value'],
    ).toMatch(/[\u4e00-\u9fff]/);
    expect(normalizeZuiDocument(source)).toEqual(before);
  });

  it('uses the declared search query and refuses unrelated or visual overrides', () => {
    const source = field('SearchField');
    const review = defaultReviewCases(
      'zircon_editor/assets/ui/search.zui',
      'component',
      source,
    )[0];
    expect(
      reviewDocumentForCase(source, review).nodes?.['root'].props?.['query'],
    ).toBe('material');
    expect(() =>
      reviewDocumentForCase(source, {
        ...review,
        data: { componentInput: { value: 'wrong property' } },
      }),
    ).toThrow('declared text field value');
    expect(() =>
      reviewDocumentForCase(source, {
        ...review,
        data: { componentInput: { query: 'q', visibility: 'collapsed' } },
      }),
    ).toThrow('Unsupported component input');
  });

  it('does not synthesize input content or interaction states for fixtures', () => {
    const cases = defaultReviewCases(
      'zircon_runtime/assets/ui/runtime/fixtures/input.zui',
      'component',
      field(),
    );
    expect(
      cases.every(
        (item) => item.state === 'default' && !Object.keys(item.data).length,
      ),
    ).toBe(true);
  });

  it('reviews interaction states declared by a composite action slot', () => {
    const source: ZuiDocument = {
      asset: { kind: 'component', id: 'res://ui/panel.zui', version: 2 },
      components: {
        Panel: {
          root: 'root',
          slots: {
            title: { required: true, accepts: ['WorkbenchSectionTitle'] },
            actions: { multiple: true, accepts: ['WorkbenchButton'] },
          },
        },
      },
      nodes: { root: { component: 'HorizontalGroup' } },
    };
    const states = [
      'default',
      'hover',
      'pressed',
      'focused',
      'disabled',
      'selected',
    ];
    expect(componentReviewStates(source)).toEqual(states);
    const cases = defaultReviewCases(
      'zircon_editor/assets/ui/editor/components/panel.zui',
      'component',
      source,
    );
    expect(cases).toHaveLength(24);
    expect([...new Set(cases.map(({ state }) => state))]).toEqual(states);
  });

  it('reviews the authored selectable Workbench table row at every component width', () => {
    const source: ZuiDocument = {
      asset: {
        kind: 'component',
        id: 'res://ui/editor/components/workbench/primitives/data/workbench_table_row.zui',
        version: 2,
      },
      components: { WorkbenchTableRow: { root: 'root' } },
      nodes: {
        root: {
          component: 'Table',
          control_id: 'WorkbenchTableRowRoot',
          props: {
            options: ['Item_01', 'Mesh', '2.4 MB', '2m ago'],
            selected: false,
            input_interactive: true,
            input_clickable: true,
          },
        },
      },
    };
    const cases = defaultReviewCases(
      'zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_table_row.zui',
      'component',
      source,
    );
    expect(cases).toHaveLength(22);
    expect(
      cases
        .filter((item) => item.state.startsWith('scroll-'))
        .map(({ state, viewport, scrollPosition }) => [
          state,
          viewport.width,
          scrollPosition,
        ]),
    ).toEqual([
      ['scroll-before', 360, 'start'],
      ['scroll-before', 240, 'start'],
      ['scroll-before', 480, 'start'],
      ['scroll-after', 360, 'end'],
      ['scroll-after', 240, 'end'],
      ['scroll-after', 480, 'end'],
    ]);
    const selected = cases.filter(({ state }) => state === 'selected');
    expect(selected).toHaveLength(4);
    expect(selected.map(({ viewport }) => viewport.width)).toEqual([
      360, 360, 240, 480,
    ]);
    expect(
      reviewDocumentForCase(source, selected[2]).nodes?.root.state,
    ).toMatchObject({
      selected: true,
    });
    expect(source.nodes?.root.props?.['selected']).toBe(false);
  });

  it.each([
    ['Dialog', ['default', 'open', 'closed', 'focused', 'focus-return']],
    ['ConfirmDialog', ['default', 'open', 'closed', 'focused', 'focus-return']],
    [
      'CommandPalette',
      ['default', 'open', 'closed', 'focused', 'empty', 'focus-return'],
    ],
    [
      'NotificationCenter',
      ['default', 'open', 'closed', 'selected', 'empty', 'focus-return'],
    ],
    ['DragOverlay', ['default', 'dragging', 'drop-allowed', 'drop-blocked']],
    ['WorkbenchToast', ['default', 'open']],
    ['AgentPlan', ['default', 'running', 'complete', 'blocked']],
    ['ToolCalls', ['default', 'running', 'success', 'failure']],
    ['AgentApproval', ['default', 'pending', 'approved', 'denied']],
    ['AIUsage', ['default', 'normal', 'warning', 'exceeded']],
    ['DataGrid', ['default', 'selected', 'empty']],
    ['TreeView', ['default', 'open', 'selected', 'empty']],
    [
      'AgentChat',
      [
        'default',
        'focused',
        'disabled',
        'empty',
        'running',
        'error',
        'long-en',
        'long-zh',
      ],
    ],
    [
      'ChatComposer',
      [
        'default',
        'focused',
        'disabled',
        'empty',
        'running',
        'error',
        'long-en',
        'long-zh',
      ],
    ],
  ])('uses source-owned review states for %s', (component, states) => {
    const source = nativePainter(component);
    expect(componentReviewStates(source)).toEqual(states);
    const cases = defaultReviewCases(
      `zircon_editor/assets/ui/${component}.zui`,
      'component',
      source,
    );
    const expectedStates = states;
    expect([...new Set(cases.map(({ state }) => state))]).toEqual(
      [
        'Dialog',
        'ConfirmDialog',
        'CommandPalette',
        'NotificationCenter',
      ].includes(component)
        ? ['open', ...expectedStates.filter((state) => state !== 'open')]
        : expectedStates,
    );
  });

  it('adds native painter states to a real consumer host rather than only component roots', () => {
    const source: ZuiDocument = {
      asset: { kind: 'component', id: 'res://ui/showcase.zui', version: 2 },
      components: { Showcase: { root: 'host' } },
      nodes: {
        host: { component: 'VerticalGroup', children: [{ node: 'dialog' }] },
        dialog: {
          component: 'Dialog',
          props: { open: true, popup_open: true, title: 'Scene Settings' },
        },
      },
    };
    const cases = defaultReviewCases(
      'zircon_editor/assets/ui/editor/components/showcase/showcase.zui',
      'component',
      source,
    );
    expect([...new Set(cases.map(({ state }) => state))]).toEqual([
      'default',
      'open',
      'closed',
      'focused',
      'focus-return',
    ]);
  });

  it('keeps native painter states after prefab materialization changes the component name', () => {
    const materialized: ZuiDocument = {
      asset: { kind: 'view', id: 'res://ui/window.zui', version: 2 },
      root: { node: 'root' },
      nodes: {
        root: {
          component: 'Snackbar',
          penpot_prefab_source:
            'zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_toast.zui#WorkbenchToast',
          props: { text: 'Saved', open: false, popup_open: false },
        },
      },
    };

    expect(nativePainterReviewStates(materialized.nodes?.root)).toEqual([
      'default',
      'open',
    ]);
    const open = {
      id: 'open-1280x800-dpi1',
      sourcePath: 'zircon_editor/assets/ui/window.zui',
      host: 'editor' as const,
      viewport: { width: 1280, height: 800 },
      dpi: 1,
      locale: 'en-US',
      state: 'open',
      data: {},
    };
    expect(
      reviewDocumentForCase(materialized, open).nodes?.root.state,
    ).toMatchObject({
      open: true,
      popup_open: true,
    });
  });

  it('applies workflow state to the matching native painter without mutating source props', () => {
    const source: ZuiDocument = {
      asset: { kind: 'view', id: 'res://ui/workflow.zui', version: 2 },
      root: { node: 'root' },
      nodes: {
        root: {
          component: 'VerticalBox',
          children: [{ node: 'plan' }, { node: 'usage' }],
        },
        plan: {
          component: 'AgentPlan',
          props: {
            component_role: 'mui-x-agent-plan',
            text: 'Plan',
            collection_items: ['done|Read', 'active|Build'],
            value: 0.5,
          },
        },
        usage: {
          component: 'AIUsage',
          props: {
            component_role: 'mui-x-ai-usage',
            text: 'Usage',
            value: 0.39,
          },
        },
      },
    };
    const reviewed = reviewDocumentForCase(source, {
      id: 'complete-1280x800-dpi1',
      sourcePath: 'zircon_runtime/tests/fixtures/ui/workflow.zui',
      host: 'fixture',
      viewport: { width: 1280, height: 800 },
      dpi: 1,
      locale: 'en-US',
      state: 'complete',
      data: {},
    });
    expect(reviewed.nodes?.plan?.state).toMatchObject({
      component_variant: 'complete',
      value: 1,
    });
    expect(reviewed.nodes?.plan?.props?.['value']).toBe(0.5);
    expect(reviewed.nodes?.usage?.state).toBeUndefined();
    expect(source.nodes?.plan?.state).toBeUndefined();
  });

  it('projects chat content states into design-only state without changing source props', () => {
    const source: ZuiDocument = {
      asset: { kind: 'view', id: 'res://ui/reactbits-chat.zui', version: 2 },
      root: { node: 'root' },
      nodes: {
        root: {
          component: 'VerticalBox',
          children: [{ node: 'chat' }, { node: 'composer' }],
        },
        chat: {
          component: 'AgentChat',
          props: {
            messages: ['agent|A short answer'],
            composer_text: 'Continue',
            streaming: false,
            error: false,
          },
        },
        composer: {
          component: 'ChatComposer',
          props: { composer_text: 'Continue', streaming: false },
        },
      },
    };
    const original = normalizeZuiDocument(source);
    const reviewed = reviewDocumentForCase(source, {
      id: 'long-zh-1280x800-dpi1',
      sourcePath: 'zircon_runtime/tests/fixtures/ui/reactbits-chat.zui',
      host: 'fixture',
      viewport: { width: 1280, height: 800 },
      dpi: 1,
      locale: 'zh-CN',
      state: 'long-zh',
      data: {},
    });
    expect(reviewed.nodes?.chat?.state?.messages).toEqual([
      'user|请在窄屏断点检查响应式外壳，并保持完整上下文可见。',
      'agent|保留的宿主会让对话、工具结果和引用文件保持对齐，不隐藏溢出内容。',
    ]);
    expect(reviewed.nodes?.composer?.state?.composer_text).toMatch(
      /请继续检查完整外壳/,
    );
    expect(normalizeZuiDocument(source)).toEqual(original);

    const empty = reviewDocumentForCase(source, {
      id: 'empty-1280x800-dpi1',
      sourcePath: 'zircon_runtime/tests/fixtures/ui/reactbits-chat.zui',
      host: 'fixture',
      viewport: { width: 1280, height: 800 },
      dpi: 1,
      locale: 'en-US',
      state: 'empty',
      data: {},
    });
    expect(empty.nodes?.chat?.state).toMatchObject({ messages: [] });
    expect(empty.nodes?.composer?.state).toMatchObject({ composer_text: '' });

    const error = reviewDocumentForCase(source, {
      id: 'error-1280x800-dpi1',
      sourcePath: 'zircon_runtime/tests/fixtures/ui/reactbits-chat.zui',
      host: 'fixture',
      viewport: { width: 1280, height: 800 },
      dpi: 1,
      locale: 'en-US',
      state: 'error',
      data: {},
    });
    expect(error.nodes?.chat?.state).toMatchObject({
      error: true,
      streaming: false,
    });
    expect(error.nodes?.composer?.state).toMatchObject({
      error: true,
      streaming: false,
    });

    const focused = reviewDocumentForCase(source, {
      id: 'focused-1280x800-dpi1',
      sourcePath: 'zircon_runtime/tests/fixtures/ui/reactbits-chat.zui',
      host: 'fixture',
      viewport: { width: 1280, height: 800 },
      dpi: 1,
      locale: 'en-US',
      state: 'focused',
      data: {},
    });
    expect(focused.nodes?.chat?.state).toMatchObject({
      focused: true,
      focus_visible: true,
    });
    const disabled = reviewDocumentForCase(source, {
      id: 'disabled-1280x800-dpi1',
      sourcePath: 'zircon_runtime/tests/fixtures/ui/reactbits-chat.zui',
      host: 'fixture',
      viewport: { width: 1280, height: 800 },
      dpi: 1,
      locale: 'en-US',
      state: 'disabled',
      data: {},
    });
    expect(disabled.nodes?.composer?.state).toMatchObject({ disabled: true });
  });
});
