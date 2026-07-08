#[path = "row_data_maps/child_group_row_data_maps.rs"]
mod child_group_row_data_maps;
#[path = "row_data_maps/foundation_row_data_maps.rs"]
mod foundation_row_data_maps;
#[path = "row_data_maps/module_layout_maps.rs"]
mod module_layout_maps;
#[path = "row_data_maps/other_row_data_maps.rs"]
mod other_row_data_maps;
#[path = "row_data_maps/review_guard_row_data_maps.rs"]
mod review_guard_row_data_maps;
#[path = "row_data_maps/root_runtime_maps.rs"]
mod root_runtime_maps;

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(status) = root_runtime_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = module_layout_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = review_guard_row_data_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = foundation_row_data_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = child_group_row_data_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = other_row_data_maps::expected_status_for_slice(slice) {
        return Some(status);
    }

    None
}
