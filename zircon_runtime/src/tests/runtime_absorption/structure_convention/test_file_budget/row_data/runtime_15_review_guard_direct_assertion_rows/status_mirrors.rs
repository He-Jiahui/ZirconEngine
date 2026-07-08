use super::*;

#[path = "mirrors/child_owner_status.rs"]
mod child_owner_status;
#[path = "mirrors/child_split_status.rs"]
mod child_split_status;
#[path = "mirrors/folder_backed_status.rs"]
mod folder_backed_status;

const STATUS_MIRRORS_ROUTE_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/status_mirrors.rs";
const CHILD_OWNER_STATUS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/mirrors/child_owner_status.rs";
const CHILD_SPLIT_STATUS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/mirrors/child_split_status.rs";
const FOLDER_BACKED_STATUS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/mirrors/folder_backed_status.rs";

const STATUS_MIRROR_CHILD_SPLIT_NAME: &str =
    "Runtime 15 M3 review-guard direct-assertion status-mirror child split";
const STATUS_MIRROR_CHILD_SPLIT_ID: &str =
    "runtime_15_review_guard_direct_assertion_status_mirror_child_split_static_passed_cargo_deferred";

const STATUS_MIRROR_CHILDREN: &[(&str, &str, &str, &[&str])] = &[
    (
        "child_owner_status",
        CHILD_OWNER_STATUS_PATH,
        "runtime_15_review_guard_direct_assertion_child_owner_status_is_current",
        &["CHILD_OWNER_STATUS_NAME", "CHILD_OWNER_STATUS_ID"],
    ),
    (
        "child_split_status",
        CHILD_SPLIT_STATUS_PATH,
        "runtime_15_review_guard_direct_assertion_status_mirror_children_are_child_owned",
        &[
            "STATUS_MIRROR_CHILD_SPLIT_NAME",
            "STATUS_MIRROR_CHILD_SPLIT_ID",
        ],
    ),
    (
        "folder_backed_status",
        FOLDER_BACKED_STATUS_PATH,
        "runtime_15_review_guard_direct_assertion_folder_backed_status_is_current",
        &["FOLDER_BACKED_STATUS_NAME", "FOLDER_BACKED_STATUS_ID"],
    ),
];

#[test]
fn runtime_15_review_guard_direct_assertion_folder_backed_status_mirrors_are_current() {
    let route_source = read_runtime_src(STATUS_MIRRORS_ROUTE_PATH);

    for (module_name, path, guard_name, labels) in STATUS_MIRROR_CHILDREN {
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "review-guard direct-assertion status-mirror route mounts child",
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
    assert!(
        !route_source.contains("let status_support_rows ="),
        "status_mirrors.rs should delegate status source reads to child files"
    );
}
