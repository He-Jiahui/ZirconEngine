import { allocateNativeLayoutSize } from './penpot-layout-allocation';
import type { Board } from '@penpot/plugin-types';
import type { ProjectionContainer } from './bridge/penpot-projection-model';
import type { ZuiDocument, ZuiTable } from './bridge/zui-document';
import { resolveDesignNumber } from './bridge/zui-prefab-system';
import { applySlotPadding } from './penpot-slot-padding';
import type { LinearDesiredSize } from './penpot-weighted-flex';
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
function clamp(
  document: ZuiDocument,
  dimension: ZuiTable | undefined,
  value: number,
): number {
  const min = Math.max(
    0,
    resolveDesignNumber(document, dimension?.['min']) ?? 0,
  );
  const max = resolveDesignNumber(document, dimension?.['max']);
  return Math.max(
    min,
    Math.min(max == null || max < 0 ? Infinity : Math.max(min, max), value),
  );
}
/** A native scroll child keeps its desired main extent while Flex places the flow. */
export function applyNativeScrollContentLayout(
  parent: Board,
  document: ZuiDocument,
  container: ProjectionContainer | undefined,
  desiredSizes: ReadonlyMap<string, LinearDesiredSize>,
): void {
  if (
    parent.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'container-kind') !==
      'scroll' ||
    container?.kind !== 'flex'
  )
    return;
  const main = container.direction === 'row' ? 'width' : 'height';
  const cross = main === 'width' ? 'height' : 'width';
  const parentId = parent.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_NODE_ID,
  );
  const mounts = document.nodes?.[parentId]?.children ?? [];
  const availableCross = Math.max(
    0,
    parent[cross] -
      (cross === 'width'
        ? container.padding.left + container.padding.right
        : container.padding.top + container.padding.bottom),
  );
  for (const shape of parent.children) {
    if (
      shape.type !== 'board' ||
      shape.hidden ||
      shape.layoutChild?.absolute ||
      shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) !==
        ZUI_ROLE_NODE
    )
      continue;
    const nodeId = shape.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      ZUI_METADATA_NODE_ID,
    );
    const node = document.nodes?.[nodeId],
      desired = desiredSizes.get(nodeId);
    if (!node || !desired || !shape.layoutChild)
      throw new Error('Missing intrinsic scroll child: ' + nodeId);
    const slot = table(
      mounts.find((mount) => mount.node === nodeId)?.slot?.['layout'],
    );
    const mainTable = table(node.layout?.[main]) ?? table(slot?.[main]);
    const crossTable = table(slot?.[cross]) ?? table(node.layout?.[cross]);
    const targetMain = Math.max(
      0.01,
      clamp(document, mainTable, desired[main]),
    );
    const targetCross = Math.max(
      0.01,
      clamp(
        document,
        crossTable,
        crossTable?.['stretch'] === 'Fixed' ? desired[cross] : availableCross,
      ),
    );
    const width = main === 'width' ? targetMain : targetCross,
      height = main === 'height' ? targetMain : targetCross;
    allocateNativeLayoutSize(shape, width, height);
    if (shape.layoutChild.horizontalSizing !== 'fix')
      shape.layoutChild.horizontalSizing = 'fix';
    if (shape.layoutChild.verticalSizing !== 'fix')
      shape.layoutChild.verticalSizing = 'fix';
    applySlotPadding(shape, { left: 0, right: 0, top: 0, bottom: 0 });
  }
}
