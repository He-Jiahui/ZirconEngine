import {
  treeRowContent,
  type TreeRowPaintContext,
} from './bridge/zui-tree-row-content';
import featherIcons from 'feather-icons/dist/icons.json';
import type { ZuiNode, ZuiTable } from './bridge/zui-document';
import { nativePainterComponent } from './bridge/zui-native-painter-role';

export type NativePainterTone =
  | 'accent'
  | 'danger'
  | 'info'
  | 'muted'
  | 'primary'
  | 'selected'
  | 'success'
  | 'surface'
  | 'warning';

export interface NativePainterPlacement {
  x: number;
  y: number;
  width: number;
  height: number;
  exactBounds?: boolean;
}

export interface NativePainterPanel {
  color?: string;
  opacity?: number;
  borderColor?: string;
  borderWidth?: number;
  radius?: number;
  kind: string;
  placement: NativePainterPlacement;
  tone: NativePainterTone;
}

export interface NativePainterText {
  value: string;
  placement: NativePainterPlacement;
  tone: NativePainterTone;
  color?: string;
  lineHeight?: number;
  exactBounds?: boolean;
  weight?: '400' | '600';
  size?: number;
}

export type NativePainterIconSize = 's' | 'm' | 'l' | 'xl';

export interface NativePainterIcon {
  svg?: string;
  name: string;
  placement: NativePainterPlacement;
  tone: NativePainterTone;
  size: NativePainterIconSize;
  color?: string;
  exactBounds?: boolean;
}

export interface NativePainterContent {
  panels: NativePainterPanel[];
  texts: NativePainterText[];
  /** Optional vector affordances kept separate from text content. */
  icons?: NativePainterIcon[];
}

const MAX_PREVIEW_ROWS = 3;

export function nativePainterContent(
  node: ZuiNode,
  context?: TreeRowPaintContext,
): NativePainterContent | null {
  // Review cases carry deterministic Runtime inputs in `state`. They override
  // authored defaults in the design projection without changing the source.
  const props: ZuiTable = { ...(node.props ?? {}), ...(node.state ?? {}) };
  if (!Object.keys(props).length) return null;
  let content: NativePainterContent | null;
  switch (nativePainterComponent(node)) {
    case 'AgentChat':
      content = agentChatContent(props);
      break;
    case 'AgentPlan':
      content = agentPlanContent(props);
      break;
    case 'AgentApproval':
      content = agentApprovalContent(props);
      break;
    case 'AIUsage':
      content = aiUsageContent(props);
      break;
    case 'ChatComposer':
      content = chatComposerContent(props);
      break;
    case 'DataGrid':
      content = dataGridContent(props);
      break;
    case 'CommandPalette':
      content = commandPaletteContent(props);
      break;
    case 'ConfirmDialog':
      content = confirmDialogContent(props);
      break;
    case 'Dialog':
      content = dialogContent(props);
      break;
    case 'DragOverlay':
      content = dragOverlayContent(props);
      break;
    case 'NotificationCenter':
      content = notificationCenterContent(props);
      break;
    case 'ToolCalls':
      content = toolCallsContent(props);
      break;
    case 'TreeRow':
      return treeRowContent(node, context);
    case 'TreeView':
      content = treeViewContent(props);
      break;
    case 'WorkbenchToast':
      content = workbenchToastContent(props);
      break;
    default:
      return null;
  }
  const focusVisible =
    sourceBoolean(props, 'focus_visible') ??
    sourceBoolean(props, 'focused') ??
    false;
  return content && focusVisible ? addFocusRing(content) : content;
}

/**
 * Keep Penpot's owner-board geometry aligned with the retained host's
 * `mui-x-agent-chat` painter. Message values use the same compact
 * `role|text` transport as the retained projection. Assistant prose stays in
 * the conversation's continuous reading flow; user turns receive the
 * right-aligned bubble. An explicit assistant-bubble prop supports products
 * that intentionally choose the alternate visual treatment.
 */
function agentChatContent(props: ZuiTable): NativePainterContent {
  const content = emptyContent();
  const assistantBubble =
    sourceBoolean(props, 'assistant_bubble') === true ||
    sourceBoolean(props, 'assistant_message_bubble') === true;
  for (const message of chatMessageLayouts(props)) {
    addText(content, message.text, message.textPlacement, 'primary', '400', 11);
    if (message.role === 1 || assistantBubble) {
      addPanel(
        content,
        message.role === 0 ? 'agent-bubble' : 'user-bubble',
        message.placement,
        message.role === 0 ? 'surface' : 'selected',
      );
    }
  }
  if (isAgentChatStreaming(props)) {
    addPanel(
      content,
      'streaming-indicator',
      { x: 0.06, y: 0.94, width: 0.42, height: 0.03 },
      'accent',
    );
  }
  return content;
}

/**
 * Mirror the retained host's `mui-x-chat-composer` painter. Its input surface
 * is the owner board, authored input text occupies the leading field, and the
 * right-aligned send action remains a separate visual affordance.
 */
