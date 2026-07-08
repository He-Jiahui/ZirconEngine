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

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(date) = root_runtime_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = module_layout_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = review_guard_row_data_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = foundation_row_data_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = child_group_row_data_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = other_row_data_maps::expected_date_for_slice(slice) {
        return Some(date);
    }

    None
}
