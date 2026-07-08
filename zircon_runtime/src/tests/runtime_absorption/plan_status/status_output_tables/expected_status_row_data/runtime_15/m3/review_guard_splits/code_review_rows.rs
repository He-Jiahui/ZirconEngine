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
pub(super) const REVIEW_GUARD_P0_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    review_guard_rows::P0_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const REVIEW_GUARD_F8_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    review_guard_rows::F8_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const REVIEW_GUARD_LATE_API_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    review_guard_rows::LATE_API_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const REVIEW_GUARD_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    review_guard_rows::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    direct_assertion_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const DIRECT_ASSERTION_F12_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    direct_assertion_rows::F12_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const DIRECT_ASSERTION_ROOT_PARENT_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    direct_assertion_rows::ROOT_PARENT_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const DIRECT_ASSERTION_RENDER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    direct_assertion_rows::RENDER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const DIRECT_ASSERTION_F8_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    direct_assertion_rows::F8_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const DIRECT_ASSERTION_P0_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    direct_assertion_rows::P0_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const DIRECT_ASSERTION_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    direct_assertion_rows::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
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
pub(super) const TYPED_ERROR_STRUCTURE_STATUS_DOC_PATHS_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    typed_error_structure_rows::STATUS_DOC_PATHS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TYPED_ERROR_STRUCTURE_STATUS_DOC_DELEGATION_EXPECTED_STATUS_OUTPUT_SLICES:
    &[Slice] = typed_error_structure_rows::STATUS_DOC_DELEGATION_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TYPED_ERROR_STRUCTURE_STATUS_DOC_STATUS_MAPS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[Slice] = typed_error_structure_rows::STATUS_DOC_STATUS_MAPS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TYPED_ERROR_STRUCTURE_STATUS_DOC_STATUS_MIRRORS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[Slice] = typed_error_structure_rows::STATUS_DOC_STATUS_MIRRORS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TYPED_ERROR_STRUCTURE_STRUCTURE_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES:
    &[Slice] = typed_error_structure_rows::STRUCTURE_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TYPED_ERROR_STRUCTURE_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    typed_error_structure_rows::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
