import type { ZuiDocument, ZuiNode } from './zui-document';
import {
  prefabRoleForNode,
  resolveDesignNumber,
  responsivePreviewValue,
} from './zui-prefab-system';
import {
  projectZuiDocument,
  type ProjectedZuiNode,
  type ProjectionContainer,
} from './penpot-projection';

type AnyRecord = Record<string, unknown>;

interface ContentSize {
  width: number;
  height: number;
}

const COMPONENT_MAX_WIDTH = 4096;
const COMPONENT_MAX_HEIGHT = 4096;
const LAYOUT_SIZE_TOLERANCE = 0.5;

export function fitAutoLayoutContainersToContent(
  document: ZuiDocument,
  changes: Set<string>,
): void {
  const nodes = asRecord(document.nodes);
  if (!nodes || Object.keys(nodes).length === 0) return;
  const projections = new Map(
    projectZuiDocument(document).shapes.map((shape) => [shape.nodeId, shape]),
  );
  const fitted = new Map<string, ContentSize>();
  const visiting = new Set<string>();

  for (const nodeId of Object.keys(nodes)) {
    fitAutoLayoutNode(
      document,
      nodeId,
      nodes,
      projections,
      fitted,
      visiting,
      changes,
    );
  }
}

export function fitComponentRootsToContent(
  document: ZuiDocument,
  changes: Set<string>,
): void {
  if (document.asset.kind !== 'component') return;
  const nodes = asRecord(document.nodes);
  if (!nodes) return;
  const rootIds = uniqueStrings(
    Object.values(document.components ?? {})
      .map((component) => component.root)
      .filter((value): value is string => typeof value === 'string'),
  );

  for (const rootId of rootIds) {
    const root = asRecord(nodes[rootId]);
    if (!root) continue;
    const layout = asRecord(root['layout']) ?? {};
    root['layout'] = layout;
    const needsWidth = !hasConcreteDimension(
      document,
      asRecord(layout['width']),
    );
    const needsHeight = !hasConcreteDimension(
      document,
      asRecord(layout['height']),
    );
    if (!needsWidth && !needsHeight) continue;

    const estimated = estimateContentSize(
      document,
      rootId,
      nodes,
      new Set<string>(),
    );
    if (needsWidth) {
      writeFittedDimension(layout, 'width', Math.round(estimated.width));
    }
    if (needsHeight) {
      writeFittedDimension(layout, 'height', Math.round(estimated.height));
    }
    changes.add('fit-component-root-to-content');
  }
}

