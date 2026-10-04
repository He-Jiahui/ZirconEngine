import type {
  ZuiChildMount,
  ZuiDocument,
  ZuiNode,
  ZuiTable,
} from './zui-document';
import type {
  ProjectionContainer,
  ProjectionPadding,
} from './penpot-projection-model';
import { resolveDesignNumber } from './zui-prefab-system';
import { hasNativePainterSourceParts } from './zui-native-painter-projection';

const SIDES = ['top', 'right', 'bottom', 'left'] as const;
const WORKBENCH_WINDOW_SOURCE_PATH =
  'zircon_editor/assets/ui/editor/windows/workbench_window.zui';
const WORKBENCH_PREFERENCES_SOURCE_PATH =
  'zircon_editor/assets/ui/editor/components/workbench/floating/workbench_preferences.zui';
const table = (value: unknown): ZuiTable | undefined =>
  value !== null && typeof value === 'object' && !Array.isArray(value)
    ? (value as ZuiTable)
    : undefined;

export function projectSlotPadding(
  document: ZuiDocument,
  nodeId: string,
  mount: ZuiChildMount | undefined,
  parentKind: ProjectionContainer['kind'] | undefined,
  sourceNode?: ZuiNode,
): ProjectionPadding | undefined {
  if (!mount) return undefined;
  const raw = table(mount.slot?.['layout'])?.['padding'];
  // Auto-layout children have an implicit editable zero margin even when the
  // source omits `padding`. Keep that baseline so an editor can make a
  // deliberate margin edit and have it written back to the slot. Free/overlay
  // children have no such implicit mapping; do not invent one for them.
  if (raw === undefined) {
    if (parentKind === 'flex' || parentKind === 'grid')
      return { top: 0, right: 0, bottom: 0, left: 0 };
    return undefined;
  }
  const authored = table(raw);
  if (raw !== undefined && !authored)
    throw new Error(`Invalid slot padding table: ${nodeId}`);
  const padding = Object.fromEntries(
    SIDES.map((side) => {
      const value = authored?.[side];
      const resolved =
        value === undefined ? 0 : resolveDesignNumber(document, value);
      if (resolved === null || !Number.isFinite(resolved))
        throw new Error(`Unresolved slot padding: ${nodeId}.${side}`);
      return [side, resolved];
    }),
  ) as unknown as ProjectionPadding;
  if (parentKind === 'free' || parentKind === undefined) {
    if (nodeId === 'toast_overlay') return padding;
    const child = sourceNode ?? document.nodes?.[nodeId];
    if (isAuthoredSettingsWindowMapping(child)) return padding;
    const settingsWindowCandidate =
      nodeId === 'settings_window' ||
      child?.['penpot_review_source_node_id'] === 'settings_window' ||
      (child?.['penpot_review_source_path'] ===
        WORKBENCH_PREFERENCES_SOURCE_PATH &&
        child['penpot_review_source_node_id'] === 'preferences');
    // Overlay hosts use authored slot insets to position a native popup/toast.
    // Preserve that editable mapping instead of flattening it into a generic
    // rectangle or rejecting the complete workbench window.
    if (
      !settingsWindowCandidate &&
      child &&
      (hasNativePainterSourceParts(child) ||
        [
          'WorkbenchToast',
          'WorkbenchNotificationCenter',
          'WorkbenchDialog',
        ].includes(
          child.component,
        ))
    )
      return padding;
    if (Object.values(padding).some((value) => value !== 0))
      throw new Error(
        `Slot padding on a free/overlay parent requires an explicit mapping: ${nodeId}`,
      );
    return undefined;
  }
  return padding;
}

function isAuthoredSettingsWindowMapping(node: ZuiNode | undefined): boolean {
  if (!node) return false;
  const sourcePath = node['penpot_review_source_path'];
  const sourceNodeId = node['penpot_review_source_node_id'];
  if (
    !(
      (sourcePath === WORKBENCH_WINDOW_SOURCE_PATH &&
        sourceNodeId === 'settings_window') ||
      (sourcePath === WORKBENCH_PREFERENCES_SOURCE_PATH &&
        sourceNodeId === 'preferences')
    )
  )
    return false;
  const instancePath = node['penpot_review_instance_path'];
  if (typeof instancePath !== 'string') return false;
  try {
    const parsed = JSON.parse(instancePath) as unknown;
    if (!Array.isArray(parsed) || JSON.stringify(parsed) !== instancePath)
      return false;
    if (
      sourcePath === WORKBENCH_WINDOW_SOURCE_PATH &&
      sourceNodeId === 'settings_window'
    )
      return true;
    return parsed.some(
      (step: unknown) =>
        step !== null &&
        typeof step === 'object' &&
        !Array.isArray(step) &&
        (step as Record<string, unknown>)['sourcePath'] ===
          WORKBENCH_WINDOW_SOURCE_PATH &&
        (step as Record<string, unknown>)['sourceNodeId'] ===
          'settings_window',
    );
  } catch {
    return false;
  }
}

export function reconcileSlotPadding(
  nodeId: string,
  baseline: ProjectionPadding | undefined,
  current: ProjectionPadding | undefined,
  desiredMount: { parentId: string; mount: ZuiChildMount } | undefined,
  parentBecameFree: boolean,
  changes: string[],
): void {
  const changed = SIDES.filter(
    (side) =>
      Math.abs((current?.[side] ?? 0) - (baseline?.[side] ?? 0)) > 0.001,
  );
  if (!changed.length) return;
  if (!baseline || !current || !desiredMount || parentBecameFree)
    throw new Error(
      `Slot padding edit requires a mapped Flex/Grid child: ${nodeId}`,
    );
  const slot = (desiredMount.mount.slot ??= {});
  const layout = table(slot['layout']) ?? {};
  slot['layout'] = layout;
  const padding = table(layout['padding']) ?? {};
  layout['padding'] = padding;
  for (const side of changed) {
    padding[side] = Math.round(current[side] * 1000) / 1000;
    changes.push(
      `nodes.${desiredMount.parentId}.children.${nodeId}.slot.layout.padding.${side}`,
    );
  }
}
