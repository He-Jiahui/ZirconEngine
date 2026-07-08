use super::super::*;

pub(in super::super::super) const TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "child_sources",
        TYPED_ERROR_STATUS_DOCS_CHILD_SOURCES_CHILD,
        "pub(super) fn typed_error_status_docs_child_source_blob",
    ),
    (
        "paths",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD,
        "pub(super) use child_inventory::*;",
    ),
    (
        "source_helper_ownership",
        TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_OWNERSHIP_CHILD,
        TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_OWNERSHIP_GUARD,
    ),
    (
        "source_helper_status",
        TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_STATUS_CHILD,
        TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_STATUS_GUARD,
    ),
    (
        "sources",
        TYPED_ERROR_STATUS_DOCS_SOURCES_CHILD,
        "pub(super) fn typed_error_status_doc_sources",
    ),
];
