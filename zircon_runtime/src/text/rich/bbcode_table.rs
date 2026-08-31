use crate::text::{RichTable, RichTableCell, RichTableCellBoxStyle, RichTableColumn};

use self::attributes::{configure_columns, parse_cell_attributes};
use self::placement::{CellPlacement, TablePlacementCursor};
use super::admission::RichTextParseError;

mod attributes;
mod placement;

const DEFAULT_TABLE_COLUMNS: usize = 1;
const MAX_TABLE_COLUMNS: usize = 64;

#[derive(Clone, Debug)]
struct ActiveCell {
    start: u32,
    placement: CellPlacement,
    box_style: RichTableCellBoxStyle,
}

#[derive(Clone, Debug)]
struct ActiveTable {
    start: u32,
    depth: u16,
    columns: Vec<RichTableColumn>,
    cells: Vec<RichTableCell>,
    active_cell: Option<ActiveCell>,
    placement: TablePlacementCursor,
}

#[derive(Clone, Debug)]
pub(super) struct BbCodeTableState {
    tables: Vec<ActiveTable>,
    cell_budget: usize,
    admitted_cells: usize,
    max_depth: usize,
}

impl BbCodeTableState {
    pub(super) fn new(cell_budget: usize, max_depth: usize) -> Self {
        Self {
            tables: Vec::new(),
            cell_budget,
            admitted_cells: 0,
            max_depth,
        }
    }

    pub(super) fn open_table(
        &mut self,
        value: Option<&str>,
        start: u32,
    ) -> Result<bool, RichTextParseError> {
        let attempted_depth = self.tables.len().saturating_add(1);
        let effective_max_depth = self.max_depth.min(usize::from(u16::MAX).saturating_add(1));
        if attempted_depth > effective_max_depth {
            return Err(RichTextParseError::TableDepthBudgetExceeded {
                attempted_depth,
                max_depth: effective_max_depth,
            });
        }
        let column_count = value
            .and_then(|value| value.split(',').next())
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(DEFAULT_TABLE_COLUMNS)
            .clamp(DEFAULT_TABLE_COLUMNS, MAX_TABLE_COLUMNS);
        self.tables.push(ActiveTable {
            start,
            depth: u16::try_from(self.tables.len())
                .expect("table depth admission preserves the public depth representation"),
            columns: vec![RichTableColumn::default(); column_count],
            cells: Vec::new(),
            active_cell: None,
            placement: TablePlacementCursor::new(column_count),
        });
        Ok(true)
    }

    pub(super) fn open_cell(
        &mut self,
        value: Option<&str>,
        attributes: &[(String, String)],
        start: u32,
    ) -> Result<bool, RichTextParseError> {
        if self.tables.is_empty() {
            return Ok(false);
        }
        let attempted_cells = self.admitted_cells.saturating_add(1);
        if attempted_cells > self.cell_budget {
            return Err(RichTextParseError::TableCellCountBudgetExceeded {
                attempted_cells,
                max_cells: self.cell_budget,
            });
        }
        self.admitted_cells = attempted_cells;
        let table = self.tables.last_mut().expect("table checked above");
        close_active_cell(table, start);
        let cell_attributes = parse_cell_attributes(attributes, table.columns.len());
        let placement = table
            .placement
            .place(cell_attributes.column_span, cell_attributes.row_span);
        configure_columns(&mut table.columns, &placement, value, attributes);
        table.active_cell = Some(ActiveCell {
            start,
            placement,
            box_style: cell_attributes.box_style,
        });
        Ok(true)
    }

    pub(super) fn close_cell(&mut self, end: u32) -> bool {
        let Some(table) = self.tables.last_mut() else {
            return false;
        };
        let had_cell = table.active_cell.is_some();
        close_active_cell(table, end);
        had_cell
    }

    pub(super) fn close_table(&mut self, end: u32) -> Option<RichTable> {
        let mut table = self.tables.pop()?;
        close_active_cell(&mut table, end);
        Some(finish_table(table, end))
    }

    pub(super) fn finish(mut self, end: u32) -> Vec<RichTable> {
        let mut tables = Vec::with_capacity(self.tables.len());
        while let Some(mut table) = self.tables.pop() {
            close_active_cell(&mut table, end);
            tables.push(finish_table(table, end));
        }
        tables
    }
}

fn close_active_cell(table: &mut ActiveTable, end: u32) {
    let Some(cell) = table.active_cell.take() else {
        return;
    };
    table.cells.push(RichTableCell {
        byte_range: (cell.start.min(end), end),
        row_index: cell.placement.row_index,
        column_index: cell.placement.column_index,
        column_span: cell.placement.column_span,
        row_span: cell.placement.row_span,
        box_style: cell.box_style,
    });
}

fn finish_table(table: ActiveTable, end: u32) -> RichTable {
    RichTable {
        byte_range: (table.start.min(end), end),
        depth: table.depth,
        columns: table.columns,
        cells: table.cells,
    }
}
