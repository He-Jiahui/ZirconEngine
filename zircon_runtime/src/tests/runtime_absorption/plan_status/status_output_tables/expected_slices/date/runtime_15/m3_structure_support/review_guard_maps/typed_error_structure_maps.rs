#[path = "typed_error_structure_maps/expected_slice_map_rows.rs"]
mod expected_slice_map_rows;
#[path = "typed_error_structure_maps/moved_guard_absence_maps.rs"]
mod moved_guard_absence_maps;
#[path = "typed_error_structure_maps/native_plugin_loader_maps.rs"]
mod native_plugin_loader_maps;
#[path = "typed_error_structure_maps/structure_assertion_maps.rs"]
mod structure_assertion_maps;
#[path = "typed_error_structure_maps/structure_guard_maps.rs"]
mod structure_guard_maps;
#[path = "typed_error_structure_maps/top_level_maps.rs"]
mod top_level_maps;

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    top_level_maps::expected_date_for_slice(slice)
        .or_else(|| structure_guard_maps::expected_date_for_slice(slice))
        .or_else(|| structure_assertion_maps::expected_date_for_slice(slice))
        .or_else(|| native_plugin_loader_maps::expected_date_for_slice(slice))
        .or_else(|| moved_guard_absence_maps::expected_date_for_slice(slice))
        .or_else(|| expected_slice_map_rows::expected_date_for_slice(slice))
}
