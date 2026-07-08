type Slice = super::Slice;

#[path = "status_support_maps/child_group_row_data_map_rows.rs"]
mod child_group_row_data_map_rows;
#[path = "status_support_maps/expected_slice_map_rows.rs"]
mod expected_slice_map_rows;
#[path = "status_support_maps/guard_body_rows.rs"]
mod guard_body_rows;
#[path = "status_support_maps/priority_plan_doc_map_rows.rs"]
mod priority_plan_doc_map_rows;
#[path = "status_support_maps/route_guard_rows.rs"]
mod route_guard_rows;
#[path = "status_support_maps/route_metadata_rows.rs"]
mod route_metadata_rows;
#[path = "status_support_maps/row_data_owner_rows.rs"]
mod row_data_owner_rows;
#[path = "status_support_maps/runtime_index_anchor_map_rows.rs"]
mod runtime_index_anchor_map_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    route_metadata_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    route_metadata_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    guard_body_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    guard_body_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    route_guard_rows::RUNTIME_INDEX_ANCHOR_EXPECTED_STATUS_OUTPUT_SLICES[0],
    route_guard_rows::EXPECTED_SLICE_ROUTE_EXPECTED_STATUS_OUTPUT_SLICES[0],
    route_guard_rows::EXPECTED_SLICE_ROUTE_EXPECTED_STATUS_OUTPUT_SLICES[1],
    route_guard_rows::EXPECTED_SLICE_ROUTE_EXPECTED_STATUS_OUTPUT_SLICES[2],
    route_guard_rows::EXPECTED_SLICE_ROUTE_EXPECTED_STATUS_OUTPUT_SLICES[3],
    route_guard_rows::ROUTE_INPUT_EXPECTED_STATUS_OUTPUT_SLICES[0],
    route_guard_rows::ROUTE_INPUT_EXPECTED_STATUS_OUTPUT_SLICES[1],
    route_guard_rows::ROUTE_INPUT_EXPECTED_STATUS_OUTPUT_SLICES[2],
    route_guard_rows::ROUTE_INPUT_EXPECTED_STATUS_OUTPUT_SLICES[3],
    route_guard_rows::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES[0],
    expected_slice_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    expected_slice_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    expected_slice_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    expected_slice_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[3],
    expected_slice_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[4],
    expected_slice_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[5],
    expected_slice_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[6],
    expected_slice_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[7],
    runtime_index_anchor_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    runtime_index_anchor_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    priority_plan_doc_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    priority_plan_doc_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    child_group_row_data_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    child_group_row_data_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    row_data_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    route_guard_rows::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES[0],
];
