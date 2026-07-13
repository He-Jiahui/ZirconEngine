#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CellPlacement {
    pub row_index: u32,
    pub column_index: u16,
    pub column_span: u16,
    pub row_span: u16,
}

/// Resolves sequential cells without allocating a dense row-by-column matrix.
#[derive(Clone, Debug)]
pub(super) struct TablePlacementCursor {
    row: u32,
    column: usize,
    occupied_until_row: Vec<u32>,
}

impl TablePlacementCursor {
    pub(super) fn new(column_count: usize) -> Self {
        Self {
            row: 0,
            column: 0,
            occupied_until_row: vec![0; column_count.max(1)],
        }
    }

    pub(super) fn place(
        &mut self,
        requested_column_span: u16,
        requested_row_span: u16,
    ) -> CellPlacement {
        loop {
            self.advance_past_occupied_columns();
            if self.column >= self.occupied_until_row.len() {
                self.advance_row();
                continue;
            }

            let contiguous_free = self.occupied_until_row[self.column..]
                .iter()
                .take_while(|occupied_until| **occupied_until <= self.row)
                .count()
                .max(1);
            let column_span = usize::from(requested_column_span.max(1))
                .min(contiguous_free)
                .max(1);
            let row_span = requested_row_span.max(1);
            let row_index = self.row;
            let column_index = self.column;
            let occupied_until = self.row.saturating_add(u32::from(row_span));
            for column in
                &mut self.occupied_until_row[column_index..column_index.saturating_add(column_span)]
            {
                *column = (*column).max(occupied_until);
            }

            self.column = self.column.saturating_add(column_span);
            if self.column >= self.occupied_until_row.len() {
                self.advance_row();
            }

            return CellPlacement {
                row_index,
                column_index: u16::try_from(column_index).unwrap_or(u16::MAX),
                column_span: u16::try_from(column_span).unwrap_or(u16::MAX),
                row_span,
            };
        }
    }

    fn advance_past_occupied_columns(&mut self) {
        while self.column < self.occupied_until_row.len()
            && self.occupied_until_row[self.column] > self.row
        {
            self.column += 1;
        }
    }

    fn advance_row(&mut self) {
        self.row = self.row.saturating_add(1);
        self.column = 0;
    }
}
