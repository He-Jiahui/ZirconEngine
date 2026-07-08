type Slice = super::Slice;

#[path = "review_guard_structure/guard_body_rows.rs"]
mod guard_body_rows;
#[path = "review_guard_structure/root_route_rows.rs"]
mod root_route_rows;
#[path = "review_guard_structure/row_data_owner_rows.rs"]
mod row_data_owner_rows;
#[path = "review_guard_structure/source_inventory_rows.rs"]
mod source_inventory_rows;
#[path = "review_guard_structure/structure_guard_rows.rs"]
mod structure_guard_rows;
#[path = "review_guard_structure/typed_error_rows.rs"]
mod typed_error_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    structure_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    typed_error_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    typed_error_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    typed_error_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    typed_error_rows::EXPECTED_STATUS_OUTPUT_SLICES[3],
    typed_error_rows::EXPECTED_STATUS_OUTPUT_SLICES[4],
    typed_error_rows::EXPECTED_STATUS_OUTPUT_SLICES[5],
    root_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    root_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    root_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[8],
    root_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    root_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[7],
    guard_body_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    guard_body_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    guard_body_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    root_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[3],
    root_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[4],
    root_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[5],
    root_route_rows::EXPECTED_STATUS_OUTPUT_SLICES[6],
    source_inventory_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    source_inventory_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    source_inventory_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    source_inventory_rows::EXPECTED_STATUS_OUTPUT_SLICES[3],
    source_inventory_rows::EXPECTED_STATUS_OUTPUT_SLICES[4],
    row_data_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
];
