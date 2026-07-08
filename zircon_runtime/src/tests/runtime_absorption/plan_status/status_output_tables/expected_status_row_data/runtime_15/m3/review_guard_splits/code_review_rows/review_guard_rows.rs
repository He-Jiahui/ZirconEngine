use super::Slice;

#[path = "review_guard_rows/core_rows.rs"]
mod core_rows;
#[path = "review_guard_rows/f8_rows.rs"]
mod f8_rows;
#[path = "review_guard_rows/late_api_rows.rs"]
mod late_api_rows;
#[path = "review_guard_rows/p0_rows.rs"]
mod p0_rows;
#[path = "review_guard_rows/row_data_owner.rs"]
mod row_data_owner;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = core_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const P0_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    p0_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const F8_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    f8_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const LATE_API_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    late_api_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