function chatComposerContent(props: ZuiTable): NativePainterContent {
  const content = emptyContent();
  const composerText = firstString(props, ['composer_text', 'value_text']);
  if (composerText) {
    addText(
      content,
      composerText,
      { x: 0.06, y: 0.25, width: 0.7, height: 0.5 },
      'primary',
      '400',
      12,
    );
  }
  addPanel(
    content,
    'send-action',
    { x: 0.81, y: 0.08, width: 0.16, height: 0.84 },
    'accent',
  );
  const icon = sourceString(props, 'send_icon') ?? 'arrow-up';
  const iconSize = sourceIconSize(props, 'send_icon_size');
  if (iconSize && isSupportedNativeIcon(icon)) {
    addIcon(
      content,
      icon,
      { x: 0.81, y: 0.08, width: 0.16, height: 0.84 },
      'primary',
      iconSize,
    );
  }
  return content;
}

function dataGridContent(props: ZuiTable): NativePainterContent | null {
  const title = firstString(props, ['title', 'text']);
  const columns = sourceStrings(props, 'columns').length
    ? sourceStrings(props, 'columns')
    : sourceStrings(props, 'options');
  const rows = sourceStrings(props, 'rows').length
    ? sourceStrings(props, 'rows')
    : sourceStrings(props, 'collection_items');
  const emptyText = firstString(props, ['empty_text', 'value_text']);
  if (!title && columns.length === 0 && rows.length === 0 && !emptyText) {
    return {
      panels: [
        {
          kind: 'data-grid-surface',
          placement: { x: 0.03, y: 0.03, width: 0.94, height: 0.94 },
          tone: 'surface',
        },
      ],
      texts: [],
    };
  }

  const content = emptyContent();
  addPanel(
    content,
    'data-grid-surface',
    { x: 0.03, y: 0.03, width: 0.94, height: 0.94 },
    'surface',
  );
  addPanel(
    content,
    'data-grid-header',
    { x: 0.06, y: 0.07, width: 0.88, height: 0.16 },
    'selected',
  );
  if (title)
    addText(
      content,
      title,
      { x: 0.1, y: 0.09, width: 0.8, height: 0.1 },
      'primary',
      '600',
      13,
    );

  const visibleColumns = columns.slice(0, 5);
  const columnCount = Math.max(visibleColumns.length, 1);
  const columnWidth = 0.8 / columnCount;
  for (const [index, column] of visibleColumns.entries()) {
    addText(
      content,
      column,
      {
        x: 0.1 + columnWidth * index,
        y: 0.17,
        width: columnWidth - 0.02,
        height: 0.08,
      },
      'muted',
      '600',
      10,
    );
  }

  const selectedRowId = sourceString(props, 'selected_row_id');
  const visibleRows = rows
    .map(parseDataGridRow)
    .filter((row): row is DataGridRow => row !== null)
    .slice(0, 6);
  if (visibleRows.length === 0) {
    if (emptyText)
      addText(
        content,
        emptyText,
        { x: 0.12, y: 0.52, width: 0.76, height: 0.14 },
        'muted',
        '400',
        12,
      );
    return populatedContent(content);
  }

  const rowHeight = 0.67 / visibleRows.length;
  // Keep dense four-column asset tables readable at the 640px review
  // viewport.  The placement is normalized, so a fixed 11px label can wrap
  // inside a narrow column and collide with the following row.  A compact
  // label size preserves the authored cells while keeping each row single
  // line; wider tables retain the normal workbench scale.
  const cellTextSize = columnCount >= 4 ? 9 : 11;
  for (const [index, row] of visibleRows.entries()) {
    const selected =
      row.status === 'selected' ||
      (selectedRowId !== null && row.cells[0] === selectedRowId) ||
      (sourceBoolean(props, 'selected') === true && index === 0);
    const y = 0.26 + rowHeight * index;
    addPanel(
      content,
      selected ? 'data-grid-row-selected' : 'data-grid-row',
      { x: 0.06, y, width: 0.88, height: Math.max(0.08, rowHeight - 0.015) },
      selected ? 'accent' : dataGridRowTone(row.status),
    );
    for (const [cellIndex, cell] of row.cells.slice(0, columnCount).entries()) {
      addText(
        content,
        cell,
        {
          x: 0.1 + columnWidth * cellIndex,
          y: y + 0.025,
          width: columnWidth - 0.02,
          height: Math.max(0.05, rowHeight - 0.05),
        },
        cellIndex === 0 ? 'primary' : 'muted',
        cellIndex === 0 ? '600' : '400',
        cellTextSize,
      );
    }
  }
  return populatedContent(content);
}

interface DataGridRow {
  status: string;
  cells: string[];
}

function parseDataGridRow(value: string): DataGridRow | null {
  const parts = value
    .split('|')
    .map((part) => part.trim())
    .filter((part) => part.length > 0);
  if (parts.length === 0) return null;
  const knownStatus = [
    'selected',
    'normal',
    'success',
    'warning',
    'error',
    'pending',
  ].includes(parts[0].toLowerCase());
  const status = knownStatus ? parts.shift()!.toLowerCase() : 'normal';
  return parts.length ? { status, cells: parts } : null;
}

function dataGridRowTone(status: string): NativePainterTone {
  switch (status) {
    case 'success':
      return 'success';
    case 'warning':
      return 'warning';
    case 'error':
      return 'danger';
    case 'pending':
      return 'info';
    default:
      return 'surface';
  }
}

