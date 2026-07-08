use super::*;

#[test]
fn runtime_15_status_output_naming_boundary_render_graphics_map_rows_guard_is_folder_backed() {
    let parent = read_guard_parent();
    let children = read_guard_children();

    for moved_anchor in [
        "#[test]",
        "let status_parent = read_runtime_src(STATUS_CHILD_PATHS[3])",
        "Runtime 15 M2 graphics render-framework receiver naming hard cutover",
        MAP_ROWS_SLICE,
        MAP_ROWS_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "naming_boundary/render_graphics_map_rows.rs should delegate moved guard anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "render-graphics map row guard children keep moved checks",
        &children,
        &[
            "runtime_15_naming_boundary_render_graphics_map_rows_route_mounts_are_child_owned",
            "runtime_15_status_output_naming_boundary_render_graphics_map_rows_are_folder_backed",
            "runtime_15_naming_boundary_render_graphics_map_rows_status_rows_are_synced",
            "runtime_15_naming_boundary_render_graphics_map_rows_guard_status_mirrors_are_synced",
            "runtime_15_naming_boundary_render_graphics_map_rows_docs_are_synced",
            "runtime_15_naming_boundary_render_graphics_map_rows_guard_children_stay_budgeted",
            MAP_ROWS_GUARD_GUARD,
        ],
    );
}
