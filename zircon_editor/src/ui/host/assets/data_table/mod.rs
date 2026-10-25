//! DataAsset projection and candidate encoding; schema association and document host stay external.

#[path = "cell_edit.rs"]
mod cell_edit;
#[path = "cell_error.rs"]
mod cell_error;
#[path = "cell_value.rs"]
mod cell_value;

#[path = "codec.rs"]
mod codec;
#[path = "csv/mod.rs"]
mod csv;
#[path = "editing.rs"]
mod editing;
#[path = "error.rs"]
mod error;
#[path = "fields.rs"]
mod fields;
#[path = "raw_source.rs"]
mod raw_source;
#[path = "row_ops/mod.rs"]
mod row_ops;
#[path = "row_rename.rs"]
mod row_rename;
#[path = "source.rs"]
mod source;

pub use cell_edit::DataTableCellEdit;
pub use cell_error::DataTableCellError;
pub use codec::DataTableSourceAdapter;
pub use csv::{DataTableCsvCodec, DataTableCsvError};
pub use editing::DataTableCellCodec;
pub use error::DataTableSourceError;
pub use source::{DataTableRow, DataTableSource};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
