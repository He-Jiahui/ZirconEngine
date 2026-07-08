use super::ExpectedStatusOutputSlice;

#[path = "module_layout/base_rows.rs"]
mod base_rows;
#[path = "module_layout/child_summary_rows.rs"]
mod child_summary_rows;
#[path = "module_layout/child_summary_status_doc_rows.rs"]
mod child_summary_status_doc_rows;
#[path = "module_layout/status_doc_rows.rs"]
mod status_doc_rows;

pub(super) const BASE_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    base_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    status_doc_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CHILD_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    child_summary_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CHILD_SUMMARY_STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = child_summary_status_doc_rows::EXPECTED_STATUS_OUTPUT_SLICES;
