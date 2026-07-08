use super::super::*;

pub(in super::super::super) const TYPED_ERROR_STATUS_DOCS_PATHS_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        TYPED_ERROR_STATUS_DOCS_PATHS_ROOT_PATHS_CHILD,
        "TYPED_ERROR_STATUS_DOCS_PATHS_CHILD",
    ),
    (
        "review_guard_paths",
        TYPED_ERROR_STATUS_DOCS_PATHS_REVIEW_GUARD_PATHS_CHILD,
        "REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH",
    ),
    (
        "status_slices",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_SLICES_CHILD,
        "#[path = \"status_slices/paths.rs\"]",
    ),
    (
        "child_inventory",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_CHILD,
        "TYPED_ERROR_STATUS_DOCS_PATHS_CHILDREN",
    ),
    (
        "status_current",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_CHILD,
        "#[path = \"current/status_sync.rs\"]",
    ),
];
