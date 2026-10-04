import type { ZuiNode } from './zui-document';
import type { ProjectionSourcePart } from './penpot-projection-model';
import { nativePainterComponent } from './zui-native-painter-role';

const CANVAS_VARIANT_PARTS: Record<string, string[]> = {
  'sample-grid': [
    'x_ticks',
    'y_ticks',
    'sample_points',
    'x_axis_label',
    'y_axis_label',
  ],
  'timeline-strip': [
    'duration',
    'current_time',
    'tick_interval',
    'timeline_keys',
    'track_label',
  ],
  'weight-heatmap': [
    'heatmap_columns',
    'heatmap_rows',
    'heat_sources',
    'low_label',
    'high_label',
  ],
};

const PAINTER_PARTS: Record<string, string[]> = {
  TreeRow: [
    'text',
    'icon',
    'component_variant',
    'tree_depth',
    'tree_indent_px',
    'expanded',
    'selected',
    'checked',
    'font_size',
    'font_weight',
    'font_family',
    'line_height_ratio',
    'foreground_color',
    'selected_foreground_color',
    'disabled_foreground_color',
    'icon_color',
    'selected_icon_color',
  ],
  AgentChat: ['messages', 'composer_text', 'streaming', 'error'],
  AgentPlan: [
    'text',
    'title',
    'collection_items',
    'steps',
    'value',
    'min',
    'max',
  ],
  AgentApproval: [
    'text',
    'title',
    'message',
    'scope',
    'value_text',
    'label_text',
    'options',
    'allow_label',
    'deny_label',
    'approval_state',
    'destructive',
  ],
  AIUsage: ['text', 'title', 'value_text', 'value', 'min', 'max', 'detail'],
  ChatComposer: [
    'composer_text',
    'streaming',
    'error',
    'send_icon',
    'send_icon_size',
    'send_icon_color',
  ],
  DataGrid: [
    'text',
    'title',
    'options',
    'columns',
    'collection_items',
    'rows',
    'empty_text',
    'value_text',
    'selected_row_id',
    'density',
    'pagination_page_index',
    'pagination_page_size',
    'pagination_total_count',
  ],
  CommandPalette: [
    'query',
    'placeholder',
    'window_count',
    'window_offset',
    'total_count',
  ],
  ConfirmDialog: [
    'title',
    'message',
    'confirm_action_color',
    'cancel_action_color',
  ],
  Dialog: ['title', 'message', 'open', 'popup_open'],
  DragOverlay: [
    'payload_kind',
    'payload_label',
    'payload_reference',
    'drop_indicator_text',
    'drop_allowed',
  ],
  NotificationCenter: [
    'title',
    'notifications',
    'empty_text',
    'unread_count',
    'visible_limit',
  ],
  ToolCalls: ['text', 'title', 'collection_items', 'calls', 'visible_limit'],
  TreeView: [
    'text',
    'title',
    'collection_items',
    'items',
    'empty_text',
    'value_text',
    'expanded',
    'selected_id',
    'query',
    'density',
    'tree_depth',
    'selected',
    'checked',
  ],
  WorkbenchToast: [
    'text',
    'value',
    'severity',
    'action_label',
    'current_toast_id',
  ],
};

/**
 * Return the authored source properties represented by a native painter.
 * The owner remains the original semantic ZUI node; no synthetic children
 * are introduced, so edits can be reconciled back to the same source node.
 */
export function nativePainterSourceParts(
  node: ZuiNode,
  nodeId: string,
): ProjectionSourcePart[] {
  const variant = String(node.props?.['component_variant'] ?? '')
    .split(/\s+/)
    .find((value) => value in CANVAS_VARIANT_PARTS);
  const properties = variant
    ? CANVAS_VARIANT_PARTS[variant]
    : (PAINTER_PARTS[nativePainterComponent(node) ?? ''] ?? []);
  const props = node.props ?? {};
  return properties
    .filter((property) => Object.prototype.hasOwnProperty.call(props, property))
    .map((property) => ({ nodeId, property }));
}

export function hasNativePainterSourceParts(node: ZuiNode): boolean {
  return nativePainterSourceParts(node, '').length > 0;
}
