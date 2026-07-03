use super::Slice;

#[path = "structure_guard_rows/folder_backed_summary.rs"]
mod folder_backed_summary;
#[path = "structure_guard_rows/root_and_children.rs"]
mod root_and_children;
#[path = "structure_guard_rows/row_data_owner.rs"]
mod row_data_owner;
#[path = "structure_guard_rows/status_docs.rs"]
mod status_docs;
#[path = "structure_guard_rows/typed_error.rs"]
mod typed_error;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    root_and_children::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_DOCS_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    status_docs::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const FOLDER_BACKED_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    folder_backed_summary::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    typed_error::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
