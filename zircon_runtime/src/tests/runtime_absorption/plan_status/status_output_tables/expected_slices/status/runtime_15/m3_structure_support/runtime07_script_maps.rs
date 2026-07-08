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

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(status) = runtime07_guard_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = runtime07_split_layout_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = runtime07_owner_budget_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = script_vm_runtime_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = expected_slice_map_rows::expected_status_for_slice(slice) {
        return Some(status);
    }

    None
}
