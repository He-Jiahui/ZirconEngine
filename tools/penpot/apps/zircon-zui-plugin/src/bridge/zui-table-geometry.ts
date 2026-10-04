import type { ControlGeometry } from './zui-control-geometry';
import type { TableProjection } from './zui-table-projection';

export const TABLE_COLUMN_SAMPLES = ['Name', 'Type', 'Size', 'Revision'];
const RATIOS = [0.36, 0.27, 0.19, 0.18];

/** Retained-host column allocation, with measurements supplied by the renderer. */
export function tableColumnWidths(
  table: TableProjection,
  available: number,
  measure: (text: string) => number,
): number[] {
  const width = Number.isFinite(available) ? Math.max(0, available) : 0;
  const cellMinimums = RATIOS.map((_, index) =>
    table.cells
      .filter((cell) => cell.column === index)
      .reduce((minimum, cell) => Math.max(minimum, measure(cell.value)), 0),
  );
  const floors = [
    table.rowHeight * 4 + table.insetX,
    table.rowHeight * 2,
    table.rowHeight * 2,
    table.rowHeight * 2 + table.gapLarge + table.insetY,
  ];
  const minimums = floors.map((floor, index) =>
    Math.max(
      floor,
      Math.max(measure(TABLE_COLUMN_SAMPLES[index]), cellMinimums[index]) +
        table.insetX * 2 +
        table.textClipGuard,
    ),
  );
  const visible = table.narrow
    ? [true, true, false, false]
    : [true, true, true, true];
  const minimum = () =>
    minimums.reduce(
      (sum, value, index) => sum + (visible[index] ? value : 0),
      0,
    );
  for (const index of [3, 2, 1]) {
    if (minimum() <= width) break;
    visible[index] = false;
  }
  if (!visible.slice(1).some(Boolean) && width < minimum())
    return [width, 0, 0, 0];
  const ratioSum = RATIOS.reduce(
    (sum, ratio, index) => sum + (visible[index] ? ratio : 0),
    0,
  );
  const widths = RATIOS.map((ratio, index) =>
    visible[index] ? Math.max(minimums[index], (width * ratio) / ratioSum) : 0,
  );
  const overflow = widths.reduce((sum, value) => sum + value, 0) - width;
  const flexible = widths.reduce(
    (sum, value, index) => sum + Math.max(0, value - minimums[index]),
    0,
  );
  return widths.map((value, index) =>
    overflow > 0 && flexible > 0 && value > minimums[index]
      ? Math.max(
          minimums[index],
          value - (overflow * (value - minimums[index])) / flexible,
        )
      : value,
  );
}

export function tableGeometry(
  table: TableProjection,
  width: number,
  height: number,
  measure: (text: string) => number,
): ControlGeometry {
  const available = Math.max(0, width - table.insetX * 2 - table.actionWidth);
  const widths = tableColumnWidths(table, available, measure);
  const textHeight = Math.max(0, height - table.insetY * 2);
  const texts: ControlGeometry['texts'] = {};
  let offset = 0;
  for (let index = 0; index < 4; index++) {
    const columnWidth = widths[index];
    const inset = Math.min(table.insetX, columnWidth * 0.5);
    const cell = table.cells.find((cell) => cell.column === index);
    if (cell && columnWidth > 0 && textHeight > 0) {
      const slotWidth = Math.max(0, columnWidth - inset * 2);
      const textWidth =
        index >= 2 ? Math.min(slotWidth, measure(cell.value)) : slotWidth;
      if (textWidth > 0)
        texts[cell.key] = {
          x:
            table.insetX +
            table.offsetX +
            offset +
            table.cellOffsets[index] +
            inset +
            (index >= 2 ? slotWidth - textWidth : 0),
          y: table.insetY + table.offsetY,
          width: textWidth,
          height: textHeight,
        };
    }
    offset += columnWidth;
  }
  return { parts: {}, texts };
}
