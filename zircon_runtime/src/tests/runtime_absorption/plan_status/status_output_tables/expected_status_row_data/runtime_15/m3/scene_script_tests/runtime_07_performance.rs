pub(super) type Slice = super::ExpectedStatusOutputSlice;

#[path = "runtime_07_performance/owner_budget_rows.rs"]
mod owner_budget_rows;
#[path = "runtime_07_performance/primary_guard_rows.rs"]
mod primary_guard_rows;
#[path = "runtime_07_performance/row_data_owner.rs"]
mod row_data_owner;
#[path = "runtime_07_performance/split_layout_rows.rs"]
mod split_layout_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    primary_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const SPLIT_LAYOUT_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    split_layout_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const OWNER_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    owner_budget_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
