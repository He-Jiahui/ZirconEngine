import type { ZuiNode } from './bridge/zui-document';
import { nativePainterContent } from './penpot-native-painter-content';

function contentFor(
  component: string,
  props: ZuiNode['props'],
  state?: ZuiNode['state'],
) {
  return nativePainterContent({ component, props, state });
}

describe('native painter review content', () => {
  it('uses authored dialog copy and actions without substituting product text', () => {
    const content = contentFor('ConfirmDialog', {
      title: 'Delete selected prefab?',
      message: 'This removes the prefab reference from the scene.',
      cancel_text: 'Cancel',
      confirm_text: 'Delete',
    });

    expect(content?.texts.map(({ value }) => value)).toEqual([
      'Delete selected prefab?',
      'This removes the prefab reference from the scene.',
      'Cancel',
      'Delete',
    ]);
    expect(content?.panels.map(({ kind }) => kind)).toEqual([
      'header',
      'action',
      'action-danger',
    ]);
  });

  it('resolves the authored painter from a materialized prefab source', () => {
    const content = nativePainterContent({
      component: 'Snackbar',
      penpot_prefab_source:
        'zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_toast.zui#WorkbenchToast',
      props: { text: 'Asset imported', action_label: 'Open' },
    });

    expect(content?.texts.map(({ value }) => value)).toEqual([
      'Asset imported',
      'Open',
    ]);
  });

  it('gives a wrapping toast message a host-sized text box', () => {
    const content = contentFor('WorkbenchToast', {
      text: 'Operation completed successfully',
    });

    expect(content?.texts[0]?.placement).toMatchObject({
      y: 0.05,
      height: 0.9,
    });
    expect(content?.panels[0]?.placement).toMatchObject({
      y: 0.08,
      height: 0.84,
    });
  });

  it('renders palette rows from the authored command model', () => {
    const content = contentFor('CommandPalette', {
      query: 'build',
      placeholder: 'Search commands',
      commands: [
        'open_scene|label=Open Scene|shortcut=Ctrl+O',
        'build_project|label=Build Project|shortcut=Ctrl+B',
      ],
      filtered_commands: ['build_project'],
      selected_command_id: 'build_project',
    });

    expect(content?.texts.map(({ value }) => value)).toEqual([
      'build',
      'Build Project',
      'Ctrl+B',
    ]);
    expect(content?.panels.some(({ kind }) => kind === 'selected-row')).toBe(
      true,
    );
  });

  it('renders notification rows from the authored notification model', () => {
    const content = contentFor('NotificationCenter', {
      title: 'Notifications',
      visible_limit: 1,
      notifications: [
        'build|title=Build failed|message=Shader compile error|severity=error|unread=true',
        'asset|title=Asset import complete|message=StoneWall.mesh ready|severity=success|unread=false',
      ],
    });

    expect(content?.texts.map(({ value }) => value)).toEqual([
      'Notifications',
      'Build failed',
      'Shader compile error',
    ]);
    expect(content?.panels.some(({ kind }) => kind === 'status-error')).toBe(
      true,
    );
  });

  it('mirrors the Runtime-owned agent chat bubbles and streaming indicator', () => {
    const content = contentFor('AgentChat', {
      messages: [
        'user|Review the narrow shell',
        'agent|Checking responsive rules',
      ],
      composer_text: 'Continue the review',
      streaming: true,
      error: false,
    });

    expect(content?.panels).toHaveLength(2);
    const userBubble = content?.panels.find(
      ({ kind }) => kind === 'user-bubble',
    );
    expect(userBubble?.tone).toBe('selected');
    expect(userBubble?.placement.x).toBeCloseTo(0.38);
    expect(userBubble?.placement.y).toBeCloseTo(0.06);
    expect(userBubble?.placement.width).toBeCloseTo(0.56);
    expect(userBubble?.placement.height).toBeCloseTo(0.4025);
    expect(
      content?.panels.find(({ kind }) => kind === 'agent-bubble'),
    ).toBeUndefined();
    expect(
      content?.panels.find(({ kind }) => kind === 'streaming-indicator'),
    ).toMatchObject({
      placement: { x: 0.06, y: 0.94, width: 0.42, height: 0.03 },
      tone: 'accent',
    });
    expect(content?.texts.map(({ value }) => value)).toEqual([
      'Review the narrow shell',
      'Checking responsive rules',
    ]);
    expect(content?.texts[0].placement.x).toBeGreaterThan(
      content?.texts[1].placement.x ?? 0,
    );
  });

  it('keeps a multi-turn agent thread in source order with semantic user bubbles', () => {
    const content = contentFor('AgentChat', {
      messages: [
        'user|First question',
        'agent|First answer',
        'user|Follow-up with more context',
        'agent|Second answer with a tool result',
      ],
    });

    expect(content?.texts.map(({ value }) => value)).toEqual([
      'First question',
      'First answer',
      'Follow-up with more context',
      'Second answer with a tool result',
    ]);
    expect(
      content?.panels.filter(({ kind }) => kind === 'agent-bubble'),
    ).toHaveLength(0);
    expect(
      content?.panels.filter(({ kind }) => kind === 'user-bubble'),
    ).toHaveLength(2);
    const placements = content?.texts.map(({ placement }) => placement) ?? [];
    expect(placements.map(({ y }) => y)).toEqual(
      [...placements.map(({ y }) => y)].sort((a, b) => a - b),
    );
    expect(placements[0].x).toBeGreaterThan(placements[1].x);
    expect(placements[2].x).toBeGreaterThan(placements[3].x);
  });

  it('allows an authored assistant-bubble variant without changing text order', () => {
    const content = contentFor('AgentChat', {
      messages: ['agent|Answer', 'user|Question'],
      assistant_bubble: true,
    });

    expect(content?.texts.map(({ value }) => value)).toEqual([
      'Answer',
      'Question',
    ]);
    expect(
      content?.panels.filter(({ kind }) => kind === 'agent-bubble'),
    ).toHaveLength(1);
    expect(
      content?.panels.filter(({ kind }) => kind === 'user-bubble'),
    ).toHaveLength(1);
  });

  it('mirrors authored composer text and the Runtime-owned send affordance', () => {
    const content = contentFor('ChatComposer', {
      composer_text: 'Review the responsive collapse path',
      streaming: true,
    });

    expect(content?.panels).toEqual([
      {
        kind: 'send-action',
        placement: { x: 0.81, y: 0.08, width: 0.16, height: 0.84 },
        tone: 'accent',
      },
    ]);
    expect(content?.texts).toEqual([
      {
        value: 'Review the responsive collapse path',
        placement: { x: 0.06, y: 0.25, width: 0.7, height: 0.5 },
        tone: 'primary',
        weight: '400',
        size: 12,
      },
    ]);
    expect(content?.icons).toEqual([
      {
        name: 'arrow-up',
        placement: { x: 0.81, y: 0.08, width: 0.16, height: 0.84 },
        tone: 'primary',
        size: 's',
      },
    ]);
  });

  it('keeps icon sizing semantic and rejects numeric tiers', () => {
    const valid = contentFor('ChatComposer', {
      send_icon: 'arrow-up',
      send_icon_size: 'xl',
    });
    const invalid = contentFor('ChatComposer', {
      send_icon: 'arrow-up',
      send_icon_size: '18',
    });

    expect(valid?.icons?.[0]?.size).toBe('xl');
    expect(invalid?.icons).toEqual([]);
  });

  it('renders the agent plan from source-owned step records and progress', () => {
    const content = contentFor('AgentPlan', {
      title: 'Agent plan · 2/4 complete',
      steps: [
        'done|Map ReactBits references',
        'active|Build retained ZUI surface',
        'next|Compare native screenshots',
      ],
      value: 0.5,
      min: 0,
      max: 1,
    });

    expect(content?.texts.map(({ value }) => value)).toEqual([
      'Agent plan · 2/4 complete',
      '✓  Map ReactBits references',
      '◉  Build retained ZUI surface',
      '○  Compare native screenshots',
    ]);
    expect(content?.panels.map(({ kind }) => kind)).toEqual([
      'workflow-surface',
      'progress-track',
      'progress-fill',
      'workflow-step-done',
      'workflow-step-active',
      'workflow-step-next',
    ]);
  });

  it('renders tool-call feedback with status-specific rows', () => {
    const content = contentFor('ToolCalls', {
      title: 'Tool calls · 3',
      calls: [
        'success|read_file|layout.zui',
        'pending|search_web|ReactBits',
        'failure|apply_patch|blocked',
      ],
    });

    expect(content?.texts.map(({ value }) => value)).toEqual([
      'Tool calls · 3',
      '✓  read_file  ·  layout.zui',
      '◌  search_web  ·  ReactBits',
      '!  apply_patch  ·  blocked',
    ]);
    expect(content?.panels.map(({ kind }) => kind)).toEqual([
      'workflow-surface',
      'tool-row-success',
      'tool-row-pending',
      'tool-row-failure',
    ]);
  });

  it('renders approval copy and keeps destructive action tone source-owned', () => {
    const content = contentFor('AgentApproval', {
      title: 'Approval required',
      message:
        'The agent wants to write the generated layout into the project.',
      scope: 'Scope: zircon_runtime/tests/fixtures/ui · reversible',
      allow_label: 'Allow write',
      deny_label: 'Deny',
      approval_state: 'pending',
      destructive: true,
    });

    expect(content?.texts.map(({ value }) => value)).toEqual([
      'Approval required',
      'The agent wants to write the generated layout into the project.',
      'Scope: zircon_runtime/tests/fixtures/ui · reversible',
      'Deny',
      'Allow write',
    ]);
    expect(content?.panels.map(({ kind, tone }) => [kind, tone])).toEqual([
      ['workflow-surface', 'surface'],
      ['approval-warning', 'warning'],
      ['approval-deny', 'surface'],
      ['approval-allow', 'danger'],
    ]);
  });

  it('uses options as the action fallback for source-owned approval cards', () => {
    const content = contentFor('AgentApproval', {
      text: 'Approval required',
      value_text: 'Write the generated layout',
      label_text: 'Scope: fixtures',
      options: ['Deny', 'Allow write'],
      destructive: true,
    });

    expect(content?.texts.map(({ value }) => value)).toEqual([
      'Approval required',
      'Write the generated layout',
      'Scope: fixtures',
      'Deny',
      'Allow write',
    ]);
  });

  it('renders AI usage as a bounded source-owned progress lane', () => {
    const content = contentFor('AIUsage', {
      title: 'AI usage',
      value: 0.39,
      min: 0,
      max: 1,
      detail: '12.4k used · 19.6k remaining · $0.08',
    });

    expect(content?.texts.map(({ value }) => value)).toEqual([
      'AI usage',
      '12.4k used · 19.6k remaining · $0.08',
    ]);
    expect(content?.panels.map(({ kind }) => kind)).toEqual([
      'workflow-surface',
      'usage-track',
      'usage-fill',
    ]);
    expect(
      content?.panels.find(({ kind }) => kind === 'usage-fill')?.placement,
    ).toEqual({
      x: 0.08,
      y: 0.52,
      width: 0.3276,
      height: 0.08,
    });
  });

  it('renders a data grid from authored columns and row records', () => {
    const content = contentFor('DataGrid', {
      title: 'Assets',
      options: ['Name', 'Type', 'Status'],
      collection_items: [
        'selected|Tree.mesh|Mesh|Ready',
        'normal|Rock.mat|Material|Review',
      ],
      selected_row_id: 'Tree.mesh',
      empty_text: 'No assets match the filter',
    });

    expect(content?.texts.map(({ value }) => value)).toEqual([
      'Assets',
      'Name',
      'Type',
      'Status',
      'Tree.mesh',
      'Mesh',
      'Ready',
      'Rock.mat',
      'Material',
      'Review',
    ]);
    expect(content?.panels.map(({ kind }) => kind)).toEqual([
      'data-grid-surface',
      'data-grid-header',
      'data-grid-row-selected',
      'data-grid-row',
    ]);
    expect(content?.texts.slice(4).every(({ size }) => size === 11)).toBe(true);
  });

  it('compacts four-column data-grid cells for narrow review viewports', () => {
    const content = contentFor('DataGrid', {
      title: 'Assets',
      options: ['Name', 'Type', 'Size', 'Status'],
      collection_items: ['normal|SM_Tree_Oak_01|Static Mesh|2.4 MB|Ready'],
    });

    expect(content?.texts.slice(5).every(({ size }) => size === 9)).toBe(true);
  });

  it('renders a source-owned empty data grid without inventing rows', () => {
    const content = contentFor(
      'DataGrid',
      {
        text: 'Assets',
        options: ['Name', 'Type'],
        collection_items: ['normal|Tree.mesh|Mesh'],
        empty_text: 'No assets match the filter',
      },
      { collection_items: [] },
    );

    expect(content?.texts.map(({ value }) => value)).toEqual([
      'Assets',
      'Name',
      'Type',
      'No assets match the filter',
    ]);
    expect(content?.panels.map(({ kind }) => kind)).toEqual([
      'data-grid-surface',
      'data-grid-header',
    ]);
  });

  it('renders tree rows with authored depth and selection markers', () => {
    const content = contentFor('TreeView', {
      title: 'Folders',
      collection_items: [
        'expanded|0|Assets',
        'selected|1|Meshes',
        'normal|1|Materials',
      ],
      selected_id: 'Meshes',
      empty_text: 'No folders',
    });

    expect(content?.texts.map(({ value }) => value)).toEqual([
      'Folders',
      '▾  Assets',
      '●  Meshes',
      '○  Materials',
    ]);
    expect(content?.panels.map(({ kind }) => kind)).toEqual([
      'tree-surface',
      'tree-row-expanded',
      'tree-row-selected',
      'tree-row',
    ]);
  });

  it('uses temporary native review state over authored props for empty and selected data', () => {
    const empty = contentFor(
      'CommandPalette',
      {
        commands: ['build|label=Build'],
        filtered_commands: ['build'],
        empty_text: 'No commands found',
      },
      { commands: [], filtered_commands: [] },
    );
    const selected = contentFor(
      'NotificationCenter',
      {
        notifications: [
          'asset|title=Asset import complete|message=StoneWall.mesh ready',
        ],
        selected_notification_id: '',
      },
      { selected_notification_id: 'asset' },
    );

    expect(empty?.texts.map(({ value }) => value)).toEqual([
      'No commands found',
    ]);
    expect(
      selected?.panels.find(({ kind }) => kind === 'notification-row')?.tone,
    ).toBe('selected');
  });

  it('uses the source drop decision for drag indicator tone', () => {
    const allowed = contentFor('DragOverlay', {
      payload_label: 'StoneWall.mesh',
      drop_indicator_text: 'Drop into scene',
      drop_allowed: true,
    });
    const blocked = contentFor(
      'DragOverlay',
      {
        payload_label: 'StoneWall.mesh',
        drop_indicator_text: 'Drop into scene',
        drop_allowed: true,
      },
      { drop_allowed: false },
    );

    expect(
      allowed?.panels.find(({ kind }) => kind === 'drop-indicator')?.tone,
    ).toBe('accent');
    expect(
      blocked?.panels.find(({ kind }) => kind === 'drop-indicator')?.tone,
    ).toBe('danger');
  });

  it('adds a visible focus ring only for focus-visible review state', () => {
    const unfocused = contentFor('Dialog', { title: 'Scene Settings' });
    const focused = contentFor(
      'Dialog',
      { title: 'Scene Settings' },
      { focused: true, focus_visible: true },
    );

    expect(
      unfocused?.panels.some(({ kind }) => kind.startsWith('focus-ring-')),
    ).toBe(false);
    expect(
      focused?.panels.filter(({ kind }) => kind.startsWith('focus-ring-')),
    ).toHaveLength(4);
    expect(
      focused?.panels
        .filter(({ kind }) => kind.startsWith('focus-ring-'))
        .map(({ tone }) => tone),
    ).toEqual(['accent', 'accent', 'accent', 'accent']);
  });

  it.each([
    'CommandPalette',
    'ConfirmDialog',
    'Dialog',
    'DragOverlay',
    'NotificationCenter',
    'WorkbenchToast',
    'AgentPlan',
    'ToolCalls',
    'AgentApproval',
    'AIUsage',
    'DataGrid',
    'TreeView',
  ])(
    'keeps %s empty when the source has no display properties',
    (component) => {
      expect(contentFor(component, {})).toBeNull();
    },
  );
});
