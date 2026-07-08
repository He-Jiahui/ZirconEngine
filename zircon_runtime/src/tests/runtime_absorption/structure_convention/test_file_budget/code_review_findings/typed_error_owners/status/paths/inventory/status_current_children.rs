use super::super::*;

pub(in super::super::super) const TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "ownership",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_OWNERSHIP_CHILD,
        "assert_typed_error_status_doc_paths_are_child_backed",
    ),
    (
        "status_sync",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_STATUS_SYNC_CHILD,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_GUARD,
    ),
    (
        "sources",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_CHILD,
        "typed_error_status_doc_paths_child_source_blob",
    ),
    (
        "split_layout",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_CHILD,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_GUARD,
    ),
];
