import type { ZuiNode } from './zui-document';

/** Runtime-owned painter names that remain stable across prefab expansion. */
export const NATIVE_PAINTER_COMPONENTS = new Set([
  'AgentChat',
  'AgentPlan',
  'AgentApproval',
  'AIUsage',
  'ChatComposer',
  'DataGrid',
  'TreeView',
  'TreeRow',
  'CommandPalette',
  'ConfirmDialog',
  'Dialog',
  'DragOverlay',
  'NotificationCenter',
  'ToolCalls',
  'WorkbenchToast',
]);

/**
 * Resolve the authored painter role for a node.
 *
 * LayoutDependencies materializes a prefab using the prefab root's runtime
 * component (for example `Snackbar`) and records the authored component in
 * `penpot_prefab_source`.  Review and projection code must use that source
 * identity instead of treating the materialized root as an unrelated generic
 * control.
 */
export function nativePainterComponent(node?: ZuiNode): string | undefined {
  if (!node) return undefined;
  const direct = canonicalPainterName(node.component);
  if (direct) return direct;
  const source = node['penpot_prefab_source'];
  if (typeof source !== 'string') return undefined;
  return canonicalPainterName(source.split('#').at(-1));
}

function canonicalPainterName(value: unknown): string | undefined {
  if (typeof value !== 'string') return undefined;
  const candidate =
    value.trim() === 'WorkbenchTreeRow' ? 'TreeRow' : value.trim();
  return NATIVE_PAINTER_COMPONENTS.has(candidate) ? candidate : undefined;
}