function fitAutoLayoutNode(
  document: ZuiDocument,
  nodeId: string,
  nodes: AnyRecord,
  projections: Map<string, ProjectedZuiNode>,
  fitted: Map<string, ContentSize>,
  visiting: Set<string>,
  changes: Set<string>,
): ContentSize {
  const cached = fitted.get(nodeId);
  if (cached) return cached;
  const projection = projections.get(nodeId);
  const node = asRecord(nodes[nodeId]);
  if (!projection || !node || visiting.has(nodeId)) {
    return projection?.geometry ?? { width: 1, height: 1 };
  }
  visiting.add(nodeId);
  const childSizes = projection.children.map((child) => ({
    projection: child,
    size: fitAutoLayoutNode(
      document,
      child.nodeId,
      nodes,
      projections,
      fitted,
      visiting,
      changes,
    ),
  }));
  visiting.delete(nodeId);

  const visibleChildren = childSizes.filter(({ projection: child }) => {
    const childNode = asRecord(nodes[child.nodeId]);
    return childNode && !isDirectPreviewHidden(childNode);
  });
  const existingLayout = asRecord(node['layout']);
  const layout = existingLayout ?? {};
  if (visibleChildren.length > 0 && projection.container.kind === 'free') {
    for (const axis of ['width', 'height'] as const) {
      const coordinate = axis === 'width' ? 'x' : 'y';
      const required = Math.max(
        ...visibleChildren.map(({ projection: child, size }) => {
          const childLayout = asRecord(
            asRecord(nodes[child.nodeId])?.['layout'],
          );
          const anchor =
            resolveDesignNumber(
              document,
              asRecord(childLayout?.['anchor'])?.[coordinate],
            ) ?? 0;
          const pivot =
            resolveDesignNumber(
              document,
              asRecord(childLayout?.['pivot'])?.[coordinate],
            ) ?? 0;
          const offset = child.geometry[coordinate];
          return Math.max(
            size[axis],
            anchor > 0 ? (size[axis] * pivot - offset) / anchor : 0,
            anchor < 1 ? (size[axis] * (1 - pivot) + offset) / (1 - anchor) : 0,
          );
        }),
      );
      if (expandLayoutDimension(document, layout, axis, required)) {
        if (!existingLayout) node['layout'] = layout;
        changes.add('fit-anchored-root-to-content');
      }
    }
  }
  if (visibleChildren.length > 0 && projection.container.kind !== 'free') {
    const required = autoLayoutContentSize(
      document,
      layout,
      projection.container,
      visibleChildren.map(({ size }) => size),
    );
    const scroll = scrollAxes(node);
    const widthChanged =
      !scroll.horizontal &&
      expandLayoutDimension(document, layout, 'width', required.width);
    const heightChanged =
      !scroll.vertical &&
      expandLayoutDimension(document, layout, 'height', required.height);
    if (widthChanged || heightChanged) {
      if (!existingLayout) node['layout'] = layout;
      changes.add('fit-auto-layout-to-content');
    }
  }

  const result = {
    width:
      layoutDimension(document, layout, 'width') ?? projection.geometry.width,
    height:
      layoutDimension(document, layout, 'height') ?? projection.geometry.height,
  };
  const width = asRecord(layout['width']);
  if (
    width?.['stretch'] === 'Stretch' &&
    !hasConcreteDimension(document, width)
  ) {
    const props = asRecord(node['props']);
    const text = props?.['text'];
    const role = projection.prefabRole;
    const content =
      !projection.children.length &&
      typeof text === 'string' &&
      ['label', 'title', 'caption'].includes(role)
        ? Math.min(
            640,
            Math.max(
              48,
              text.length * (projection.text?.fontSize ?? 12) * 0.54 + 16,
            ),
          )
        : 0;
    result.width = content;
  }
  fitted.set(nodeId, result);
  return result;
}

function autoLayoutContentSize(
  document: ZuiDocument,
  layout: AnyRecord,
  container: ProjectionContainer,
  children: ContentSize[],
): ContentSize {
  const horizontalPadding = container.padding.left + container.padding.right;
  const verticalPadding = container.padding.top + container.padding.bottom;
  if (container.kind === 'grid') {
    const grid = gridContentSize(
      document,
      { columns: container.columns },
      children,
      container.columnGap,
    );
    const rows = Math.ceil(children.length / Math.max(1, container.columns));
    const rowHeights = Array.from({ length: rows }, () => 0);
    children.forEach((child, index) => {
      const row = Math.floor(index / Math.max(1, container.columns));
      rowHeights[row] = Math.max(rowHeights[row] ?? 0, child.height);
    });
    return {
      width: grid.width + horizontalPadding,
      height:
        rowHeights.reduce((sum, value) => sum + value, 0) +
        Math.max(0, rows - 1) * container.rowGap +
        verticalPadding,
    };
  }
  if (container.wrap && container.direction === 'row') {
    return wrappedRowContentSize(document, layout, container, children);
  }
  if (container.direction === 'row') {
    return {
      width:
        children.reduce((sum, child) => sum + child.width, 0) +
        Math.max(0, children.length - 1) * container.columnGap +
        horizontalPadding,
      height:
        Math.max(...children.map((child) => child.height)) + verticalPadding,
    };
  }
  return {
    width:
      Math.max(...children.map((child) => child.width)) + horizontalPadding,
    height:
      children.reduce((sum, child) => sum + child.height, 0) +
      Math.max(0, children.length - 1) * container.rowGap +
      verticalPadding,
  };
}

