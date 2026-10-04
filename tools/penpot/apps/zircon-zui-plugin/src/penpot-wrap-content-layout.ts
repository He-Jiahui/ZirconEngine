import { allocateNativeLayoutSize } from './penpot-layout-allocation';
import type { Board } from '@penpot/plugin-types';
import type { ProjectionContainer } from './bridge/penpot-projection-model';
import type { ZuiDocument, ZuiTable } from './bridge/zui-document';
import { resolveDesignNumber } from './bridge/zui-prefab-system';
import { applySlotPadding } from './penpot-slot-padding';
import {
  solveWeightedFlexWidths,
  type LinearDesiredSize,
} from './penpot-weighted-flex';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_NODE,
} from './metadata';

function table(value: unknown): ZuiTable | undefined {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
    ? (value as ZuiTable)
    : undefined;
}
function number(document: ZuiDocument, value: unknown, fallback = 0): number {
  return resolveDesignNumber(document, value) ?? fallback;
}
function axis(document: ZuiDocument, value: ZuiTable | undefined) {
  const min = Math.max(0, number(document, value?.['min']));
  const max = number(document, value?.['max'], -1);
  return {
    min,
    max: max < 0 ? Infinity : Math.max(min, max),
    preferred: Math.max(
      min,
      Math.min(
        max < 0 ? Infinity : Math.max(min, max),
        number(document, value?.['preferred']),
      ),
    ),
    priority: number(document, value?.['priority']),
    weight: Math.max(0, number(document, value?.['weight'], 1)),
    fixed: value?.['stretch'] === 'Fixed',
    explicitStretch: value?.['stretch'] === 'Stretch',
  };
}

