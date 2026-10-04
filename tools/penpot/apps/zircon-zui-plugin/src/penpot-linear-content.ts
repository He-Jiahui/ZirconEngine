import type { Board, Text } from '@penpot/plugin-types';
import type {
  ZuiChildMount,
  ZuiDocument,
  ZuiNode,
  ZuiTable,
} from './bridge/zui-document';
import type {
  ProjectedZuiNode,
  ZuiAssetProjection,
} from './bridge/penpot-projection-model';
import { resolveDesignNumber } from './bridge/zui-prefab-system';
import {
  applyPenpotTextRuns,
  applyPenpotTextStyle,
  resolvePenpotTextStyle,
} from './penpot-text-style';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
} from './metadata';
import { measureNativeContainerContent } from './penpot-container-content';
import type { LinearDesiredSize } from './penpot-weighted-flex';
import {
  measureNativeLeafContent,
  measureNativeMaterialContent,
} from './penpot-content-material';

const CONTENT_MEASUREMENT = 'linear-content-measurement';
const CONTENT_SIGNATURE = 'linear-content-measurement-signature';
const ZERO_SIZE: LinearDesiredSize = { width: 0, height: 0 };

function table(value: unknown): ZuiTable | undefined {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
    ? (value as ZuiTable)
    : undefined;
}
function number(document: ZuiDocument, value: unknown, fallback = 0): number {
  return resolveDesignNumber(document, value) ?? fallback;
}
function axisTable(
  node: ZuiNode,
  mount: ZuiChildMount | undefined,
  parent: ProjectedZuiNode | undefined,
  axis: 'width' | 'height',
): ZuiTable | undefined {
  const own = table(node.layout?.[axis]),
    slot = table(table(mount?.slot?.['layout'])?.[axis]);
  const restored =
    parent?.container.kind === 'flex'
      ? parent.container.direction === 'row'
        ? 'width'
        : 'height'
      : undefined;
  return restored === axis ? (own ?? slot) : (slot ?? own);
}
function desiredAxis(
  document: ZuiDocument,
  dimension: ZuiTable | undefined,
  boundary: unknown,
  content: number,
): number {
  const min = Math.max(0, number(document, dimension?.['min']));
  const rawMax = number(document, dimension?.['max'], -1);
  const max = rawMax < 0 ? Infinity : Math.max(min, rawMax);
  const preferred = Math.max(
    min,
    Math.min(max, Math.max(0, number(document, dimension?.['preferred']))),
  );
  return Math.max(
    min,
    Math.min(
      max,
      boundary === 'Fixed' || boundary === 'ParentDirected'
        ? preferred
        : Math.max(preferred, content),
    ),
  );
}
function leafNeedsText(projection: ProjectedZuiNode): boolean {
  return (
    !projection.previewHidden &&
    projection.children.every(
      (child) => child.previewHidden || child.visualDetached,
    ) &&
    Boolean(projection.text?.characters)
  );
}
function contentMeasurements(asset: Board): Text[] {
  return asset.children.filter(
    (shape): shape is Text =>
      shape.type === 'text' &&
      Boolean(
        shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, CONTENT_MEASUREMENT),
      ),
  );
}
/** Keep intrinsic text probes current when the existing review board is reused. */
export function synchronizeLinearContentMeasurements(
  asset: Board,
  document: ZuiDocument,
  projection: ZuiAssetProjection,
): void {
  const existing = new Map(
    contentMeasurements(asset).map((text) => [
      text.getSharedPluginData(ZUI_METADATA_NAMESPACE, CONTENT_MEASUREMENT),
      text,
    ]),
  );
  const active = new Set<string>();
  for (const projected of projection.shapes) {
    if (!leafNeedsText(projected)) continue;
    const node = document.nodes?.[projected.nodeId];
    if (!node || !projected.text) continue;
    active.add(projected.nodeId);
    const style = resolvePenpotTextStyle(document, node, projected.text);
    const signature = JSON.stringify({
      text: projected.text.characters,
      style,
      runs: projected.text.richTextRuns ?? [],
    });
    let text = existing.get(projected.nodeId);
    const created = !text;
    if (!text) {
      text = penpot.createText(projected.text.characters) ?? undefined;
      if (!text)
        throw new Error(
          'Missing native content measurement for ' + projected.nodeId,
        );
      text.name = 'Content font measurement: ' + projected.nodeId;
      text.opacity = 0;
      text.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        ZUI_METADATA_ROLE,
        ZUI_ROLE_AUXILIARY,
      );
      text.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        CONTENT_MEASUREMENT,
        projected.nodeId,
      );
      asset.appendChild(text);
      if (text.layoutChild) text.layoutChild.absolute = true;
      text.x = asset.x;
      text.y = asset.y;
    }
    if (
      created ||
      text.getSharedPluginData(ZUI_METADATA_NAMESPACE, CONTENT_SIGNATURE) !==
        signature
    ) {
      if (text.characters !== projected.text.characters)
        text.characters = projected.text.characters;
      applyPenpotTextStyle(text, style);
      applyPenpotTextRuns(text, style, projected.text.richTextRuns);
      // Native intrinsic text measurement is unbounded. Authored dimensions
      // constrain the desired size afterwards; an allocated width must not wrap it.
      text.growType = 'auto-width';
      text.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        CONTENT_SIGNATURE,
        signature,
      );
    }
  }
  for (const [nodeId, text] of existing) {
    if (!active.has(nodeId)) text.remove();
  }
}
export function linearContentMeasurementsReady(asset: Board): boolean {
  return contentMeasurements(asset).every(
    ({ width, height }) =>
      Number.isFinite(width) &&
      width > 1 &&
      Number.isFinite(height) &&
      height > 1,
  );
}