function wrappedRowContentSize(
  document: ZuiDocument,
  layout: AnyRecord,
  container: ProjectionContainer,
  children: ContentSize[],
): ContentSize {
  const horizontalPadding = container.padding.left + container.padding.right;
  const availableWidth = Math.max(
    1,
    (layoutDimension(document, layout, 'width') ??
      children.reduce((sum, child) => sum + child.width, 0) +
        Math.max(0, children.length - 1) * container.columnGap +
        horizontalPadding) - horizontalPadding,
  );
  let rowWidth = 0;
  let rowHeight = 0;
  let contentWidth = 0;
  let contentHeight = 0;
  let rows = 0;
  for (const child of children) {
    const nextWidth =
      rowWidth === 0
        ? child.width
        : rowWidth + container.columnGap + child.width;
    if (rowWidth > 0 && nextWidth > availableWidth + LAYOUT_SIZE_TOLERANCE) {
      contentWidth = Math.max(contentWidth, rowWidth);
      contentHeight += rowHeight;
      rows += 1;
      rowWidth = child.width;
      rowHeight = child.height;
    } else {
      rowWidth = nextWidth;
      rowHeight = Math.max(rowHeight, child.height);
    }
  }
  if (rowWidth > 0) {
    contentWidth = Math.max(contentWidth, rowWidth);
    contentHeight += rowHeight;
    rows += 1;
  }
  return {
    width:
      Math.max(
        contentWidth,
        Math.max(...children.map((child) => child.width)),
      ) + horizontalPadding,
    height:
      contentHeight +
      Math.max(0, rows - 1) * container.rowGap +
      container.padding.top +
      container.padding.bottom,
  };
}

function expandLayoutDimension(
  document: ZuiDocument,
  layout: AnyRecord,
  axis: 'width' | 'height',
  requiredValue: number,
): boolean {
  const maximum = axis === 'width' ? COMPONENT_MAX_WIDTH : COMPONENT_MAX_HEIGHT;
  const required = Math.ceil(clamp(requiredValue, 1, maximum));
  const dimension = asRecord(layout[axis]) ?? {};
  const current = layoutDimension(document, layout, axis);
  if (axis === 'width' && dimension['stretch'] === 'Fixed' && current !== null)
    return false;
  if (current !== null && current + LAYOUT_SIZE_TOLERANCE >= required) {
    return false;
  }
  layout[axis] = dimension;
  for (const key of ['min', 'preferred', 'max']) {
    if (
      key === 'max' &&
      dimension['stretch'] === 'Stretch' &&
      dimension[key] === undefined
    )
      continue;
    const value = resolveDesignNumber(document, dimension[key]);
    if (value === null || value < required) dimension[key] = required;
  }
  if (dimension['stretch'] === undefined) dimension['stretch'] = 'Fixed';
  return true;
}

function isScrollableNode(node: AnyRecord): boolean {
  const component = stringValue(node['component'])?.toLowerCase() ?? '';
  if (component.includes('scroll')) return true;
  const props = asRecord(node['props']);
  return props?.['scroll_x'] === true || props?.['scroll_y'] === true;
}

function scrollAxes(node: AnyRecord): {
  horizontal: boolean;
  vertical: boolean;
} {
  if (!isScrollableNode(node)) return { horizontal: false, vertical: false };
  const props = asRecord(node['props']);
  const layout = asRecord(node['layout']);
  const scroll = asRecord(layout?.['scroll']);
  const axis = String(
    asRecord(layout?.['container'])?.['axis'] ?? props?.['scroll_axis'] ?? '',
  ).toLowerCase();
  const horizontal =
    scroll?.['horizontal'] === true ||
    props?.['scroll_x'] === true ||
    axis === 'horizontal' ||
    axis === 'both';
  const vertical =
    scroll?.['vertical'] === true ||
    props?.['scroll_y'] === true ||
    axis === 'vertical' ||
    axis === 'both' ||
    !horizontal;
  return { horizontal, vertical };
}

function isDirectPreviewHidden(node: AnyRecord): boolean {
  const props = asRecord(node['props']);
  const resolved = responsivePreviewValue(props?.['visibility']);
  const visibility =
    typeof resolved === 'string' ? resolved.trim().toLowerCase() : '';
  return (
    props?.['visible'] === false ||
    visibility === 'collapsed' ||
    visibility === 'hidden' ||
    visibility === 'none'
  );
}

