use super::*;

const STATUS_SUPPORT_STATUS_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/priority_plan_doc_maps.rs";
const STATUS_SUPPORT_DATE_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/priority_plan_doc_maps.rs";

#[path = "mirrors/child_split_status.rs"]
mod child_split_status;
#[path = "mirrors/folder_backed_status.rs"]
mod folder_backed_status;
#[path = "mirrors/owner_guard_status.rs"]
mod owner_guard_status;
#[path = "mirrors/row_owner_status.rs"]
mod row_owner_status;

const STATUS_MIRRORS_ROUTE_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/status_mirrors.rs";
const CHILD_SPLIT_STATUS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/mirrors/child_split_status.rs";
const ROW_OWNER_STATUS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/mirrors/row_owner_status.rs";
const FOLDER_BACKED_STATUS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/mirrors/folder_backed_status.rs";
const OWNER_GUARD_STATUS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/mirrors/owner_guard_status.rs";

const STATUS_MIRROR_CHILD_SPLIT_NAME: &str =
    "Runtime 15 M3 priority plan docs status-mirror child split";
const STATUS_MIRROR_CHILD_SPLIT_ID: &str =
    "runtime_15_priority_plan_docs_status_mirror_child_split_static_passed_cargo_deferred";

const STATUS_MIRROR_CHILDREN: &[(&str, &str, &str, &[&str])] = &[
    (
        "child_split_status",
        CHILD_SPLIT_STATUS_PATH,
        "runtime_15_priority_plan_docs_status_mirror_children_are_child_owned",
        &[
            "STATUS_MIRROR_CHILD_SPLIT_NAME",
            "STATUS_MIRROR_CHILD_SPLIT_ID",
        ],
    ),
    (
        "row_owner_status",
        ROW_OWNER_STATUS_PATH,
        "runtime_15_priority_plan_docs_row_owner_status_rows_are_current",
        &["HISTORICAL_STATUS_NAME", "HISTORICAL_STATUS_ID"],
    ),
    (
        "folder_backed_status",
        FOLDER_BACKED_STATUS_PATH,
        "runtime_15_priority_plan_docs_folder_backed_status_rows_are_current",
        &["FOLDER_BACKED_STATUS_NAME", "FOLDER_BACKED_STATUS_ID"],
    ),
    (
        "owner_guard_status",
        OWNER_GUARD_STATUS_PATH,
        "runtime_15_priority_plan_docs_owner_guard_status_rows_are_current",
        &[
            "OWNER_GUARD_CHILD_STATUS_NAME",
            "OWNER_GUARD_CHILD_STATUS_ID",
        ],
    ),
];

#[test]
fn runtime_15_priority_plan_docs_row_data_guard_folder_backed_status_mirrors_are_current() {
    let route_source = read_runtime_src(STATUS_MIRRORS_ROUTE_PATH);

    for (module_name, path, guard_name, labels) in STATUS_MIRROR_CHILDREN {
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "priority-plan-doc status-mirror route mounts child",
            &route_source,
            &[module_mount.as_str(), *path, *guard_name],
        );
        let child_source = read_runtime_src(path);
        assert_contains_all(path, &child_source, &[*guard_name]);
        assert_contains_all(path, &child_source, labels);

        let line_count = child_source.lines().count();
        assert!(
            line_count < 100,
            "{path} should stay below its status-mirror child budget; got {line_count} lines"
        );
    }
    let delegated_status_read = ["let status_", "map ="].concat();
    assert!(
        !route_source.contains(&delegated_status_read),
        "status_mirrors.rs should delegate status source reads to child files"
    );
}