/** Native desired sizes are measured bottom-up and never read allocated fill extents. */
export function measureLinearDesiredSizes(
  document: ZuiDocument,
  projection: ZuiAssetProjection,
  leafContent: (
    node: ZuiNode,
    projected: ProjectedZuiNode,
  ) => LinearDesiredSize,
): Map<string, LinearDesiredSize> {
  const result = new Map<string, LinearDesiredSize>();
  const visit = (
    projected: ProjectedZuiNode,
    parent?: ProjectedZuiNode,
    mount?: ZuiChildMount,
    detachedRoot = false,
  ): LinearDesiredSize => {
    if (
      projected.previewHidden ||
      (projected.visualDetached && !detachedRoot)
    ) {
      result.set(projected.nodeId, ZERO_SIZE);
      return ZERO_SIZE;
    }
    const node = document.nodes?.[projected.nodeId];
    if (!node) throw new Error(`Missing desired-size node ${projected.nodeId}`);
    const children = projected.children
      .filter((child) => !child.previewHidden && !child.visualDetached)
      .map((child) => {
        const childMount = node.children?.find(
          (item) => item.node === child.nodeId,
        );
        const desired = visit(child, projected, childMount);
        return {
          node: document.nodes![child.nodeId],
          mount: childMount,
          desired,
          padding: child.slotPadding,
        };
      });
    const layout = projected.container;
    const width = axisTable(node, mount, parent, 'width');
    const pref = number(document, width?.['preferred']);
    const max = number(document, width?.['max'], -1);
    const available = Math.max(
      0,
      (width?.['stretch'] === 'Fixed' && pref > 0
        ? pref
        : max >= 0
          ? max
          : pref > 0
            ? pref
            : Infinity) -
        Math.max(0, layout.padding.left + layout.padding.right),
    );
    let content = children.length
      ? measureNativeContainerContent(
          document,
          node,
          projected,
          children,
          available,
          mount,
        )
      : leafContent(node, projected);
    if (children.length)
      content =
        measureNativeMaterialContent(document, node, content) ?? content;
    const boundary =
      table(mount?.slot?.['layout'])?.['boundary'] ?? node.layout?.['boundary'];
    const size = {
      width: desiredAxis(
        document,
        axisTable(node, mount, parent, 'width'),
        boundary,
        content.width + Math.max(0, layout.padding.left + layout.padding.right),
      ),
      height: desiredAxis(
        document,
        axisTable(node, mount, parent, 'height'),
        boundary,
        content.height +
          Math.max(0, layout.padding.top + layout.padding.bottom),
      ),
    };
    result.set(projected.nodeId, size);
    return size;
  };
  for (const root of [
    ...projection.rootNodes,
    ...projection.detachedNodes,
    ...projection.shapes.filter((shape) => shape.visualDetached),
  ])
    visit(root, undefined, undefined, true);
  return result;
}
export function measuredLinearDesiredSizes(
  asset: Board,
  document: ZuiDocument,
  projection: ZuiAssetProjection,
): Map<string, LinearDesiredSize> {
  const measured = new Map(
    contentMeasurements(asset).map((text) => [
      text.getSharedPluginData(ZUI_METADATA_NAMESPACE, CONTENT_MEASUREMENT),
      { width: text.width, height: text.height },
    ]),
  );
  return measureLinearDesiredSizes(document, projection, (node, projected) => {
    if (!projected.text?.characters || node.component === 'Space')
      return measureNativeLeafContent(document, node, ZERO_SIZE);
    const size = measured.get(projected.nodeId);
    if (!size)
      throw new Error(
        `Native content font measurement unavailable: ${projected.nodeId}`,
      );
    return measureNativeLeafContent(document, node, size);
  });
}

export function linearContentMeasurementSnapshot(asset: Board): {
  nodeId: string;
  text: string;
  width: number;
  height: number;
  styleSignature: string;
}[] {
  return contentMeasurements(asset).map((shape) => ({
    nodeId: shape.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      CONTENT_MEASUREMENT,
    ),
    text: shape.characters,
    styleSignature: shape.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      CONTENT_SIGNATURE,
    ),
    width: shape.width,
    height: shape.height,
  }));
}
