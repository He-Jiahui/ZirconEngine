type Slice = super::Slice;

#[path = "structure_support/foundation_rows.rs"]
mod foundation_rows;
#[path = "structure_support/guard_rows.rs"]
mod guard_rows;
#[path = "structure_support/map_rows.rs"]
mod map_rows;
#[path = "structure_support/parent_route_rows.rs"]
mod parent_route_rows;
#[path = "structure_support/review_route_rows.rs"]
mod review_route_rows;
#[path = "structure_support/row_data_owner_rows.rs"]
mod row_data_owner_rows;
#[path = "structure_support/typed_error_rows.rs"]
mod typed_error_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    parent_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    parent_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[3],
    guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[4],
    map_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    parent_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    parent_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[3],
    parent_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[4],
    guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[5],
    map_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    map_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    map_rows::EXPECTED_STATUS_OUTPUT_SLICES[3],
    map_rows::EXPECTED_STATUS_OUTPUT_SLICES[4],
    map_rows::EXPECTED_STATUS_OUTPUT_SLICES[5],
    foundation_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    foundation_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    foundation_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    foundation_rows::EXPECTED_STATUS_OUTPUT_SLICES[3],
    foundation_rows::EXPECTED_STATUS_OUTPUT_SLICES[4],
    foundation_rows::EXPECTED_STATUS_OUTPUT_SLICES[5],
    review_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    review_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    review_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    typed_error_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    row_data_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    row_data_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    row_data_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
];
