use zircon_runtime_interface::reflect::ReflectFieldId;

use super::DataTableSourceError;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum DataTableCellError {
    #[error(transparent)]
    Source(#[from] DataTableSourceError),
    #[error("source row {0} does not exist")]
    MissingRow(String),
    #[error("source row {0} occurs more than once in the removal selection")]
    DuplicateRowSelection(String),
    #[error("field {0} is not a serializable column in this row type")]
    UnknownField(ReflectFieldId),
    #[error("field {0} is not editable in this editor projection")]
    FieldNotEditable(ReflectFieldId),
    #[error("row {row}, field {field_id} occurs more than once in the edit batch")]
    DuplicateEdit {
        row: String,
        field_id: ReflectFieldId,
    },
    #[error("row {row}, field {field_id} no longer has the expected typed value")]
    ExpectedValueMismatch {
        row: String,
        field_id: ReflectFieldId,
    },
    #[error("row {row}, field {field_id}: {reason}")]
    InvalidValue {
        row: String,
        field_id: ReflectFieldId,
        reason: String,
    },
    #[error("source cell ranges or typed round trip did not preserve the captured source")]
    SourceInvariant,
}