function treeViewContent(props: ZuiTable): NativePainterContent | null {
  const title = firstString(props, ['title', 'text']);
  const items = sourceStrings(props, 'items').length
    ? sourceStrings(props, 'items')
    : sourceStrings(props, 'collection_items');
  const emptyText = firstString(props, ['empty_text', 'value_text']);
  if (!title && items.length === 0 && !emptyText) {
    return {
      panels: [
        {
          kind: 'tree-surface',
          placement: { x: 0.03, y: 0.03, width: 0.94, height: 0.94 },
          tone: 'surface',
        },
      ],
      texts: [],
    };
  }

  const content = emptyContent();
  addPanel(
    content,
    'tree-surface',
    { x: 0.03, y: 0.03, width: 0.94, height: 0.94 },
    'surface',
  );
  if (title)
    addText(
      content,
      title,
      { x: 0.1, y: 0.09, width: 0.8, height: 0.1 },
      'primary',
      '600',
      13,
    );

  const selectedId = sourceString(props, 'selected_id');
  const visibleItems = items
    .map(parseTreeItem)
    .filter((item): item is TreeItem => item !== null)
    .slice(0, 8);
  if (visibleItems.length === 0) {
    if (emptyText)
      addText(
        content,
        emptyText,
        { x: 0.12, y: 0.5, width: 0.76, height: 0.14 },
        'muted',
        '400',
        12,
      );
    return populatedContent(content);
  }

  const rowHeight = 0.72 / visibleItems.length;
  for (const [index, item] of visibleItems.entries()) {
    const selected =
      item.status === 'selected' ||
      (selectedId !== null && item.label === selectedId) ||
      (sourceBoolean(props, 'selected') === true && index === 0);
    const expanded = item.status === 'expanded' || item.status === 'open';
    const y = 0.22 + rowHeight * index;
    addPanel(
      content,
      expanded
        ? 'tree-row-expanded'
        : selected
          ? 'tree-row-selected'
          : 'tree-row',
      {
        x: 0.07,
        y,
        width: 0.86,
        height: Math.max(0.08, rowHeight - 0.015),
      },
      selected ? 'accent' : expanded ? 'info' : 'surface',
    );
    const marker = expanded ? '▾' : selected ? '●' : '○';
    const indent = Math.min(3, Math.max(0, item.depth)) * 0.055;
    addText(
      content,
      `${marker}  ${item.label}`,
      {
        x: 0.1 + indent,
        y: y + 0.025,
        width: Math.max(0.2, 0.78 - indent),
        height: Math.max(0.05, rowHeight - 0.05),
      },
      selected ? 'primary' : expanded ? 'info' : 'muted',
      selected || expanded ? '600' : '400',
      11,
    );
  }
  return populatedContent(content);
}

interface TreeItem {
  status: string;
  depth: number;
  label: string;
}

function parseTreeItem(value: string): TreeItem | null {
  const parts = value
    .split('|')
    .map((part) => part.trim())
    .filter((part) => part.length > 0);
  if (parts.length === 0) return null;
  let status = 'normal';
  let depth = 0;
  let labelParts = parts;
  if (
    ['expanded', 'open', 'selected', 'normal', 'leaf'].includes(
      parts[0].toLowerCase(),
    )
  ) {
    status = parts[0].toLowerCase();
    labelParts = parts.slice(1);
  }
  if (labelParts.length > 1 && /^\d+$/.test(labelParts[0])) {
    depth = Number.parseInt(labelParts[0], 10);
    labelParts = labelParts.slice(1);
  }
  const label = labelParts.join(' · ').trim();
  return label ? { status, depth, label } : null;
}

function agentPlanContent(props: ZuiTable): NativePainterContent | null {
  const title = firstString(props, ['title', 'text']);
  const steps = sourceStrings(props, 'steps').length
    ? sourceStrings(props, 'steps')
    : sourceStrings(props, 'collection_items');
  if (!title && steps.length === 0) return null;

  const content = emptyContent();
  addPanel(
    content,
    'workflow-surface',
    { x: 0.04, y: 0.04, width: 0.92, height: 0.92 },
    'surface',
  );
  if (title)
    addText(
      content,
      title,
      { x: 0.1, y: 0.09, width: 0.8, height: 0.1 },
      'primary',
      '600',
      13,
    );

  const progress = normalizedProgress(props);
  addPanel(
    content,
    'progress-track',
    { x: 0.1, y: 0.22, width: 0.8, height: 0.06 },
    'selected',
  );
  addPanel(
    content,
    'progress-fill',
    { x: 0.1, y: 0.22, width: 0.8 * progress, height: 0.06 },
    'accent',
  );

  const visibleSteps = steps
    .map(parseWorkflowStep)
    .filter((step): step is WorkflowStep => step !== null)
    .slice(0, 4);
  const rowHeight = visibleSteps.length ? 0.62 / visibleSteps.length : 0;
  for (const [index, step] of visibleSteps.entries()) {
    const y = 0.32 + rowHeight * index;
    const kind = `workflow-step-${step.status}`;
    addPanel(
      content,
      kind,
      { x: 0.08, y, width: 0.84, height: Math.max(0.1, rowHeight - 0.02) },
      workflowStepTone(step.status),
    );
    addText(
      content,
      `${workflowStepMarker(step.status)}  ${step.label}`,
      {
        x: 0.12,
        y: y + 0.02,
        width: 0.74,
        height: Math.max(0.07, rowHeight - 0.04),
      },
      workflowStepTextTone(step.status),
      step.status === 'active' ? '600' : '400',
      12,
    );
  }
  return populatedContent(content);
}

