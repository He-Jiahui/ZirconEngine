pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 naming-boundary expected-slice parent maps folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 naming-boundary render-graphics expected-slice map rows folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 naming-boundary expected-slice guard route metadata child split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 naming-boundary expected-slice route metadata folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 naming-boundary expected-slice guard body folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 naming-boundary expected-slice guard body route metadata folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 naming-boundary route-owner split" => Some("2026-07-05"),
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
