use super::ExpectedStatusOutputSlice;

#[path = "status_docs/child_group_moved_row_rows.rs"]
mod child_group_moved_row_rows;
#[path = "status_docs/child_group_status_doc_rows.rs"]
mod child_group_status_doc_rows;
#[path = "status_docs/child_group_status_row_doc_rows.rs"]
mod child_group_status_row_doc_rows;
#[path = "status_docs/foundation_m2_rows.rs"]
mod foundation_m2_rows;

pub(super) const FOUNDATION_M2_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    foundation_m2_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CHILD_GROUP_STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = child_group_status_doc_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CHILD_GROUP_STATUS_ROW_DOC_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = child_group_status_row_doc_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CHILD_GROUP_MOVED_ROW_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = child_group_moved_row_rows::EXPECTED_STATUS_OUTPUT_SLICES;
