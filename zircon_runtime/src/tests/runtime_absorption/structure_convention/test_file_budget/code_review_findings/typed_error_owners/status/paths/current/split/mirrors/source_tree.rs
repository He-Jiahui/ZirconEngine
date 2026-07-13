use super::super::super::super::super::super::super::super::*;
use super::super::super::super::root_paths::TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD;

const TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_STATUS_MIRROR_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/current/split/mirrors/source_tree.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/current/split/mirrors/status_current.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/current/split/mirrors/status_documents.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/current/split/mirrors/status_maps.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/current/split/mirrors/folder_backed.rs",
];

pub(in super::super) fn typed_error_status_doc_paths_status_current_split_layout_status_mirrors_child_source_blob(
) -> String {
    let mut blob = read_runtime_src(
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD,
    );
    blob.push('\n');
    for child_path in
        typed_error_status_doc_paths_status_current_split_layout_status_mirrors_child_paths()
    {
        blob.push_str(&read_runtime_src(child_path));
        blob.push('\n');
    }
    blob
}

pub(super) fn typed_error_status_doc_paths_status_current_split_layout_status_mirrors_child_paths(
) -> &'static [&'static str] {
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_STATUS_MIRROR_CHILDREN
}
