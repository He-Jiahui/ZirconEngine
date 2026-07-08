pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 naming-boundary render-graphics expected-slice map rows folder-backed split" => {
            Some("runtime_15_naming_boundary_render_graphics_expected_slice_map_rows_folder_backed_static_passed_cargo_deferred")
        }
        _ => None,
    }
}
