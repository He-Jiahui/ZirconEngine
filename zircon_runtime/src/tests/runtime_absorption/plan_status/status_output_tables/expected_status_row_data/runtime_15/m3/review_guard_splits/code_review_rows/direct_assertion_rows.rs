type Slice = super::Slice;

#[path = "direct_assertion_rows/core_rows.rs"]
mod core_rows;
#[path = "direct_assertion_rows/f12_rows.rs"]
mod f12_rows;
#[path = "direct_assertion_rows/f8_rows.rs"]
mod f8_rows;
#[path = "direct_assertion_rows/p0_rows.rs"]
mod p0_rows;
#[path = "direct_assertion_rows/render_rows.rs"]
mod render_rows;
#[path = "direct_assertion_rows/root_parent_rows.rs"]
mod root_parent_rows;
#[path = "direct_assertion_rows/row_data_owner_rows.rs"]
mod row_data_owner_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = core_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const F12_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    f12_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROOT_PARENT_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    root_parent_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RENDER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    render_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const F8_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    f8_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const P0_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    p0_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES;
