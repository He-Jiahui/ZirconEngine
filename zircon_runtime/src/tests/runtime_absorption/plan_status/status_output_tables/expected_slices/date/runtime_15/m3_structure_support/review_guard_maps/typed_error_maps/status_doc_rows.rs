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

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(date) = base_status_doc_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = paths_inventory_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = delegation_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = status_maps_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = status_mirrors_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = expected_slice_map_rows::expected_date_for_slice(slice) {
        return Some(date);
    }

    None
}
