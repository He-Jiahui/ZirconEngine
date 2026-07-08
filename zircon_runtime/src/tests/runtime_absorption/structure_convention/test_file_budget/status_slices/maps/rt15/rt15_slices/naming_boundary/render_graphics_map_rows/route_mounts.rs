use super::*;

#[test]
fn runtime_15_naming_boundary_render_graphics_map_rows_route_mounts_are_child_owned() {
    let naming_boundary_parent = read_runtime_src(NAMING_BOUNDARY_ROUTE_PARENT);
    let guard_parent = read_guard_parent();
    let status_parent = read_runtime_src(STATUS_CHILD_PATHS[3]);
    let date_parent = read_runtime_src(DATE_CHILD_PATHS[3]);

    assert_contains_all(
        "naming-boundary expected-slice maps parent mounts render-graphics map row guard",
        &naming_boundary_parent,
        &[
            "#[path = \"naming_boundary/render_graphics_map_rows.rs\"]",
            "mod render_graphics_map_rows;",
        ],
    );
    assert_contains_all(
        "render-graphics map row guard parent mounts focused children",
        &guard_parent,
        &[
            "#[path = \"render_graphics_map_rows/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"render_graphics_map_rows/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"render_graphics_map_rows/paths.rs\"]",
            "mod paths;",
            "#[path = \"render_graphics_map_rows/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"render_graphics_map_rows/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"render_graphics_map_rows/status_rows.rs\"]",
            "mod status_rows;",
            "use paths::*;",
        ],
    );
    for (label, parent) in [
        ("status render-graphics map parent", status_parent.as_str()),
        ("date render-graphics map parent", date_parent.as_str()),
    ] {
        assert_contains_all(
            label,
            parent,
            &[
                "#[path = \"render_graphics/asset_font_rows.rs\"]",
                "mod asset_font_rows;",
                "#[path = \"render_graphics/expected_slice_rows.rs\"]",
                "mod expected_slice_rows;",
                "#[path = \"render_graphics/fixture_fallback_rows.rs\"]",
                "mod fixture_fallback_rows;",
                "#[path = \"render_graphics/plugin_texture_rows.rs\"]",
                "mod plugin_texture_rows;",
                "#[path = \"render_graphics/render_framework_rows.rs\"]",
                "mod render_framework_rows;",
                "#[path = \"render_graphics/scene_render_rows.rs\"]",
                "mod scene_render_rows;",
                "#[path = \"render_graphics/shader_model_rows.rs\"]",
                "mod shader_model_rows;",
            ],
        );
    }
}
