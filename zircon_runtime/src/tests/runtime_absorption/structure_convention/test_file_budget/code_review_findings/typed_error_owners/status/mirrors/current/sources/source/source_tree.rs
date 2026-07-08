use super::super::super::super::super::super::super::super::*;
use super::super::super::super::super::{
    typed_error_status_doc_root_paths_source_blob,
    TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_SOURCES_SOURCE_SPLIT_CHILD,
    TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_SOURCES_STATUS_CURRENT_CHILDREN_CHILD,
    TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_SOURCES_STATUS_MIRRORS_CHILDREN_CHILD,
};

const TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_SOURCES_SOURCE_SPLIT_CHILDREN:
    &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/mirrors/current/sources/source/source_tree.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/mirrors/current/sources/source/status_current.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/mirrors/current/sources/source/folder_backed.rs",
];

pub(super) fn typed_error_status_mirrors_status_current_sources_guard_blob() -> String {
    let mut blob = read_runtime_src(
        TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_SOURCES_SOURCE_SPLIT_CHILD,
    );
    blob.push('\n');
    for child_path in typed_error_status_mirrors_status_current_sources_guard_children() {
        blob.push_str(&read_runtime_src(child_path));
        blob.push('\n');
    }
    for child_path in [
        TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_SOURCES_STATUS_CURRENT_CHILDREN_CHILD,
        TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_SOURCES_STATUS_MIRRORS_CHILDREN_CHILD,
    ] {
        blob.push_str(&read_runtime_src(child_path));
        blob.push('\n');
    }
    blob.push_str(&typed_error_status_doc_root_paths_source_blob());
    blob
}

pub(super) fn typed_error_status_mirrors_status_current_sources_guard_children(
) -> &'static [&'static str] {
    TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_SOURCES_SOURCE_SPLIT_CHILDREN
}