function toolCallsContent(props: ZuiTable): NativePainterContent | null {
  const title = firstString(props, ['title', 'text']);
  const calls = sourceStrings(props, 'calls').length
    ? sourceStrings(props, 'calls')
    : sourceStrings(props, 'collection_items');
  if (!title && calls.length === 0) return null;

  const content = emptyContent();
  addPanel(
    content,
    'workflow-surface',
    { x: 0.04, y: 0.04, width: 0.92, height: 0.92 },
    'surface',
  );
  if (title)
    addText(
      content,
      title,
      { x: 0.1, y: 0.09, width: 0.8, height: 0.1 },
      'primary',
      '600',
      13,
    );

  const limit = Math.max(
    1,
    Math.min(4, sourceInteger(props, 'visible_limit') ?? 4),
  );
  const visibleCalls = calls
    .map(parseToolCall)
    .filter((call): call is ToolCall => call !== null)
    .slice(0, limit);
  const rowHeight = visibleCalls.length ? 0.72 / visibleCalls.length : 0;
  for (const [index, call] of visibleCalls.entries()) {
    const y = 0.2 + rowHeight * index;
    addPanel(
      content,
      `tool-row-${call.status}`,
      { x: 0.08, y, width: 0.84, height: Math.max(0.12, rowHeight - 0.02) },
      toolCallTone(call.status),
    );
    addText(
      content,
      `${toolCallMarker(call.status)}  ${call.tool}  ·  ${call.detail}`,
      {
        x: 0.12,
        y: y + 0.02,
        width: 0.76,
        height: Math.max(0.08, rowHeight - 0.04),
      },
      'primary',
      '400',
      12,
    );
  }
  return populatedContent(content);
}

function agentApprovalContent(props: ZuiTable): NativePainterContent | null {
  const title = firstString(props, ['title', 'text']);
  const message = firstString(props, ['message', 'value_text']);
  const scope = firstString(props, ['scope', 'label_text']);
  const actionOptions = sourceStrings(props, 'options');
  const deny =
    firstString(props, ['deny_label', 'cancel_text']) ??
    actionOptions[0] ??
    null;
  const allow =
    firstString(props, ['allow_label', 'confirm_text']) ??
    actionOptions[1] ??
    null;
  if (!title && !message && !scope && !deny && !allow) return null;

  const content = emptyContent();
  addPanel(
    content,
    'workflow-surface',
    { x: 0.04, y: 0.04, width: 0.92, height: 0.92 },
    'surface',
  );
  addPanel(
    content,
    'approval-warning',
    { x: 0.04, y: 0.04, width: 0.92, height: 0.04 },
    'warning',
  );
  if (title)
    addText(
      content,
      title,
      { x: 0.1, y: 0.09, width: 0.8, height: 0.1 },
      'primary',
      '600',
      13,
    );
  if (message)
    addText(
      content,
      message,
      { x: 0.1, y: 0.24, width: 0.8, height: 0.2 },
      'primary',
      '400',
      12,
    );
  if (scope)
    addText(
      content,
      scope,
      { x: 0.1, y: 0.47, width: 0.8, height: 0.12 },
      'muted',
      '400',
      11,
    );
  if (deny) {
    addPanel(
      content,
      'approval-deny',
      { x: 0.48, y: 0.72, width: 0.19, height: 0.16 },
      'surface',
    );
    addText(
      content,
      deny,
      { x: 0.5, y: 0.74, width: 0.15, height: 0.12 },
      'primary',
      '600',
      12,
    );
  }
  if (allow) {
    const tone =
      sourceBoolean(props, 'destructive') === true ? 'danger' : 'accent';
    addPanel(
      content,
      'approval-allow',
      { x: 0.7, y: 0.72, width: 0.2, height: 0.16 },
      tone,
    );
    addText(
      content,
      allow,
      { x: 0.72, y: 0.74, width: 0.16, height: 0.12 },
      'primary',
      '600',
      12,
    );
  }
  return populatedContent(content);
}

function aiUsageContent(props: ZuiTable): NativePainterContent | null {
  const title = firstString(props, ['title', 'text']);
  const detail = firstString(props, ['detail', 'value_text']);
  if (!title && !detail && props['value'] === undefined) return null;

  const content = emptyContent();
  addPanel(
    content,
    'workflow-surface',
    { x: 0.04, y: 0.04, width: 0.92, height: 0.92 },
    'surface',
  );
  if (title)
    addText(
      content,
      title,
      { x: 0.1, y: 0.1, width: 0.8, height: 0.1 },
      'primary',
      '600',
      13,
    );
  const progress = normalizedProgress(props);
  addPanel(
    content,
    'usage-track',
    { x: 0.08, y: 0.52, width: 0.84, height: 0.08 },
    'selected',
  );
  addPanel(
    content,
    'usage-fill',
    { x: 0.08, y: 0.52, width: 0.84 * progress, height: 0.08 },
    'accent',
  );
  if (detail)
    addText(
      content,
      detail,
      { x: 0.1, y: 0.68, width: 0.8, height: 0.12 },
      'muted',
      '400',
      11,
    );
  return populatedContent(content);
}

