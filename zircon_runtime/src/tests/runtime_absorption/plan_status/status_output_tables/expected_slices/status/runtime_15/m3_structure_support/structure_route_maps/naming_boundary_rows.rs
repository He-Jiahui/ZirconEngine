pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 naming-boundary expected-slice parent maps folder-backed split" => Some(
            "runtime_15_naming_boundary_expected_slice_parent_maps_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 naming-boundary render-graphics expected-slice map rows folder-backed split" => Some(
            "runtime_15_naming_boundary_render_graphics_expected_slice_map_rows_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 naming-boundary expected-slice guard route metadata child split" => Some(
            "runtime_15_naming_boundary_expected_slice_guard_route_metadata_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 naming-boundary expected-slice route metadata folder-backed split" => Some(
            "runtime_15_naming_boundary_expected_slice_route_metadata_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 naming-boundary expected-slice guard body folder-backed split" => Some(
            "runtime_15_naming_boundary_expected_slice_guard_body_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 naming-boundary expected-slice guard body route metadata folder-backed split" => Some(
            "runtime_15_naming_boundary_expected_slice_guard_body_route_metadata_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 naming-boundary route-owner split" => {
            Some("runtime_15_naming_boundary_route_owner_split_static_passed_cargo_deferred")
        }
        _ => None,
    }
}

// Guard: runtime_15_status_output_naming_boundary_expected_slice_maps_are_folder_backed.
// Guard: runtime_15_status_output_naming_boundary_render_graphics_map_rows_are_folder_backed.
// Guard: runtime_15_status_output_naming_boundary_expected_slice_route_metadata_is_child_owned.
// Guard: runtime_15_status_output_naming_boundary_expected_slice_route_metadata_is_folder_backed.
// Guard: runtime_15_status_output_naming_boundary_expected_slice_guard_body_is_child_owned.
// Guard: runtime_15_status_output_naming_boundary_expected_slice_guard_body_route_metadata_is_folder_backed.
// Guard: runtime_15_naming_boundary_route_owner_is_folder_backed.
