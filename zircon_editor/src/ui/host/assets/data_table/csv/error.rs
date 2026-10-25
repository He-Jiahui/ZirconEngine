use super::super::{DataTableCellError, DataTableSourceError};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum DataTableCsvError {
    #[error(transparent)]
    Source(#[from] DataTableSourceError),
    #[error(transparent)]
    Cell(#[from] DataTableCellError),
    #[error("CSV has no header record")]
    MissingHeader,
    #[error("invalid CSV at UTF-8 byte {byte}: {reason}")]
    Syntax { byte: usize, reason: &'static str },
    #[error("CSV header: {0}")]
    Header(String),
    #[error("CSV data record {row} has {found} cells, expected {expected}")]
    RowWidth {
        row: usize,
        expected: usize,
        found: usize,
    },
}