interface WorkflowStep {
  status: 'done' | 'active' | 'next';
  label: string;
}

function parseWorkflowStep(value: string): WorkflowStep | null {
  const [rawStatus, ...parts] = value.split('|');
  const status = rawStatus.trim().toLowerCase();
  if (!['done', 'active', 'next'].includes(status)) return null;
  const label = parts.join('|').trim();
  return label ? { status: status as WorkflowStep['status'], label } : null;
}

function workflowStepMarker(status: WorkflowStep['status']): string {
  return status === 'done' ? '✓' : status === 'active' ? '◉' : '○';
}

function workflowStepTone(status: WorkflowStep['status']): NativePainterTone {
  return status === 'done'
    ? 'success'
    : status === 'active'
      ? 'info'
      : 'surface';
}

function workflowStepTextTone(
  status: WorkflowStep['status'],
): NativePainterTone {
  return status === 'next' ? 'muted' : 'primary';
}

interface ToolCall {
  status: 'success' | 'pending' | 'failure';
  tool: string;
  detail: string;
}

function parseToolCall(value: string): ToolCall | null {
  const [rawStatus, rawTool, ...parts] = value.split('|');
  const status = rawStatus.trim().toLowerCase();
  if (!['success', 'pending', 'failure'].includes(status)) return null;
  const tool = rawTool?.trim();
  const detail = parts.join('|').trim();
  return tool && detail
    ? { status: status as ToolCall['status'], tool, detail }
    : null;
}

function toolCallMarker(status: ToolCall['status']): string {
  return status === 'success' ? '✓' : status === 'pending' ? '◌' : '!';
}

function toolCallTone(status: ToolCall['status']): NativePainterTone {
  return status === 'success'
    ? 'success'
    : status === 'pending'
      ? 'info'
      : 'danger';
}

function normalizedProgress(props: ZuiTable): number {
  const value = props['value'];
  if (typeof value !== 'number' || !Number.isFinite(value)) return 0;
  const min =
    typeof props['min'] === 'number' && Number.isFinite(props['min'])
      ? props['min']
      : 0;
  const max =
    typeof props['max'] === 'number' && Number.isFinite(props['max'])
      ? props['max']
      : 1;
  if (max <= min) return 0;
  return Math.min(1, Math.max(0, (value - min) / (max - min)));
}

interface ChatMessageLayout {
  role: number;
  text: string;
  placement: NativePainterPlacement;
  textPlacement: NativePainterPlacement;
}

function chatMessageLayouts(props: ZuiTable): ChatMessageLayout[] {
  const messages = sourceStrings(props, 'messages')
    .map((value, index) => {
      const parsed = parseChatMessage(value);
      return {
        role: parsed.role ?? (index % 2 === 0 ? 0 : 1),
        text: parsed.text,
      };
    })
    .filter(({ text }) => text.length > 0);
  if (!messages.length) {
    const fallback = firstString(props, ['text']);
    if (fallback) messages.push({ role: 0, text: fallback });
  }

  const totalCount = messages.length;
  const maxVisible = Math.min(
    64,
    Math.max(1, sourceInteger(props, 'max_visible_messages') ?? 12),
  );
  const visibleCount = Math.min(totalCount, maxVisible);
  const visible = messages.slice(0, visibleCount);
  if (visibleCount < totalCount && visible.length) {
    const overflowText = firstString(props, [
      'overflow_text',
      'overflow_label',
    ]);
    if (overflowText) visible[visible.length - 1].text += `\n${overflowText}`;
  }
  const inset = 0.06;
  const gap = 0.025;
  const indicatorReserve = isAgentChatStreaming(props) ? 0.05 : 0;
  const rowHeight = Math.max(
    0.12,
    (1 - inset * 2 - indicatorReserve - gap * (visible.length - 1)) /
      visible.length,
  );
  return visible.map((message, index) => {
    const bubbleWidth = message.role === 0 ? 0.72 : 0.56;
    const x = message.role === 0 ? inset : 1 - inset - bubbleWidth;
    const y = inset + index * (rowHeight + gap);
    const placement = { x, y, width: bubbleWidth, height: rowHeight };
    return {
      role: message.role,
      text: message.text,
      placement,
      textPlacement: {
        x: x + 0.025,
        y: y + 0.03,
        width: bubbleWidth - 0.05,
        height: Math.max(0.04, rowHeight - 0.06),
      },
    };
  });
}

function parseChatMessage(value: string): {
  role: number | null;
  text: string;
} {
  for (const separator of ['|', ':']) {
    const separatorIndex = value.indexOf(separator);
    if (separatorIndex < 0) continue;
    const prefix = value.slice(0, separatorIndex).trim().toLowerCase();
    const text = value.slice(separatorIndex + 1).trim();
    const role =
      prefix === 'assistant' || prefix === 'agent' || prefix === 'system'
        ? 0
        : prefix === 'user' || prefix === 'human' || prefix === 'you'
          ? 1
          : null;
    if (role !== null && text) return { role, text };
  }
  return { role: null, text: value.trim() };
}

