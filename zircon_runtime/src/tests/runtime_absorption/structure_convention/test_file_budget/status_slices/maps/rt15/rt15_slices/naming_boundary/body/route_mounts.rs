use super::*;

#[test]
fn runtime_15_status_output_naming_boundary_expected_slice_maps_are_folder_backed() {
    for (label, parent, function_call) in [
        (
            "status naming-boundary parent",
            read_runtime_src(STATUS_PARENT_PATH),
            "expected_status_for_slice(slice)",
        ),
        (
            "date naming-boundary parent",
            read_runtime_src(DATE_PARENT_PATH),
            "expected_date_for_slice(slice)",
        ),
    ] {
        assert_contains_all(
            label,
            &parent,
            &[
                "mod core_bootstrap;",
                "mod scene_asset_runtime;",
                "mod plugin_ui_platform;",
                "mod render_graphics;",
                function_call,
            ],
        );
    }
}
