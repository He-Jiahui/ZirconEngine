use super::super::*;

pub(super) const TYPED_ERROR_STATUS_DOCS_STATUS_MIRROR_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "folder_backed_status",
        TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_FOLDER_BACKED_STATUS_CHILD,
        "assert_typed_error_status_docs_folder_backed_status_is_current",
    ),
    (
        "budgets",
        TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_BUDGETS_CHILD,
        "assert_typed_error_status_mirror_child_budgets_are_current",
    ),
    (
        "status_current",
        TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_CHILD,
        "#[path = \"current/status_sync.rs\"]",
    ),
];

pub(super) const TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "ownership",
        TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_OWNERSHIP_CHILD,
        TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_OWNERSHIP_GUARD,
    ),
    (
        "status_sync",
        TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_STATUS_SYNC_CHILD,
        TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_GUARD,
    ),
    (
        "sources",
        TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_SOURCES_CHILD,
        "typed_error_status_mirror_child_source_blob",
    ),
    (
        "split_layout",
        TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_SPLIT_LAYOUT_CHILD,
        "mod split_layout;",
    ),
];
