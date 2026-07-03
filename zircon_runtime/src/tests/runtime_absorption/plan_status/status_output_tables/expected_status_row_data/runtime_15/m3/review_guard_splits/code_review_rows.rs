type Slice = super::ExpectedStatusOutputSlice;

#[path = "code_review_rows/direct_assertion_rows.rs"]
mod direct_assertion_rows;
#[path = "code_review_rows/plugin_importer_rows.rs"]
mod plugin_importer_rows;
#[path = "code_review_rows/review_guard_rows.rs"]
mod review_guard_rows;
#[path = "code_review_rows/row_data_owner.rs"]
mod row_data_owner;
#[path = "code_review_rows/structure_guard_rows.rs"]
mod structure_guard_rows;
#[path = "code_review_rows/typed_error_structure_rows.rs"]
mod typed_error_structure_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    review_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    direct_assertion_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    plugin_importer_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STRUCTURE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    structure_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STRUCTURE_GUARD_STATUS_DOCS_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    structure_guard_rows::STATUS_DOCS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STRUCTURE_GUARD_FOLDER_BACKED_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    structure_guard_rows::FOLDER_BACKED_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STRUCTURE_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    structure_guard_rows::TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STRUCTURE_GUARD_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    structure_guard_rows::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TYPED_ERROR_STRUCTURE_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    typed_error_structure_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
