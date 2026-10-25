use zircon_runtime::asset::DataAssetFormat;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum DataTableSourceError {
    #[error("data table source format {0:?} is unsupported")]
    UnsupportedFormat(DataAssetFormat),
    #[error("an explicitly associated row type is required")]
    MissingRowType,
    #[error("invalid row type: {0}")]
    InvalidRowType(String),
    #[error("column {field} has unsupported type {value_type_path}")]
    UnsupportedColumn {
        field: String,
        value_type_path: String,
    },
    #[error("invalid data table JSON: {0}")]
    Encoding(String),
    #[error("source text and its populated canonical JSON cache disagree")]
    CanonicalMismatch,
    #[error("source row {source_row} requires a nonempty string key in {key_field}")]
    InvalidRowKey {
        source_row: usize,
        key_field: String,
    },
    #[error("duplicate source row key {0}")]
    DuplicateRowKey(String),
    #[error("row {row} contains unknown column {field}")]
    UnknownColumn { row: String, field: String },
    #[error("row {row} is missing column {field}")]
    MissingColumn { row: String, field: String },
    #[error("row {row}, column {field} ({value_type_path}): {reason}")]
    InvalidCell {
        row: String,
        field: String,
        value_type_path: String,
        reason: String,
    },
}
