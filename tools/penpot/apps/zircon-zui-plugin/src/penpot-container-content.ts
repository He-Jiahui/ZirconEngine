import type {
  ProjectedZuiNode,
  ProjectionPadding,
} from './bridge/penpot-projection-model';
import type {
  ZuiChildMount,
  ZuiDocument,
  ZuiNode,
  ZuiTable,
} from './bridge/zui-document';
import { resolveDesignNumber } from './bridge/zui-prefab-system';
import type { LinearDesiredSize } from './penpot-weighted-flex';

export interface ContainerDesiredChild {
  node: ZuiNode;
  mount?: ZuiChildMount;
  desired: LinearDesiredSize;
  padding?: ProjectionPadding;
}
function table(value: unknown): ZuiTable | undefined {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
    ? (value as ZuiTable)
    : undefined;
}
function num(document: ZuiDocument, value: unknown, fallback = 0): number {
  return resolveDesignNumber(document, value) ?? fallback;
}
function extent(
  child: ContainerDesiredChild,
  includePadding = true,
): LinearDesiredSize {
  return {
    width:
      child.desired.width +
      (includePadding
        ? Math.max(0, (child.padding?.left ?? 0) + (child.padding?.right ?? 0))
        : 0),
    height:
      child.desired.height +
      (includePadding
        ? Math.max(0, (child.padding?.top ?? 0) + (child.padding?.bottom ?? 0))
        : 0),
  };
}
function placement(
  document: ZuiDocument,
  child: ContainerDesiredChild,
  index: number,
  columns: number,
) {
  const slot = table(child.mount?.slot?.['layout']);
  if (slot)
    return {
      column: Math.max(0, Math.floor(num(document, slot['column']))),
      row: Math.max(0, Math.floor(num(document, slot['row']))),
      columnSpan: Math.max(
        1,
        Math.floor(num(document, slot['column_span'], 1)),
      ),
      rowSpan: Math.max(1, Math.floor(num(document, slot['row_span'], 1))),
    };
  const props = child.node.props;
  const size = props?.['size'];
  const offset = props?.['offset'];
  if (typeof size === 'number' || typeof offset === 'number')
    return {
      column: Math.max(0, Math.floor(num(document, offset))),
      row: 0,
      columnSpan: Math.max(1, Math.floor(num(document, size, 1))),
      rowSpan: 1,
    };
  return {
    column: index % columns,
    row: Math.floor(index / columns),
    columnSpan: 1,
    rowSpan: 1,
  };
}
/** Mirrors native container measurement before any allocated Penpot extents exist. */
export function measureNativeContainerContent(
  document: ZuiDocument,
  node: ZuiNode,
  projected: ProjectedZuiNode,
  children: ContainerDesiredChild[],
  availableWrapWidth: number,
  mount?: ZuiChildMount,
): LinearDesiredSize {
  const layout = projected.container;
  const raw =
    table(table(mount?.slot?.['layout'])?.['container']) ??
    table(node.layout?.['container']);
  const kind = String(raw?.['kind'] ?? node.component).toLowerCase();
  if (kind === 'space') return { width: 0, height: 0 };
  const scroll = ['scrollablebox', 'scrollbox'].includes(kind);
  const sizes = children.map((child) => extent(child, !scroll));
  if (layout.kind === 'flex' && !layout.wrap) {
    return layout.direction === 'row'
      ? {
          width:
            sizes.reduce((sum, child) => sum + child.width, 0) +
            Math.max(0, layout.columnGap) * Math.max(0, sizes.length - 1),
          height: Math.max(0, ...sizes.map((child) => child.height)),
        }
      : {
          width: Math.max(0, ...sizes.map((child) => child.width)),
          height:
            sizes.reduce((sum, child) => sum + child.height, 0) +
            Math.max(0, layout.rowGap) * Math.max(0, sizes.length - 1),
        };
  }
  if (layout.kind === 'flex' && layout.wrap) {
    const minimumWidth = num(document, raw?.['item_min_width']);
    let rowWidth = 0,
      rowHeight = 0,
      width = 0,
      height = 0,
      rowCount = 0,
      rowItems = 0;
    for (const child of children) {
      const padded = extent(child);
      const itemWidth =
        Math.max(child.desired.width, minimumWidth) +
        (padded.width - child.desired.width);
      const next =
        rowItems === 0
          ? itemWidth
          : rowWidth + Math.max(0, layout.columnGap) + itemWidth;
      if (rowItems > 0 && next > availableWrapWidth) {
        width = Math.max(width, rowWidth);
        height += rowHeight;
        rowCount++;
        rowWidth = 0;
        rowHeight = 0;
        rowItems = 0;
      }
      rowWidth +=
        (rowItems > 0 ? Math.max(0, layout.columnGap) : 0) + itemWidth;
      rowHeight = Math.max(rowHeight, padded.height);
      rowItems++;
    }
    if (rowItems) {
      width = Math.max(width, rowWidth);
      height += rowHeight;
      rowCount++;
    }
    return {
      width,
      height: height + Math.max(0, layout.rowGap) * Math.max(0, rowCount - 1),
    };
  }
  if (layout.kind === 'grid') {
    let columns = Math.max(1, layout.columns),
      rows = Math.max(1, layout.rows);
    children.forEach((child, index) => {
      const cell = placement(document, child, index, columns);
      columns = Math.max(columns, cell.column + cell.columnSpan);
      rows = Math.max(rows, cell.row + cell.rowSpan);
    });
    const widths = Array<number>(columns).fill(0),
      heights = Array<number>(rows).fill(0);
    children.forEach((child, index) => {
      const cell = placement(document, child, index, columns),
        padded = sizes[index];
      const column = Math.min(cell.column, columns - 1),
        row = Math.min(cell.row, rows - 1);
      const columnSpan = Math.min(cell.columnSpan, columns - column),
        rowSpan = Math.min(cell.rowSpan, rows - row);
      for (let i = column; i < column + columnSpan; i++)
        widths[i] = Math.max(widths[i], padded.width / columnSpan);
      for (let i = row; i < row + rowSpan; i++)
        heights[i] = Math.max(heights[i], padded.height / rowSpan);
    });
    return {
      width:
        widths.reduce((sum, value) => sum + value, 0) +
        Math.max(0, layout.columnGap) * (columns - 1),
      height:
        heights.reduce((sum, value) => sum + value, 0) +
        Math.max(0, layout.rowGap) * (rows - 1),
    };
  }
  return {
    width: Math.max(0, ...sizes.map((child) => child.width)),
    height: Math.max(0, ...sizes.map((child) => child.height)),
  };
}