function isAgentChatStreaming(props: ZuiTable): boolean {
  if (sourceBoolean(props, 'streaming') === true) return true;
  if (sourceBoolean(props, 'popup_open') === true) return true;
  const variant = firstString(props, [
    'component_variant',
    'variant',
    'mui_variant',
  ]);
  return variant?.toLowerCase().includes('streaming') ?? false;
}

function dialogContent(props: ZuiTable): NativePainterContent | null {
  const content = emptyContent();
  const title = firstString(props, ['title', 'text']);
  const message = sourceString(props, 'message');
  const action = firstString(props, ['action', 'confirm_text']);
  if (title) {
    addPanel(
      content,
      'header',
      { x: 0.06, y: 0.07, width: 0.88, height: 0.16 },
      'surface',
    );
    addText(
      content,
      title,
      { x: 0.12, y: 0.09, width: 0.76, height: 0.12 },
      'primary',
      '600',
      16,
    );
  }
  if (message)
    addText(
      content,
      message,
      { x: 0.12, y: 0.29, width: 0.76, height: 0.3 },
      'muted',
      '400',
      12,
    );
  if (action) {
    addPanel(
      content,
      'action',
      { x: 0.7, y: 0.73, width: 0.2, height: 0.16 },
      'accent',
    );
    addText(
      content,
      action,
      { x: 0.72, y: 0.74, width: 0.16, height: 0.14 },
      'primary',
      '600',
      12,
    );
  }
  return populatedContent(content);
}

function confirmDialogContent(props: ZuiTable): NativePainterContent | null {
  const content = emptyContent();
  const title = firstString(props, ['title', 'text']);
  const message = sourceString(props, 'message');
  const cancel = sourceString(props, 'cancel_text');
  const confirm = sourceString(props, 'confirm_text');
  const destructive = sourceBoolean(props, 'destructive');
  const confirmationTone =
    destructive === false
      ? severityTone(sourceString(props, 'severity') ?? 'accent')
      : 'danger';
  if (title) {
    addPanel(
      content,
      'header',
      { x: 0.06, y: 0.07, width: 0.88, height: 0.15 },
      'surface',
    );
    addText(
      content,
      title,
      { x: 0.12, y: 0.09, width: 0.76, height: 0.12 },
      'primary',
      '600',
      16,
    );
  }
  if (message)
    addText(
      content,
      message,
      { x: 0.12, y: 0.28, width: 0.76, height: 0.3 },
      'muted',
      '400',
      12,
    );
  if (cancel) {
    addPanel(
      content,
      'action',
      { x: 0.47, y: 0.73, width: 0.2, height: 0.16 },
      'surface',
    );
    addText(
      content,
      cancel,
      { x: 0.49, y: 0.74, width: 0.16, height: 0.14 },
      'primary',
      '600',
      12,
    );
  }
  if (confirm) {
    addPanel(
      content,
      confirmationTone === 'danger' ? 'action-danger' : 'action-confirm',
      { x: 0.7, y: 0.73, width: 0.2, height: 0.16 },
      confirmationTone,
    );
    addText(
      content,
      confirm,
      { x: 0.72, y: 0.74, width: 0.16, height: 0.14 },
      'primary',
      '600',
      12,
    );
  }
  return populatedContent(content);
}

function commandPaletteContent(props: ZuiTable): NativePainterContent | null {
  const content = emptyContent();
  const query = sourceString(props, 'query');
  const placeholder = sourceString(props, 'placeholder');
  const search = query || placeholder;
  if (search) {
    addPanel(
      content,
      'search-field',
      { x: 0.05, y: 0.07, width: 0.9, height: 0.16 },
      'surface',
    );
    addText(
      content,
      search,
      { x: 0.09, y: 0.09, width: 0.76, height: 0.12 },
      query ? 'primary' : 'muted',
      '400',
      12,
    );
  }

  const commands = sourceStrings(props, 'commands')
    .map(parseDelimitedRecord)
    .filter((command): command is DelimitedRecord => command !== null);
  const filteredIds = new Set(sourceStrings(props, 'filtered_commands'));
  const visibleCommands = (
    filteredIds.size > 0
      ? commands.filter(({ id }) => filteredIds.has(id))
      : commands
  ).slice(0, MAX_PREVIEW_ROWS);
  const selected = sourceString(props, 'selected_command_id');
  const rowHeight = visibleCommands.length ? 0.6 / visibleCommands.length : 0;
  for (const [index, command] of visibleCommands.entries()) {
    const y = 0.29 + rowHeight * index;
    const selectedRow = command.id === selected;
    addPanel(
      content,
      selectedRow ? 'selected-row' : 'command-row',
      { x: 0.05, y, width: 0.9, height: Math.max(0.13, rowHeight - 0.02) },
      selectedRow ? 'accent' : 'surface',
    );
    addText(
      content,
      command.fields['label'] ?? command.id,
      {
        x: 0.09,
        y: y + 0.02,
        width: 0.56,
        height: Math.max(0.09, rowHeight - 0.04),
      },
      'primary',
      selectedRow ? '600' : '400',
      12,
    );
    const shortcut = command.fields['shortcut'];
    if (shortcut)
      addText(
        content,
        shortcut,
        {
          x: 0.69,
          y: y + 0.02,
          width: 0.19,
          height: Math.max(0.09, rowHeight - 0.04),
        },
        'muted',
        '400',
        12,
      );
  }
  if (visibleCommands.length === 0) {
    const empty = sourceString(props, 'empty_text');
    if (empty)
      addText(
        content,
        empty,
        { x: 0.09, y: 0.36, width: 0.76, height: 0.16 },
        'muted',
        '400',
        12,
      );
  }
  return populatedContent(content);
}

