use super::ExpectedStatusOutputSlice;

#[path = "review_guard_splits/code_review_rows.rs"]
mod code_review_rows;
#[path = "review_guard_splits/status_support_rows.rs"]
mod status_support_rows;
#[path = "review_guard_splits/typed_error_rows.rs"]
mod typed_error_rows;

pub(super) const CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    code_review_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = code_review_rows::DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = code_review_rows::PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CODE_REVIEW_STRUCTURE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = code_review_rows::STRUCTURE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CODE_REVIEW_STRUCTURE_GUARD_STATUS_DOCS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    code_review_rows::STRUCTURE_GUARD_STATUS_DOCS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CODE_REVIEW_STRUCTURE_GUARD_FOLDER_BACKED_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    code_review_rows::STRUCTURE_GUARD_FOLDER_BACKED_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CODE_REVIEW_STRUCTURE_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    code_review_rows::STRUCTURE_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CODE_REVIEW_STRUCTURE_GUARD_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    code_review_rows::STRUCTURE_GUARD_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CODE_REVIEW_TYPED_ERROR_STRUCTURE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    code_review_rows::TYPED_ERROR_STRUCTURE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CODE_REVIEW_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = code_review_rows::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    status_support_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    typed_error_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TYPED_ERROR_RUNTIME_SURFACE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = typed_error_rows::RUNTIME_SURFACE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TYPED_ERROR_ASSET_SHADER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = typed_error_rows::ASSET_SHADER_EXPECTED_STATUS_OUTPUT_SLICES;
