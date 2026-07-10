type Slice = super::Slice;
#[path = "top_level_support/child_owner_rows.rs"]
mod child_owner_rows;
#[path = "top_level_support/maps_guard_rows.rs"]
mod maps_guard_rows;
#[path = "top_level_support/naming_boundary_rows.rs"]
mod naming_boundary_rows;
#[path = "top_level_support/row_data_owner_rows.rs"]
mod row_data_owner_rows;
#[path = "top_level_support/support_layout_rows.rs"]
mod support_layout_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    support_layout_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    child_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    child_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    child_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    child_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES[3],
    maps_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    maps_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    maps_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    maps_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[3],
    maps_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[4],
    maps_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[5],
    maps_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[6],
    maps_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[7],
    maps_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[8],
    naming_boundary_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    naming_boundary_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    support_layout_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    support_layout_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    support_layout_rows::EXPECTED_STATUS_OUTPUT_SLICES[3],
    row_data_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
];
