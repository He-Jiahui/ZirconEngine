use super::*;

const EXPECTED_SLICE_ROW_DATA_GUARD_CHILDREN: &[(&str, &str)] = &[
    (
        "child_sources",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/expected_slice_maps/child_sources.rs",
    ),
    (
        "route_mounts",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/expected_slice_maps/route_mounts.rs",
    ),
    (
        "aggregation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/expected_slice_maps/aggregation.rs",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/expected_slice_maps/status_mirrors.rs",
    ),
    (
        "folder_backed",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/expected_slice_maps/folder_backed.rs",
    ),
];

#[test]
fn runtime_15_status_support_expected_slice_row_data_guard_is_folder_backed() {
    let route = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/expected_slice_maps.rs",
    );

    for (module_name, path) in EXPECTED_SLICE_ROW_DATA_GUARD_CHILDREN {
        let path_mount = format!("#[path = \"expected_slice_maps/{module_name}.rs\"]");
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "expected-slice row-data guard parent mounts child owner",
            &route,
            &[path_mount.as_str(), module_mount.as_str()],
        );

        let child_source = read_runtime_src(path);
        let line_count = child_source.lines().count();
        assert!(
            line_count < 95,
            "{path} should stay below its expected-slice row-data guard child budget; got {line_count} lines"
        );
    }

    for moved_anchor in [
        "Runtime 15 M3 status-support expected-slice row-data owner folder-backed split",
        "status-support expected-slice row-data parent mounts child owner",
        "Runtime 15 M3 aggregation exports expected-slice row-data children",
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    ] {
        assert!(
            !route.contains(moved_anchor),
            "expected_slice_maps.rs should route {moved_anchor} through child owners"
        );
    }
}