function hasConcreteDimension(
  document: ZuiDocument,
  dimension: AnyRecord | undefined,
): boolean {
  if (!dimension) return false;
  return ['preferred', 'min', 'max'].some(
    (key) => resolveDesignNumber(document, dimension[key]) !== null,
  );
}

function writeFittedDimension(
  layout: AnyRecord,
  axis: 'width' | 'height',
  value: number,
): void {
  const dimension = asRecord(layout[axis]) ?? {};
  layout[axis] = dimension;
  const fallback = axis === 'width' ? 160 : 40;
  const maximum = axis === 'width' ? COMPONENT_MAX_WIDTH : COMPONENT_MAX_HEIGHT;
  const bounded = clamp(Number.isFinite(value) ? value : fallback, 1, maximum);
  dimension['min'] = bounded;
  dimension['preferred'] = bounded;
  if (dimension['stretch'] !== 'Stretch') dimension['max'] = bounded;
  if (dimension['stretch'] === undefined) dimension['stretch'] = 'Fixed';
}

function estimateContentSize(
  document: ZuiDocument,
  nodeId: string,
  nodes: AnyRecord,
  visited: Set<string>,
): ContentSize {
  if (visited.has(nodeId)) return { width: 1, height: 1 };
  visited.add(nodeId);
  const node = asRecord(nodes[nodeId]);
  if (!node) return { width: 1, height: 1 };
  const layout = asRecord(node['layout']);
  const explicitWidth = layoutDimension(document, layout, 'width');
  const explicitHeight = layoutDimension(document, layout, 'height');
  const children = childMounts(node);
  const childSizes = children.map(({ nodeId: childId }) =>
    estimateContentSize(document, childId, nodes, new Set(visited)),
  );
  const container = layout ? asRecord(layout['container']) : undefined;
  const rawKind =
    stringValue(container?.['kind']) ?? stringValue(node['component']) ?? '';
  const kind = rawKind.toLowerCase();
  const padding = {
    top: layoutSide(document, layout, 'top'),
    right: layoutSide(document, layout, 'right'),
    bottom: layoutSide(document, layout, 'bottom'),
    left: layoutSide(document, layout, 'left'),
  };
  const gap = resolveDesignNumber(document, container?.['gap']) ?? 8;
  let content: ContentSize;

  if (childSizes.length === 0) {
    content = leafContentSize(document, node);
  } else if (/(?:horizontal|row)/.test(kind)) {
    content = {
      width:
        childSizes.reduce((sum, child) => sum + child.width, 0) +
        Math.max(0, childSizes.length - 1) * gap,
      height: Math.max(...childSizes.map((child) => child.height)),
    };
  } else if (/(?:grid|masonry)/.test(kind)) {
    content = gridContentSize(document, container, childSizes, gap);
  } else if (/(?:overlay|free|absolute|stack)/.test(kind)) {
    content = freeContentSize(document, children, childSizes, nodes);
  } else {
    content = {
      width: Math.max(...childSizes.map((child) => child.width)),
      height:
        childSizes.reduce((sum, child) => sum + child.height, 0) +
        Math.max(0, childSizes.length - 1) * gap,
    };
  }

  return {
    width:
      explicitWidth ??
      Math.max(1, content.width + padding.left + padding.right),
    height:
      explicitHeight ??
      Math.max(1, content.height + padding.top + padding.bottom),
  };
}

function gridContentSize(
  document: ZuiDocument,
  container: AnyRecord | undefined,
  childSizes: ContentSize[],
  gap: number,
): ContentSize {
  const columns = Math.max(
    1,
    Math.round(resolveDesignNumber(document, container?.['columns']) ?? 1),
  );
  const columnWidths = Array.from({ length: columns }, () => 0);
  const rowHeights: number[] = [];
  childSizes.forEach((child, index) => {
    const column = index % columns;
    const row = Math.floor(index / columns);
    columnWidths[column] = Math.max(columnWidths[column], child.width);
    rowHeights[row] = Math.max(rowHeights[row] ?? 0, child.height);
  });
  return {
    width:
      columnWidths.reduce((sum, value) => sum + value, 0) +
      Math.max(0, columnWidths.length - 1) * gap,
    height:
      rowHeights.reduce((sum, value) => sum + value, 0) +
      Math.max(0, rowHeights.length - 1) * gap,
  };
}

