use super::*;

const EXPECTED_SLICE_OWNER_PATH_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "base_and_top_level",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps/base_and_top_level.rs",
        "EXPECTED_SLICE_BASE_AND_TOP_LEVEL_OWNER_PATHS",
    ),
    (
        "route_metadata",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps/route_metadata.rs",
        "EXPECTED_SLICE_ROUTE_METADATA_OWNER_PATHS",
    ),
    (
        "structure_support",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps/structure_support.rs",
        "EXPECTED_SLICE_STRUCTURE_SUPPORT_OWNER_PATHS",
    ),
    (
        "status_support_maps",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps/status_support_maps.rs",
        "EXPECTED_SLICE_STATUS_SUPPORT_MAPS_OWNER_PATHS",
    ),
    (
        "review_guard_structure",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps/review_guard_structure.rs",
        "EXPECTED_SLICE_REVIEW_GUARD_STRUCTURE_OWNER_PATHS",
    ),
    (
        "warning_cleanup",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps/warning_cleanup.rs",
        "EXPECTED_SLICE_WARNING_CLEANUP_OWNER_PATHS",
    ),
];

pub(super) fn assert_expected_slice_owner_path_route_exposes_child_groups() {
    let route = expected_slice_owner_path_route_source();
    assert_contains_all(
        "status-support expected-slice owner path route exposes child groups",
        &route,
        &["STATUS_SUPPORT_EXPECTED_SLICE_MAP_OWNER_PATH_GROUPS"],
    );
    for (module_name, path, representative_const) in EXPECTED_SLICE_OWNER_PATH_CHILDREN {
        let module_mount = format!("#[path = \"expected_slice_maps/{module_name}.rs\"]");
        let module_decl = format!("mod {module_name};");
        assert_contains_all(
            "status-support expected-slice owner path route mounts child",
            &route,
            &[
                module_mount.as_str(),
                module_decl.as_str(),
                *representative_const,
            ],
        );
        let child_source = read_runtime_src(path);
        assert_contains_all(path, &child_source, &[*representative_const]);
        let line_count = child_source.lines().count();
        assert!(
            line_count < 120,
            "{path} should stay below its expected-slice owner-path child budget; got {line_count} lines"
        );
    }
}

pub(super) fn expected_slice_owner_path_route_source() -> String {
    read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps.rs",
    )
}

pub(super) fn expected_slice_owner_path_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in EXPECTED_SLICE_OWNER_PATH_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
