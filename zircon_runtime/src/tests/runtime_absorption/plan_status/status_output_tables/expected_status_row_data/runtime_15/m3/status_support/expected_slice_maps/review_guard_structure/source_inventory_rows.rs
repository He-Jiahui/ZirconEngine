type Slice = super::Slice;

#[path = "source_inventory_rows/root_source_rows.rs"]
mod root_source_rows;
#[path = "source_inventory_rows/route_source_rows.rs"]
mod route_source_rows;
#[path = "source_inventory_rows/status_map_rows.rs"]
mod status_map_rows;
#[path = "source_inventory_rows/structure_path_rows.rs"]
mod structure_path_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    root_source_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    root_source_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    status_map_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    route_source_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    route_source_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    route_source_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    structure_path_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
];
