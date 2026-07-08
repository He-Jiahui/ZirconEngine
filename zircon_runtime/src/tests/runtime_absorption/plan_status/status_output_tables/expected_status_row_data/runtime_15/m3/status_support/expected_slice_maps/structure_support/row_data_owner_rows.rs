type Slice = super::Slice;

#[path = "row_data_owner_rows/code_review_guard_maps.rs"]
mod code_review_guard_maps;
#[path = "row_data_owner_rows/status_support_row_data.rs"]
mod status_support_row_data;
#[path = "row_data_owner_rows/structure_route_maps.rs"]
mod structure_route_maps;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    code_review_guard_maps::EXPECTED_STATUS_OUTPUT_SLICES[0],
    status_support_row_data::EXPECTED_STATUS_OUTPUT_SLICES[0],
    structure_route_maps::EXPECTED_STATUS_OUTPUT_SLICES[0],
];
