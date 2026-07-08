use super::ExpectedStatusOutputSlice;

#[path = "review_guard/base_rows.rs"]
mod base_rows;
#[path = "review_guard/code_review_rows.rs"]
mod code_review_rows;
#[path = "review_guard/direct_assertion_rows.rs"]
mod direct_assertion_rows;
#[path = "review_guard/moved_row_rows.rs"]
mod moved_row_rows;
#[path = "review_guard/row_data_rows.rs"]
mod row_data_rows;
#[path = "review_guard/status_doc_rows.rs"]
mod status_doc_rows;

const ROW_DATA_STATUS_MIRROR_STATUS_ANCHORS: &[&str] = &[
    "Runtime 15 M3 review-guard row-data guard folder-backed split",
    "runtime_15_review_guard_row_data_guard_folder_backed_static_passed_cargo_deferred",
    "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data.rs",
    "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/delegation.rs",
    "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/moved_rows.rs",
    "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/aggregation.rs",
    "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/status_mirrors.rs",
    "runtime_15_review_guard_row_data_guard_is_folder_backed",
    "runtime_15_status_output_m3_review_guard_row_data_is_child_owner",
    "Runtime 15 M3 review-guard row-data status-mirror child split",
    "runtime_15_review_guard_row_data_status_mirror_child_split_static_passed_cargo_deferred",
    "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/status_mirrors/child_split_status.rs",
    "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/status_mirrors/folder_backed_status.rs",
    "runtime_15_review_guard_row_data_status_mirror_children_are_child_owned",
    "Cargo gate deferred",
];

pub(super) const BASE_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    base_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const MOVED_ROW_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    moved_row_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    code_review_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    row_data_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    status_doc_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    direct_assertion_rows::EXPECTED_STATUS_OUTPUT_SLICES;
