use super::*;

#[path = "mirrors/child_split_status.rs"]
mod child_split_status;
#[path = "mirrors/folder_backed_status.rs"]
mod folder_backed_status;
#[path = "mirrors/historical_status.rs"]
mod historical_status;

const STATUS_MIRRORS_ROUTE_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_lock_poison_status_row_data_children/status_mirrors.rs";
const CHILD_SPLIT_STATUS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_lock_poison_status_row_data_children/mirrors/child_split_status.rs";
const HISTORICAL_STATUS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_lock_poison_status_row_data_children/mirrors/historical_status.rs";
const FOLDER_BACKED_STATUS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_lock_poison_status_row_data_children/mirrors/folder_backed_status.rs";
const M3_STRUCTURE_SUPPORT_STATUS_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/row_data_owner_maps.rs";
const M3_STRUCTURE_SUPPORT_DATE_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/row_data_owner_maps.rs";
const PRODUCTION_GUARD_SUPPORT_ROWS_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/lock_poison_scene_script_rows.rs";

const STATUS_MIRROR_CHILD_SPLIT_NAME: &str =
    "Runtime 15 M3 lock-poison status row-data status-mirror child split";
const STATUS_MIRROR_CHILD_SPLIT_ID: &str =
    "runtime_15_lock_poison_status_row_data_status_mirror_child_split_static_passed_cargo_deferred";
const STATUS_MIRROR_CHILD_SPLIT_GUARD_NAME: &str =
    "runtime_15_lock_poison_status_row_data_status_mirror_children_are_child_owned";

const STATUS_MIRROR_CHILDREN: &[(&str, &str, &str, &[&str])] = &[
    (
        "child_split_status",
        CHILD_SPLIT_STATUS_PATH,
        "runtime_15_lock_poison_status_row_data_status_mirror_status_rows_are_current",
        &[
            "STATUS_MIRROR_CHILD_SPLIT_NAME",
            "STATUS_MIRROR_CHILD_SPLIT_ID",
        ],
    ),
    (
        "historical_status",
        HISTORICAL_STATUS_PATH,
        "runtime_15_lock_poison_status_row_data_historical_status_is_current",
        &["CHILD_OWNER_STATUS_NAME", "CHILD_OWNER_STATUS_ID"],
    ),
    (
        "folder_backed_status",
        FOLDER_BACKED_STATUS_PATH,
        "runtime_15_lock_poison_status_row_data_folder_backed_status_mirrors_are_current",
        &["FOLDER_BACKED_STATUS_NAME", "FOLDER_BACKED_STATUS_ID"],
    ),
];

#[test]
fn runtime_15_lock_poison_status_row_data_status_mirror_children_are_child_owned() {
    let route_source = read_runtime_src(STATUS_MIRRORS_ROUTE_PATH);

    for (module_name, path, guard_name, labels) in STATUS_MIRROR_CHILDREN {
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "lock-poison status row-data status-mirror route mounts child",
            &route_source,
            &[module_mount.as_str(), *path, *guard_name],
        );
        let child_source = read_runtime_src(path);
        assert_contains_all(path, &child_source, &[*guard_name]);
        assert_contains_all(path, &child_source, labels);

        let line_count = child_source.lines().count();
        assert!(
            line_count < 90,
            "{path} should stay below its status-mirror child budget; got {line_count} lines"
        );
    }
    for forbidden in [
        ["let row_data_", "owner ="].concat(),
        ["let production_", "guard_support ="].concat(),
        ["let runtime_", "15_plan ="].concat(),
        ["let session_", "note ="].concat(),
    ] {
        assert!(
            !route_source.contains(&forbidden),
            "status_mirrors.rs should delegate status source reads to child files"
        );
    }
}
