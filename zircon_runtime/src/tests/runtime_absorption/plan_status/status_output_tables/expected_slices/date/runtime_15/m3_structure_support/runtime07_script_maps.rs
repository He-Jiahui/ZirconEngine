#[path = "runtime07_script_maps/expected_slice_map_rows.rs"]
mod expected_slice_map_rows;
#[path = "runtime07_script_maps/runtime07_guard_maps.rs"]
mod runtime07_guard_maps;
#[path = "runtime07_script_maps/runtime07_owner_budget_maps.rs"]
mod runtime07_owner_budget_maps;
#[path = "runtime07_script_maps/runtime07_split_layout_maps.rs"]
mod runtime07_split_layout_maps;
#[path = "runtime07_script_maps/script_vm_runtime_maps.rs"]
mod script_vm_runtime_maps;

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(date) = runtime07_guard_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = runtime07_split_layout_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = runtime07_owner_budget_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = script_vm_runtime_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = expected_slice_map_rows::expected_date_for_slice(slice) {
        return Some(date);
    }

    None
}