/** Native Wrap keeps each row's intrinsic cross extent while Flex places the flow. */
export function applyNativeWrapContentLayout(
  parent: Board,
  document: ZuiDocument,
  container: ProjectionContainer | undefined,
  desiredSizes: ReadonlyMap<string, LinearDesiredSize>,
): void {
  if (
    container?.kind !== 'flex' ||
    !container.wrap ||
    container.direction !== 'row'
  )
    return;
  const parentId = parent.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_NODE_ID,
  );
  const node = document.nodes?.[parentId],
    mounts = node?.children ?? [];
  const raw = table(node?.layout?.['container']);
  const minimumWidth = Math.max(0, number(document, raw?.['item_min_width']));
  const availableWidth = Math.max(
    0,
    parent.width - container.padding.left - container.padding.right,
  );
  const entries = parent.children
    .filter(
      (shape): shape is Board =>
        shape.type === 'board' &&
        !shape.hidden &&
        !shape.layoutChild?.absolute &&
        shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) ===
          ZUI_ROLE_NODE,
    )
    .map((child) => {
      const id = child.getSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        ZUI_METADATA_NODE_ID,
      );
      const childNode = document.nodes?.[id],
        desired = desiredSizes.get(id);
      const mount = mounts.find((item) => item.node === id),
        slot = table(mount?.slot?.['layout']);
      if (!childNode || !desired || !child.layoutChild || !mount)
        throw new Error('Missing intrinsic Wrap child: ' + id);
      // Native Wrap restores the node's main-axis table; slot cross-axis tables retain precedence.
      const width = axis(
        document,
        table(childNode.layout?.['width']) ?? table(slot?.['width']),
      );
      const height = axis(
        document,
        table(slot?.['height']) ?? table(childNode.layout?.['height']),
      );
      for (const key of ['anchor', 'pivot', 'position']) {
        const point = table(slot?.[key]) ?? table(childNode.layout?.[key]);
        if (
          number(document, point?.['x']) !== 0 ||
          number(document, point?.['y']) !== 0
        )
          throw new Error('Unsupported native Wrap child placement: ' + id);
      }
      const alignment = table(slot?.['alignment']);
      const horizontal = String(alignment?.['horizontal'] ?? 'Start');
      const vertical = String(alignment?.['vertical'] ?? 'Start');
      if (
        !['Start', 'Fill'].includes(horizontal) ||
        !['Start', 'Fill', 'Center', 'End'].includes(vertical) ||
        (!height.fixed && ['Center', 'End'].includes(vertical))
      )
        throw new Error('Unsupported native Wrap slot alignment: ' + id);
      const alignSelf: 'start' | 'center' | 'end' =
        vertical === 'Center' ? 'center' : vertical === 'End' ? 'end' : 'start';
      const rawPadding = table(slot?.['padding']);
      const padding = {
        left: number(document, rawPadding?.['left']),
        right: number(document, rawPadding?.['right']),
        top: number(document, rawPadding?.['top']),
        bottom: number(document, rawPadding?.['bottom']),
      };
      if (
        Object.values(padding).some(
          (value) => value < 0 || !Number.isFinite(value),
        ) ||
        !Number.isFinite(desired.width) ||
        !Number.isFinite(desired.height)
      )
        throw new Error('Unsupported native Wrap content or padding: ' + id);
      const intrinsicHeight = Math.max(
        height.min,
        Math.min(
          height.max,
          height.fixed
            ? Math.max(desired.height, height.preferred)
            : desired.height,
        ),
      );
      const min = Math.max(desired.width, minimumWidth, width.min);
      const grow =
        !width.fixed &&
        width.weight > 0 &&
        (width.explicitStretch || desired.width <= 0 || width.preferred > 0);
      const basis =
        width.fixed || (!width.explicitStretch && desired.width > 0)
          ? Math.max(desired.width, width.preferred)
          : width.preferred;
      return {
        child,
        desired,
        width,
        height,
        padding,
        alignSelf,
        intrinsicHeight,
        min,
        preferred: Math.max(min, Math.min(Math.max(min, width.max), basis)),
        grow,
      };
    });
  if (!entries.length) return;
  // Taffy owns supported Wrap rows; nonzero main priority selects native fallback.
  const nativeFallback = entries.some((entry) => entry.width.priority !== 0);
  const rows: (typeof entries)[] = [];
  let row: typeof entries = [],
    occupied = 0;
  for (const entry of entries) {
    const extent =
      (nativeFallback
        ? Math.max(entry.desired.width, minimumWidth)
        : entry.preferred) +
      entry.padding.left +
      entry.padding.right;
    const next =
      occupied + (row.length ? Math.max(0, container.columnGap) : 0) + extent;
    if (row.length && next > availableWidth) {
      rows.push(row);
      row = [];
      occupied = 0;
    }
    occupied += (row.length ? Math.max(0, container.columnGap) : 0) + extent;
    row.push(entry);
  }
  if (row.length) rows.push(row);
  if (parent.flex) {
    if (parent.flex.alignItems !== 'start') parent.flex.alignItems = 'start';
    if (parent.flex.alignContent !== 'start')
      parent.flex.alignContent = 'start';
  }
  for (const items of rows) {
    const rowHeight = Math.max(
      0,
      ...items.map(
        (entry) =>
          entry.intrinsicHeight + entry.padding.top + entry.padding.bottom,
      ),
    );
    const widths = nativeFallback
      ? items.map(
          (entry) =>
            Math.max(entry.desired.width, minimumWidth) +
            entry.padding.left +
            entry.padding.right,
        )
      : solveWeightedFlexWidths(
          Math.max(
            0,
            availableWidth -
              Math.max(0, container.columnGap) * (items.length - 1),
          ),
          items.map((entry) => ({
            min: entry.min + entry.padding.left + entry.padding.right,
            preferred:
              entry.preferred + entry.padding.left + entry.padding.right,
            max: Number.isFinite(entry.width.max)
              ? Math.max(entry.min, entry.width.max) +
                entry.padding.left +
                entry.padding.right
              : -1,
            priority: 0,
            weight: entry.width.weight,
            stretch: entry.grow ? 'Stretch' : 'Fixed',
          })),
        );
    items.forEach((entry, index) => {
      const width = Math.max(
        0.01,
        widths[index] - entry.padding.left - entry.padding.right,
      );
      const cross = entry.height.fixed
        ? entry.intrinsicHeight
        : rowHeight - entry.padding.top - entry.padding.bottom;
      const height = Math.max(
        0.01,
        entry.height.min,
        Math.min(entry.height.max, cross),
      );
      allocateNativeLayoutSize(entry.child, width, height);
      const child = entry.child.layoutChild!;
      if (child.alignSelf !== entry.alignSelf)
        child.alignSelf = entry.alignSelf;
      if (child.horizontalSizing !== 'fix') child.horizontalSizing = 'fix';
      if (child.verticalSizing !== 'fix') child.verticalSizing = 'fix';
      applySlotPadding(entry.child, entry.padding);
    });
  }
}
