use crate::core::framework::render::{RichTable, RichTableCell, MAX_RICH_TABLE_ROW_SPAN};

#[derive(Clone, Copy, Debug)]
pub(super) struct PlacedTableCell<'a> {
    pub cell: &'a RichTableCell,
    pub row: usize,
    pub column: usize,
    pub row_span: usize,
    pub column_span: usize,
}

#[derive(Clone, Debug)]
pub(super) struct TableGrid<'a> {
    pub column_count: usize,
    pub row_count: usize,
    pub cells: Vec<PlacedTableCell<'a>>,
}

impl<'a> TableGrid<'a> {
    /// Projects resolved parser coordinates into allocation-safe layout indices.
    pub(super) fn from_table(table: &'a RichTable) -> Self {
        let column_count = table.columns.len().max(1);
        let row_limit = table
            .cells
            .len()
            .saturating_mul(usize::from(MAX_RICH_TABLE_ROW_SPAN))
            .max(1);
        let mut row_count = 0;
        let cells = table
            .cells
            .iter()
            .map(|cell| {
                let column = usize::from(cell.column_index).min(column_count - 1);
                let column_span = usize::from(cell.column_span.max(1))
                    .min(column_count.saturating_sub(column))
                    .max(1);
                let row = usize::try_from(cell.row_index)
                    .unwrap_or(row_limit - 1)
                    .min(row_limit - 1);
                let row_span = usize::from(cell.row_span.max(1))
                    .min(usize::from(MAX_RICH_TABLE_ROW_SPAN))
                    .min(row_limit.saturating_sub(row))
                    .max(1);
                row_count = row_count.max(row.saturating_add(row_span));
                PlacedTableCell {
                    cell,
                    row,
                    column,
                    row_span,
                    column_span,
                }
            })
            .collect();
        Self {
            column_count,
            row_count,
            cells,
        }
    }
}
