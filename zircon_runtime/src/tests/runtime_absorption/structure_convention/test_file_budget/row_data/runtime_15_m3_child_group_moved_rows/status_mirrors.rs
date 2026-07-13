use super::*;

#[path = "mirrors/child_split_status.rs"]
mod child_split_status;
#[path = "mirrors/folder_backed_status.rs"]
mod folder_backed_status;
#[path = "mirrors/historical_status.rs"]
mod historical_status;

const STATUS_MIRRORS_ROUTE_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/status_mirrors.rs";
const CHILD_SPLIT_STATUS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/mirrors/child_split_status.rs";
const HISTORICAL_STATUS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/mirrors/historical_status.rs";
const FOLDER_BACKED_STATUS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/mirrors/folder_backed_status.rs";

const STATUS_MIRROR_CHILD_SPLIT_NAME: &str =
    "Runtime 15 M3 child-group moved-row status-mirror child split";
const STATUS_MIRROR_CHILD_SPLIT_ID: &str =
    "runtime_15_m3_child_group_moved_row_status_mirror_child_split_static_passed_cargo_deferred";
const STATUS_MIRROR_CHILD_SPLIT_GUARD_NAME: &str =
    "runtime_15_m3_child_group_moved_row_status_mirror_children_are_child_owned";

const STATUS_MIRROR_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_split_status",
        CHILD_SPLIT_STATUS_PATH,
        "runtime_15_m3_child_group_moved_row_status_mirror_status_rows_are_current",
    ),
    (
        "historical_status",
        HISTORICAL_STATUS_PATH,
        "runtime_15_m3_child_group_moved_row_historical_status_is_current",
    ),
    (
        "folder_backed_status",
        FOLDER_BACKED_STATUS_PATH,
        "runtime_15_m3_child_group_moved_row_folder_backed_status_mirrors_are_current",
    ),
];

#[test]
fn runtime_15_m3_child_group_moved_row_status_mirror_children_are_child_owned() {
    let route_source = read_runtime_src(STATUS_MIRRORS_ROUTE_PATH);

    for (module_name, path, guard_name) in STATUS_MIRROR_CHILDREN {
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "Runtime 15 M3 child-group moved-row status-mirror route mounts child",
            &route_source,
            &[module_mount.as_str(), *path, *guard_name],
        );
        let child_source = read_runtime_src(path);
        assert_contains_all(path, &child_source, &[*guard_name]);

        let line_count = child_source.lines().count();
        assert!(
            line_count < 90,
            "{path} should stay below its status-mirror child budget; got {line_count} lines"
        );
    }
    for forbidden in [
        ["let production_", "guard_support ="].concat(),
        ["let status_", "support_expected_status_map ="].concat(),
        ["let status_", "support_expected_date_map ="].concat(),
        ["let runtime_", "15_plan ="].concat(),
        ["let session_", "note ="].concat(),
    ] {
        assert!(
            !route_source.contains(&forbidden),
            "status_mirrors.rs should delegate status source reads to child files"
        );
    }
}
