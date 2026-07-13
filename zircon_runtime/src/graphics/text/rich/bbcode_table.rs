use crate::core::framework::render::{
    RichTable, RichTableCell, RichTableCellBoxStyle, RichTableColumn,
};

use self::attributes::{configure_columns, parse_cell_attributes};
use self::placement::{CellPlacement, TablePlacementCursor};

mod attributes;
mod placement;

const DEFAULT_TABLE_COLUMNS: usize = 1;
const MAX_TABLE_COLUMNS: usize = 64;
const MAX_TABLE_NESTING: usize = 8;

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

#[derive(Clone, Debug, Default)]
pub(super) struct BbCodeTableState {
    tables: Vec<ActiveTable>,
    suppressed_depth: usize,
}

impl BbCodeTableState {
    pub(super) fn open_table(&mut self, value: Option<&str>, start: u32) -> bool {
        if self.suppressed_depth > 0 || self.tables.len() >= MAX_TABLE_NESTING {
            self.suppressed_depth = self.suppressed_depth.saturating_add(1);
            return false;
        }
        let column_count = value
            .and_then(|value| value.split(',').next())
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(DEFAULT_TABLE_COLUMNS)
            .clamp(DEFAULT_TABLE_COLUMNS, MAX_TABLE_COLUMNS);
        self.tables.push(ActiveTable {
            start,
            depth: u16::try_from(self.tables.len()).unwrap_or(u16::MAX),
            columns: vec![RichTableColumn::default(); column_count],
            cells: Vec::new(),
            active_cell: None,
            placement: TablePlacementCursor::new(column_count),
        });
        true
    }

    pub(super) fn open_cell(
        &mut self,
        value: Option<&str>,
        attributes: &[(String, String)],
        start: u32,
    ) -> bool {
        if self.suppressed_depth > 0 {
            return false;
        }
        let Some(table) = self.tables.last_mut() else {
            return false;
        };
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
        true
    }

    pub(super) fn close_cell(&mut self, end: u32) -> bool {
        if self.suppressed_depth > 0 {
            return false;
        }
        let Some(table) = self.tables.last_mut() else {
            return false;
        };
        let had_cell = table.active_cell.is_some();
        close_active_cell(table, end);
        had_cell
    }

    pub(super) fn close_table(&mut self, end: u32) -> Option<RichTable> {
        if self.suppressed_depth > 0 {
            self.suppressed_depth -= 1;
            return None;
        }
        let mut table = self.tables.pop()?;
        close_active_cell(&mut table, end);
        Some(finish_table(table, end))
    }

    pub(super) fn finish(mut self, end: u32) -> Vec<RichTable> {
        if self.suppressed_depth > 0 {
            self.suppressed_depth = 0;
        }
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
