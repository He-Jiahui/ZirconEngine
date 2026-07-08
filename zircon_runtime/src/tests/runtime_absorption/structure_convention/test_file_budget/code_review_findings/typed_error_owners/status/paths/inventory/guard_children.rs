use super::super::*;

pub(in super::super::super) const TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_CHILD,
        "runtime_15_typed_error_status_docs_are_folder_backed",
    ),
    (
        "doc_mirrors",
        TYPED_ERROR_STATUS_DOCS_DOC_MIRRORS_CHILD,
        "assert_typed_error_status_doc_mirrors_are_synced",
    ),
    (
        "status_maps",
        TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_CHILD,
        "assert_typed_error_status_maps_are_synced",
    ),
    (
        "status_mirrors",
        TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_CHILD,
        "runtime_15_typed_error_status_docs_folder_backed_status_is_current",
    ),
];