interface ChildMount {
  nodeId: string;
  layout: AnyRecord | undefined;
}

function childMounts(node: AnyRecord): ChildMount[] {
  if (!Array.isArray(node['children'])) return [];
  return node['children'].flatMap((value) => {
    const mount = asRecord(value);
    const nodeId = stringValue(mount?.['node']);
    return nodeId ? [{ nodeId, layout: asRecord(mount?.['layout']) }] : [];
  });
}

function freeContentSize(
  document: ZuiDocument,
  children: ChildMount[],
  childSizes: ContentSize[],
  nodes: AnyRecord,
): ContentSize {
  let width = 0;
  let height = 0;
  childSizes.forEach((child, index) => {
    const mount = children[index];
    const childNode = asRecord(nodes[mount?.nodeId ?? '']);
    const childLayout = mount?.layout ?? asRecord(childNode?.['layout']);
    const position = childLayout
      ? asRecord(childLayout['position'])
      : undefined;
    const x = resolveDesignNumber(document, position?.['x']) ?? 0;
    const y = resolveDesignNumber(document, position?.['y']) ?? 0;
    width = Math.max(width, Math.max(0, x) + child.width);
    height = Math.max(height, Math.max(0, y) + child.height);
  });
  return { width, height };
}

function leafContentSize(document: ZuiDocument, node: AnyRecord): ContentSize {
  const component = stringValue(node['component'])?.toLowerCase() ?? '';
  const role = prefabRoleForNode(node as unknown as ZuiNode, '', false);
  if (role === 'button') return { width: 120, height: 32 };
  if (role === 'field') return { width: 220, height: 32 };
  if (role === 'toggle') return { width: 96, height: 30 };
  if (role === 'icon-button') return { width: 32, height: 32 };
  if (role === 'tab') return { width: 88, height: 30 };
  if (role === 'row') return { width: 240, height: 28 };
  if (role === 'chip') return { width: 80, height: 24 };
  if (role === 'badge') return { width: 56, height: 20 };
  if (role === 'divider') return { width: 160, height: 1 };
  if (role === 'progress') return { width: 160, height: 12 };
  if (role === 'canvas') return { width: 320, height: 180 };
  if (role === 'space') return { width: 24, height: 24 };
  if (component.includes('label') || component === 'text') {
    const props = asRecord(node['props']);
    const text =
      stringValue(props?.['text']) ?? stringValue(props?.['value_text']) ?? '';
    const fontSize = resolveDesignNumber(document, props?.['font_size']) ?? 12;
    return {
      width: Math.max(80, Math.min(420, text.length * fontSize * 0.58 + 16)),
      height: Math.max(20, fontSize * 1.45),
    };
  }
  return { width: 160, height: 40 };
}

function layoutDimension(
  document: ZuiDocument,
  layout: AnyRecord | undefined,
  axis: 'width' | 'height',
): number | null {
  const dimension = layout ? asRecord(layout[axis]) : undefined;
  if (!dimension) return null;
  return (
    resolveDesignNumber(document, dimension['preferred']) ??
    resolveDesignNumber(document, dimension['min']) ??
    resolveDesignNumber(document, dimension['max'])
  );
}

function layoutSide(
  document: ZuiDocument,
  layout: AnyRecord | undefined,
  side: 'top' | 'right' | 'bottom' | 'left',
): number {
  const padding = layout ? asRecord(layout['padding']) : undefined;
  return resolveDesignNumber(document, padding?.[side]) ?? 0;
}

function uniqueStrings(values: string[]): string[] {
  return [...new Set(values.filter((value) => value.trim() !== ''))];
}

function asRecord(value: unknown): AnyRecord | undefined {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
    ? (value as AnyRecord)
    : undefined;
}

function stringValue(value: unknown): string | undefined {
  return typeof value === 'string' ? value : undefined;
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, value));
}
