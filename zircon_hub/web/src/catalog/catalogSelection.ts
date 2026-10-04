export interface CatalogSelectableRow {
  id: string;
}

export function selectVisibleCatalogRow<Row extends CatalogSelectableRow>(
  visibleRows: readonly Row[],
  selectedRowId: string | null,
): Row | undefined {
  return visibleRows.find((row) => row.id === selectedRowId) ?? visibleRows[0];
}
