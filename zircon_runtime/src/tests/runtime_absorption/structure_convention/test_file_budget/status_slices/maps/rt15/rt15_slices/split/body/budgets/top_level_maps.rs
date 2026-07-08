use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_top_level_maps_stay_budgeted()
{
    for (path, source, max_lines) in [
        (PARENT_PATH, read_runtime_15_map_parent(), 20usize),
        (CHILD_OWNER_PATH, read_runtime_15_map("child_owners.rs"), 20),
        (
            NAMING_BOUNDARY_PATH,
            read_runtime_15_map("naming_boundary.rs"),
            25,
        ),
        (
            NAMING_BOUNDARY_SOURCES_PATH,
            read_runtime_15_map("naming_boundary/sources.rs"),
            115,
        ),
        (
            NAMING_BOUNDARY_GUARD_BODY_PATH,
            read_runtime_15_map("naming_boundary/guard_body.rs"),
            20,
        ),
        (
            NAMING_BOUNDARY_ROUTE_METADATA_PATH,
            read_runtime_15_map("naming_boundary/route_metadata.rs"),
            150,
        ),
        (
            SPLIT_LAYOUT_PATH,
            read_runtime_15_map("split_layout.rs"),
            20,
        ),
        (
            SPLIT_LAYOUT_SOURCES_PATH,
            read_runtime_15_map("split/sources.rs"),
            80,
        ),
        (
            SPLIT_LAYOUT_GUARD_BODY_PATH,
            read_runtime_15_map("split/guard_body.rs"),
            25,
        ),
        (
            SPLIT_LAYOUT_STATUS_MIRRORS_PATH,
            read_runtime_15_map("split/status_mirrors.rs"),
            100,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }
}
