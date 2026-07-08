use super::super::*;

pub(super) const TYPED_ERROR_STATUS_DOCS_DELEGATION_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "typed_error_parent",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_TYPED_ERROR_PARENT_CHILD,
        "assert_typed_error_parent_delegates_status_docs",
    ),
    (
        "status_doc_parent",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_DOC_PARENT_CHILD,
        "assert_typed_error_status_doc_parent_delegates_children",
    ),
    (
        "child_tree",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_CHILD_TREE_CHILD,
        "assert_typed_error_status_doc_children_own_delegated_assertions",
    ),
    (
        "budgets",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_BUDGETS_CHILD,
        "assert_typed_error_status_doc_delegation_budgets_are_current",
    ),
    (
        "status_current",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_CHILD,
        "#[path = \"current/status_sync.rs\"]",
    ),
];

pub(super) const TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "ownership",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_OWNERSHIP_CHILD,
        TYPED_ERROR_STATUS_DOCS_DELEGATION_OWNERSHIP_GUARD,
    ),
    (
        "status_sync",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_STATUS_SYNC_CHILD,
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_GUARD,
    ),
    (
        "sources",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SOURCES_CHILD,
        "typed_error_status_doc_delegation_child_source_blob",
    ),
    (
        "split_layout",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_CHILD,
        "mod split_layout;",
    ),
];