function notificationCenterContent(
  props: ZuiTable,
): NativePainterContent | null {
  const content = emptyContent();
  const title = sourceString(props, 'title');
  if (title) {
    addPanel(
      content,
      'header',
      { x: 0.05, y: 0.06, width: 0.9, height: 0.15 },
      'surface',
    );
    addText(
      content,
      title,
      { x: 0.1, y: 0.08, width: 0.7, height: 0.11 },
      'primary',
      '600',
      14,
    );
  }
  const limit = Math.max(
    1,
    Math.min(
      MAX_PREVIEW_ROWS,
      sourceInteger(props, 'visible_limit') ?? MAX_PREVIEW_ROWS,
    ),
  );
  const notifications = sourceStrings(props, 'notifications')
    .map(parseDelimitedRecord)
    .filter(
      (notification): notification is DelimitedRecord => notification !== null,
    )
    .slice(0, limit);
  const selectedId = sourceString(props, 'selected_notification_id');
  const rowHeight = notifications.length ? 0.67 / notifications.length : 0;
  for (const [index, notification] of notifications.entries()) {
    const y = 0.25 + rowHeight * index;
    const severity = notification.fields['severity'];
    addPanel(
      content,
      'notification-row',
      { x: 0.1, y, width: 0.82, height: Math.max(0.16, rowHeight - 0.02) },
      notification.id === selectedId ? 'selected' : 'surface',
    );
    if (severity)
      addPanel(
        content,
        `status-${severity}`,
        { x: 0.13, y: y + 0.06, width: 0.025, height: 0.045 },
        severityTone(severity),
      );
    const rowTitle = notification.fields['title'];
    if (rowTitle)
      addText(
        content,
        rowTitle,
        { x: 0.19, y: y + 0.025, width: 0.65, height: 0.07 },
        'primary',
        '600',
        12,
      );
    const message = notification.fields['message'];
    if (message)
      addText(
        content,
        message,
        { x: 0.19, y: y + 0.1, width: 0.65, height: 0.06 },
        'muted',
        '400',
        12,
      );
  }
  if (notifications.length === 0) {
    const empty = sourceString(props, 'empty_text');
    if (empty)
      addText(
        content,
        empty,
        { x: 0.1, y: 0.37, width: 0.76, height: 0.16 },
        'muted',
        '400',
        12,
      );
  }
  return populatedContent(content);
}

function dragOverlayContent(props: ZuiTable): NativePainterContent | null {
  const content = emptyContent();
  const label = firstString(props, ['payload_label', 'text']);
  const reference = sourceString(props, 'payload_reference');
  const indicator = sourceString(props, 'drop_indicator_text');
  const dropTone =
    sourceBoolean(props, 'drop_allowed') === false ? 'danger' : 'accent';
  if (label || reference) {
    addPanel(
      content,
      'drag-payload',
      { x: 0.08, y: 0.16, width: 0.84, height: 0.32 },
      'surface',
    );
    if (label)
      addText(
        content,
        label,
        { x: 0.14, y: 0.21, width: 0.7, height: 0.1 },
        'primary',
        '600',
        14,
      );
    if (reference)
      addText(
        content,
        reference,
        { x: 0.14, y: 0.33, width: 0.7, height: 0.08 },
        'muted',
        '400',
        12,
      );
  }
  if (indicator) {
    addPanel(
      content,
      'drop-indicator',
      { x: 0.08, y: 0.62, width: 0.84, height: 0.16 },
      dropTone,
    );
    addText(
      content,
      indicator,
      { x: 0.14, y: 0.64, width: 0.7, height: 0.11 },
      'primary',
      '600',
      12,
    );
  }
  return populatedContent(content);
}

function workbenchToastContent(props: ZuiTable): NativePainterContent | null {
  const content = emptyContent();
  const message = firstString(props, ['text', 'value']);
  const action = sourceString(props, 'action_label');
  if (message) {
    addPanel(
      content,
      'toast-body',
      { x: 0.05, y: 0.08, width: 0.9, height: 0.84 },
      'surface',
    );
    addText(
      content,
      message,
      // Reserve a real two-line text box. At the 240px acceptance width the
      // authored message wraps; a compact 32px host put the first glyph at a
      // negative y and the second line outside the Snackbar bounds.
      { x: 0.11, y: 0.05, width: action ? 0.52 : 0.74, height: 0.9 },
      'primary',
      '400',
      14,
    );
  }
  if (action) {
    addPanel(
      content,
      'toast-action',
      { x: 0.7, y: 0.26, width: 0.18, height: 0.18 },
      'accent',
    );
    addText(
      content,
      action,
      { x: 0.72, y: 0.28, width: 0.14, height: 0.14 },
      'primary',
      '600',
      12,
    );
  }
  return populatedContent(content);
}

