use super::super::super::super::super::super::*;
use super::path_children::TYPED_ERROR_STATUS_DOCS_PATHS_ROOT_PATHS_CHILD;

const TYPED_ERROR_STATUS_DOCS_ROOT_PATH_SOURCE_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/root_paths/path_children.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/root_paths/status_doc_core.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/root_paths/delegation.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/root_paths/status_maps.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/root_paths/status_mirrors.rs",
];

pub(in super::super::super) fn typed_error_status_doc_root_paths_source_blob() -> String {
    let mut blob = read_runtime_src(TYPED_ERROR_STATUS_DOCS_PATHS_ROOT_PATHS_CHILD);
    blob.push('\n');
    for child_path in typed_error_status_doc_root_paths_child_source_paths() {
        blob.push_str(&read_runtime_src(child_path));
        blob.push('\n');
    }
    blob
}

pub(in super::super) fn typed_error_status_doc_root_paths_child_source_paths(
) -> &'static [&'static str] {
    TYPED_ERROR_STATUS_DOCS_ROOT_PATH_SOURCE_CHILDREN
}
