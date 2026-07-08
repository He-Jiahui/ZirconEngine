type Slice = super::Slice;

#[path = "typed_error_rows/guard_body_rows.rs"]
mod guard_body_rows;
#[path = "typed_error_rows/map_row_guard_rows.rs"]
mod map_row_guard_rows;
#[path = "typed_error_rows/route_metadata_rows.rs"]
mod route_metadata_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    route_metadata_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    route_metadata_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    guard_body_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    guard_body_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    guard_body_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    map_row_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    map_row_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
];
