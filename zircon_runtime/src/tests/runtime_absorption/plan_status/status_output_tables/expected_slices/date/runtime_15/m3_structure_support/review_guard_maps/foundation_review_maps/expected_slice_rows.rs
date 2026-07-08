#[path = "expected_slice_rows/expected_slice_map_rows.rs"]
mod expected_slice_map_rows;
#[path = "expected_slice_rows/foundation_status_rows.rs"]
mod foundation_status_rows;
#[path = "expected_slice_rows/root_route_rows.rs"]
mod root_route_rows;
#[path = "expected_slice_rows/route_children_rows.rs"]
mod route_children_rows;
#[path = "expected_slice_rows/route_metadata_rows.rs"]
mod route_metadata_rows;
#[path = "expected_slice_rows/source_metadata_rows.rs"]
mod source_metadata_rows;
#[path = "expected_slice_rows/status_map_rows.rs"]
mod status_map_rows;

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    route_metadata_rows::expected_date_for_slice(slice)
        .or_else(|| root_route_rows::expected_date_for_slice(slice))
        .or_else(|| foundation_status_rows::expected_date_for_slice(slice))
        .or_else(|| route_children_rows::expected_date_for_slice(slice))
        .or_else(|| source_metadata_rows::expected_date_for_slice(slice))
        .or_else(|| status_map_rows::expected_date_for_slice(slice))
        .or_else(|| expected_slice_map_rows::expected_date_for_slice(slice))
}
