use super::super::*;

pub(super) const TYPED_ERROR_STATUS_DOCS_STATUS_MAP_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "review_slices",
        TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_REVIEW_SLICES_CHILD,
        "assert_typed_error_review_status_maps_are_synced",
    ),
    (
        "status_current",
        TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_CHILD,
        "#[path = \"current/status_sync.rs\"]",
    ),
];

pub(super) const TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "ownership",
        TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_OWNERSHIP_CHILD,
        TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_OWNERSHIP_GUARD,
    ),
    (
        "status_sync",
        TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_STATUS_SYNC_CHILD,
        TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_GUARD,
    ),
    (
        "sources",
        TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_SOURCES_CHILD,
        "typed_error_status_map_child_source_blob",
    ),
    (
        "split_layout",
        TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_SPLIT_LAYOUT_CHILD,
        "mod split_layout;",
    ),
];
