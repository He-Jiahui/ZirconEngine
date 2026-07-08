type Slice = super::Slice;

#[path = "route_guard_rows/expected_slice_route_rows.rs"]
mod expected_slice_route_rows;
#[path = "route_guard_rows/route_input_rows.rs"]
mod route_input_rows;
#[path = "route_guard_rows/row_data_owner.rs"]
mod row_data_owner;
#[path = "route_guard_rows/runtime_index_anchor_rows.rs"]
mod runtime_index_anchor_rows;

pub(super) const RUNTIME_INDEX_ANCHOR_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    runtime_index_anchor_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const EXPECTED_SLICE_ROUTE_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    expected_slice_route_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROUTE_INPUT_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    route_input_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    RUNTIME_INDEX_ANCHOR_EXPECTED_STATUS_OUTPUT_SLICES[0],
    EXPECTED_SLICE_ROUTE_EXPECTED_STATUS_OUTPUT_SLICES[0],
    EXPECTED_SLICE_ROUTE_EXPECTED_STATUS_OUTPUT_SLICES[1],
    EXPECTED_SLICE_ROUTE_EXPECTED_STATUS_OUTPUT_SLICES[2],
    EXPECTED_SLICE_ROUTE_EXPECTED_STATUS_OUTPUT_SLICES[3],
    ROUTE_INPUT_EXPECTED_STATUS_OUTPUT_SLICES[0],
    ROUTE_INPUT_EXPECTED_STATUS_OUTPUT_SLICES[1],
    ROUTE_INPUT_EXPECTED_STATUS_OUTPUT_SLICES[2],
    ROUTE_INPUT_EXPECTED_STATUS_OUTPUT_SLICES[3],
    ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES[0],
];
