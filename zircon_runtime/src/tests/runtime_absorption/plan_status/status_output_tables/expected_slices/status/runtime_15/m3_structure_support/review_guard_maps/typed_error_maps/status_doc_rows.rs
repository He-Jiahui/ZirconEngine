#[path = "status_doc_rows/base_status_doc_rows.rs"]
mod base_status_doc_rows;
#[path = "status_doc_rows/delegation_rows.rs"]
mod delegation_rows;
#[path = "status_doc_rows/expected_slice_map_rows.rs"]
mod expected_slice_map_rows;
#[path = "status_doc_rows/paths_inventory_rows.rs"]
mod paths_inventory_rows;
#[path = "status_doc_rows/status_maps_rows.rs"]
mod status_maps_rows;
#[path = "status_doc_rows/status_mirrors_rows.rs"]
mod status_mirrors_rows;

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(status) = base_status_doc_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = paths_inventory_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = delegation_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = status_maps_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = status_mirrors_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = expected_slice_map_rows::expected_status_for_slice(slice) {
        return Some(status);
    }

    None
}
