pub(super) type Slice = super::ExpectedStatusOutputSlice;

#[path = "status_support_priority_rows/expected_slice_rows.rs"]
mod expected_slice_rows;
#[path = "status_support_priority_rows/priority_plan_docs_rows.rs"]
mod priority_plan_docs_rows;
#[path = "status_support_priority_rows/row_data_guard_rows.rs"]
mod row_data_guard_rows;
#[path = "status_support_priority_rows/row_data_owner.rs"]
mod row_data_owner;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    expected_slice_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_GUARD_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const PRIORITY_PLAN_DOCS_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    priority_plan_docs_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
