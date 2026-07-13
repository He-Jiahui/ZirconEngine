use super::*;

#[path = "mirrors/child_split_status.rs"]
mod child_split_status;
#[path = "mirrors/code_review_owner.rs"]
mod code_review_owner;
#[path = "mirrors/folder_backed.rs"]
mod folder_backed;
#[path = "mirrors/structure_guard_rows.rs"]
mod structure_guard_rows;

const STATUS_MIRROR_ROUTE_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/status_mirrors.rs";
const CHILD_SPLIT_STATUS_MIRROR_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/mirrors/child_split_status.rs";
const CODE_REVIEW_OWNER_STATUS_MIRROR_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/mirrors/code_review_owner.rs";
const STRUCTURE_GUARD_STATUS_MIRROR_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/mirrors/structure_guard_rows.rs";
const FOLDER_BACKED_STATUS_MIRROR_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/mirrors/folder_backed.rs";

const STATUS_MIRROR_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_split_status",
        CHILD_SPLIT_STATUS_MIRROR_PATH,
        "runtime_15_review_guard_code_review_status_mirror_status_rows_are_current",
    ),
    (
        "code_review_owner",
        CODE_REVIEW_OWNER_STATUS_MIRROR_PATH,
        "runtime_15_review_guard_code_review_rows_status_mirrors_are_current",
    ),
    (
        "structure_guard_rows",
        STRUCTURE_GUARD_STATUS_MIRROR_PATH,
        "runtime_15_review_guard_code_review_structure_status_mirrors_are_current",
    ),
    (
        "folder_backed",
        FOLDER_BACKED_STATUS_MIRROR_PATH,
        "runtime_15_review_guard_code_review_folder_backed_status_mirrors_are_current",
    ),
];

#[test]
fn runtime_15_review_guard_code_review_status_mirror_children_are_folder_backed() {
    let route_source = read_runtime_src(STATUS_MIRROR_ROUTE_PATH);
    for (module_name, path, guard_name) in STATUS_MIRROR_CHILDREN {
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "status_mirrors route mounts every child",
            &route_source,
            &[module_mount.as_str(), *path, *guard_name],
        );
        let child_source = read_runtime_src(path);
        assert_contains_all(path, &child_source, &[*guard_name]);
    }
    for forbidden in [
        ["CODE_REVIEW_ROWS", "_ROW_DATA_STATUS_NAME"].concat(),
        ["STRUCTURE_GUARD", "_ROW_DATA_STATUS_NAME"].concat(),
        ["FOLDER_BACKED", "_STATUS_NAME"].concat(),
        ["let runtime_", "15_plan"].concat(),
    ] {
        assert!(
            !route_source.contains(&forbidden),
            "status_mirrors.rs should route status mirror groups to child files"
        );
    }
}
