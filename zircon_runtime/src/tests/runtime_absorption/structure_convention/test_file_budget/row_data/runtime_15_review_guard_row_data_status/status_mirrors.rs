use super::*;

#[path = "mirrors/child_split_status.rs"]
mod child_split_status;
#[path = "mirrors/folder_backed_status.rs"]
mod folder_backed_status;
#[path = "mirrors/review_guard_status.rs"]
mod review_guard_status;
#[path = "mirrors/status_doc_status.rs"]
mod status_doc_status;

const STATUS_MIRRORS_ROUTE_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/status_mirrors.rs";
const CHILD_SPLIT_STATUS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/mirrors/child_split_status.rs";
const REVIEW_GUARD_STATUS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/mirrors/review_guard_status.rs";
const STATUS_DOC_STATUS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/mirrors/status_doc_status.rs";
const FOLDER_BACKED_STATUS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/mirrors/folder_backed_status.rs";

const STATUS_MIRROR_CHILD_SPLIT_NAME: &str =
    "Runtime 15 M3 review-guard row-data status-doc status-mirror child split";
const STATUS_MIRROR_CHILD_SPLIT_ID: &str = "runtime_15_review_guard_row_data_status_docs_status_mirror_child_split_static_passed_cargo_deferred";

const STATUS_MIRROR_CHILDREN: &[(&str, &str, &str, &[&str])] = &[
    (
        "child_split_status",
        CHILD_SPLIT_STATUS_PATH,
        "runtime_15_review_guard_row_data_status_doc_status_mirror_children_are_child_owned",
        &[
            "STATUS_MIRROR_CHILD_SPLIT_NAME",
            "STATUS_MIRROR_CHILD_SPLIT_ID",
        ],
    ),
    (
        "review_guard_status",
        REVIEW_GUARD_STATUS_PATH,
        "runtime_15_review_guard_row_data_status_doc_review_guard_status_is_current",
        &[
            "REVIEW_GUARD_CHILD_OWNER_STATUS_NAME",
            "REVIEW_GUARD_CHILD_OWNER_STATUS_ID",
            "TOPIC_CHILD_OWNER_STATUS_NAME",
            "TOPIC_CHILD_OWNER_STATUS_ID",
        ],
    ),
    (
        "status_doc_status",
        STATUS_DOC_STATUS_PATH,
        "runtime_15_review_guard_row_data_status_doc_status_doc_status_is_current",
        &[
            "STATUS_DOC_CHILD_OWNER_STATUS_NAME",
            "STATUS_DOC_CHILD_OWNER_STATUS_ID",
        ],
    ),
    (
        "folder_backed_status",
        FOLDER_BACKED_STATUS_PATH,
        "runtime_15_review_guard_row_data_status_doc_folder_backed_status_is_current",
        &["FOLDER_BACKED_STATUS_NAME", "FOLDER_BACKED_STATUS_ID"],
    ),
];

#[test]
fn runtime_15_review_guard_row_data_status_doc_folder_backed_status_mirrors_are_current() {
    let route_source = read_runtime_src(STATUS_MIRRORS_ROUTE_PATH);

    for (module_name, path, guard_name, labels) in STATUS_MIRROR_CHILDREN {
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "review-guard status-doc status-mirror route mounts child",
            &route_source,
            &[module_mount.as_str(), *path, *guard_name],
        );
        let child_source = read_runtime_src(path);
        assert_contains_all(path, &child_source, &[*guard_name]);
        assert_contains_all(path, &child_source, labels);

        let line_count = child_source.lines().count();
        assert!(
            line_count < 95,
            "{path} should stay below its status-mirror child budget; got {line_count} lines"
        );
    }
    let forbidden_status_reader = ["let review_guard_status_support_rows", " ="].concat();
    assert!(
        !route_source.contains(&forbidden_status_reader),
        "status_mirrors.rs should delegate status source reads to child files"
    );
}
