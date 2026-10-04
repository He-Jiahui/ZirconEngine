import type { ZuiNode, ZuiTable } from './zui-document';

export interface TableCell {
  key: string;
  column: number;
  sourceIndex: number | null;
  value: string;
}

export function isWorkbenchTable(node: ZuiNode): boolean {
  return (
    ['Table', 'TableRow', 'WorkbenchTableRow'].includes(node.component) &&
    (node.component.startsWith('Workbench') ||
      node.control_id?.startsWith('Workbench') === true ||
      node.classes?.some((name) => name.startsWith('workbench-')) === true)
  );
}

/** Mirrors retained-host template_table_rows/cells/text.rs. */
export function normalizeTableCell(column: number, value: string): string {
  const trimmed = value.trim();
  if (column === 1) {
    if (/^tex$/i.test(trimmed)) return 'Texture';
    if (/^mat$/i.test(trimmed)) return 'Material';
  }
  if (column === 2) {
    const size = /^([\d.]+)\s*(b|k|kb|m|mb|g|gb)$/i.exec(trimmed);
    if (size) {
      const unit = size[2].toUpperCase();
      return `${size[1]} ${unit.length === 1 && unit !== 'B' ? `${unit}B` : unit}`;
    }
  }
  if (column === 3) {
    if (/^rev$/i.test(trimmed)) return 'Revision';
    const revision = /^(?:rev|r)\s*(\d+)$/i.exec(trimmed);
    if (revision) return `rev ${revision[1]}`;
  }
  return trimmed;
}

function archivedCells(value: string): string[] {
  const words = value.trim().split(/\s+/);
  if (!value.trim()) return [];
  const [name, kind, size, fourth, fifth, sixth] = words;
  if (words.length >= 5 && /^(b|k|kb|m|mb|g|gb)$/i.test(fourth))
    return [
      name,
      kind,
      `${size} ${fourth}`,
      sixth ? `${fifth} ${sixth}` : fifth,
    ];
  if (words.length >= 5 && /^(r|rev)$/i.test(fourth))
    return [name, kind, size, `${fourth} ${fifth}`];
  if (words.length >= 6)
    return [name, kind, `${size} ${fourth}`, `${fifth} ${sixth}`];
  if (words.length >= 4) return [name, kind, size, fourth];
  return [value.trim()];
}

export function tableCells(props: ZuiTable): TableCell[] {
  const options = Array.isArray(props['options']) ? props['options'] : [];
  const cells = options.flatMap((value, sourceIndex) =>
    typeof value === 'string' && value.trim() ? [{ value, sourceIndex }] : [],
  );
  const declared =
    cells.length > 0 &&
    cells.filter(({ value }) => value.trim().split(/\s+/).length >= 4).length *
      2 <=
      cells.length;
  const values = declared
    ? cells
    : archivedCells(typeof props['text'] === 'string' ? props['text'] : '').map(
        (value) => ({ value, sourceIndex: null }),
      );
  return values.map(({ value, sourceIndex }, column) => ({
    key:
      sourceIndex === null ? `archived-cell-${column}` : `cell-${sourceIndex}`,
    column,
    sourceIndex,
    value: normalizeTableCell(column, value),
  }));
}
