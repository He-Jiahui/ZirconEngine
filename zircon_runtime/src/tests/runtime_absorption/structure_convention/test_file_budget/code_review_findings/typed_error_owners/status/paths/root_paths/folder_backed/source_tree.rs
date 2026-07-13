use super::super::super::super::super::super::super::*;
use super::super::path_children::TYPED_ERROR_STATUS_DOCS_PATHS_ROOT_PATHS_CHILD;

pub(super) const TYPED_ERROR_STATUS_DOCS_ROOT_PATHS_FOLDER_BACKED_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/root_paths/folder_backed.rs";

const TYPED_ERROR_STATUS_DOCS_ROOT_PATHS_FOLDER_BACKED_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/root_paths/folder_backed/source_tree.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/root_paths/folder_backed/status_current.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/root_paths/folder_backed/folder_backed.rs",
];

pub(super) fn typed_error_status_doc_root_paths_folder_backed_guard_blob() -> String {
    let mut blob = read_runtime_src(TYPED_ERROR_STATUS_DOCS_ROOT_PATHS_FOLDER_BACKED_CHILD);
    blob.push('\n');
    for child_path in typed_error_status_doc_root_paths_folder_backed_guard_children() {
        blob.push_str(&read_runtime_src(child_path));
        blob.push('\n');
    }
    blob.push_str(&read_runtime_src(
        TYPED_ERROR_STATUS_DOCS_PATHS_ROOT_PATHS_CHILD,
    ));
    blob
}

pub(super) fn typed_error_status_doc_root_paths_folder_backed_guard_children(
) -> &'static [&'static str] {
    TYPED_ERROR_STATUS_DOCS_ROOT_PATHS_FOLDER_BACKED_CHILDREN
}
