use super::Slice;

#[path = "review_guard_rows/core_rows.rs"]
mod core_rows;
#[path = "review_guard_rows/row_data_guard_rows.rs"]
mod row_data_guard_rows;
#[path = "review_guard_rows/row_data_owner.rs"]
mod row_data_owner;
#[path = "review_guard_rows/status_support_guard_rows.rs"]
mod status_support_guard_rows;
#[path = "review_guard_rows/typed_error_guard_rows.rs"]
mod typed_error_guard_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = core_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_SUPPORT_GUARD_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    status_support_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TYPED_ERROR_GUARD_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    typed_error_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_GUARD_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