function emptyContent(): NativePainterContent {
  return { panels: [], texts: [], icons: [] };
}

function populatedContent(
  content: NativePainterContent,
): NativePainterContent | null {
  return content.panels.length ||
    content.texts.length ||
    (content.icons?.length ?? 0)
    ? content
    : null;
}

function addFocusRing(content: NativePainterContent): NativePainterContent {
  const insetX = 0.006;
  const insetY = 0.012;
  const thicknessX = 0.006;
  const thicknessY = 0.012;
  addPanel(
    content,
    'focus-ring-top',
    { x: insetX, y: insetY, width: 1 - insetX * 2, height: thicknessY },
    'accent',
  );
  addPanel(
    content,
    'focus-ring-bottom',
    {
      x: insetX,
      y: 1 - insetY - thicknessY,
      width: 1 - insetX * 2,
      height: thicknessY,
    },
    'accent',
  );
  addPanel(
    content,
    'focus-ring-left',
    {
      x: insetX,
      y: insetY + thicknessY,
      width: thicknessX,
      height: 1 - (insetY + thicknessY) * 2,
    },
    'accent',
  );
  addPanel(
    content,
    'focus-ring-right',
    {
      x: 1 - insetX - thicknessX,
      y: insetY + thicknessY,
      width: thicknessX,
      height: 1 - (insetY + thicknessY) * 2,
    },
    'accent',
  );
  return content;
}

function addPanel(
  content: NativePainterContent,
  kind: string,
  placement: NativePainterPlacement,
  tone: NativePainterTone,
): void {
  content.panels.push({ kind, placement, tone });
}

function addText(
  content: NativePainterContent,
  value: string,
  placement: NativePainterPlacement,
  tone: NativePainterTone,
  weight: '400' | '600',
  size: number,
): void {
  content.texts.push({ value, placement, tone, weight, size });
}

function addIcon(
  content: NativePainterContent,
  name: string,
  placement: NativePainterPlacement,
  tone: NativePainterTone,
  size: NativePainterIconSize,
): void {
  content.icons ??= [];
  content.icons.push({
    name: normalizeNativeIconName(name),
    placement,
    tone,
    size,
  });
}

function sourceIconSize(
  props: ZuiTable,
  property: string,
): NativePainterIconSize | null {
  const value = sourceString(props, property);
  if (value === null) return 's';
  return isNativeIconSize(value) ? value : null;
}

function isNativeIconSize(value: string): value is NativePainterIconSize {
  return value === 's' || value === 'm' || value === 'l' || value === 'xl';
}

function normalizeNativeIconName(value: string): string {
  const raw = value
    .trim()
    .split('@', 1)[0]
    .split('/')
    .at(-1)
    ?.replace(/\.svg$/i, '')
    .replaceAll('_', '-')
    .toLowerCase();
  const aliases: Record<string, string> = {
    send: 'arrow-up',
    'send-arrow': 'arrow-up',
    'chevron-up': 'chevron-up',
  };
  return aliases[raw ?? ''] ?? raw ?? '';
}

function isSupportedNativeIcon(value: string): boolean {
  const name = normalizeNativeIconName(value);
  return name.length > 0 && name in featherIcons;
}

function sourceString(props: ZuiTable, property: string): string | null {
  const value = props[property];
  return typeof value === 'string' && value.trim() ? value : null;
}

function firstString(props: ZuiTable, properties: string[]): string | null {
  for (const property of properties) {
    const value = sourceString(props, property);
    if (value) return value;
  }
  return null;
}

function sourceStrings(props: ZuiTable, property: string): string[] {
  const value = props[property];
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === 'string')
    : [];
}

function sourceInteger(props: ZuiTable, property: string): number | null {
  const value = props[property];
  return typeof value === 'number' && Number.isFinite(value)
    ? Math.floor(value)
    : null;
}

function sourceBoolean(props: ZuiTable, property: string): boolean | null {
  const value = props[property];
  return typeof value === 'boolean' ? value : null;
}

interface DelimitedRecord {
  id: string;
  fields: Record<string, string>;
}

function parseDelimitedRecord(value: string): DelimitedRecord | null {
  const [rawId, ...fields] = value.split('|');
  const id = rawId.trim();
  if (!id) return null;
  const parsed: Record<string, string> = {};
  for (const field of fields) {
    const separator = field.indexOf('=');
    if (separator <= 0) continue;
    const key = field.slice(0, separator).trim();
    const fieldValue = field.slice(separator + 1).trim();
    if (key && fieldValue) parsed[key] = fieldValue;
  }
  return { id, fields: parsed };
}

function severityTone(severity: string): NativePainterTone {
  switch (severity.toLowerCase()) {
    case 'error':
    case 'danger':
      return 'danger';
    case 'warning':
      return 'warning';
    case 'success':
      return 'success';
    case 'info':
      return 'info';
    default:
      return 'accent';
  }
}
