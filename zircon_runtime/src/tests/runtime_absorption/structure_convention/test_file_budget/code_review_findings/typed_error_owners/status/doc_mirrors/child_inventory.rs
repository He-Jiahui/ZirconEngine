use super::*;

pub(super) const TYPED_ERROR_STATUS_DOC_MIRROR_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "status_slices",
        TYPED_ERROR_STATUS_DOC_MIRRORS_STATUS_SLICES_CHILD,
        "assert_typed_error_status_doc_slice_anchors_are_synced",
    ),
    (
        "source_paths",
        TYPED_ERROR_STATUS_DOC_MIRRORS_SOURCE_PATHS_CHILD,
        "assert_typed_error_status_doc_source_paths_are_synced",
    ),
    (
        "guard_anchors",
        TYPED_ERROR_STATUS_DOC_MIRRORS_GUARD_ANCHORS_CHILD,
        "assert_typed_error_status_doc_guard_anchors_are_synced",
    ),
    (
        "status_current",
        TYPED_ERROR_STATUS_DOC_MIRRORS_STATUS_CURRENT_CHILD,
        TYPED_ERROR_STATUS_DOC_MIRRORS_STATUS_GUARD,
    ),
];

pub(super) const TYPED_ERROR_STATUS_DOC_MIRROR_SOURCE_HELPER_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_inventory",
        TYPED_ERROR_STATUS_DOC_MIRRORS_CHILD_INVENTORY_CHILD,
        "TYPED_ERROR_STATUS_DOC_MIRROR_SOURCE_HELPER_CHILDREN",
    ),
    (
        "metadata",
        TYPED_ERROR_STATUS_DOC_MIRRORS_METADATA_CHILD,
        TYPED_ERROR_STATUS_DOC_MIRRORS_SOURCE_HELPER_SLICE,
    ),
    (
        "source_helper_ownership",
        TYPED_ERROR_STATUS_DOC_MIRRORS_SOURCE_HELPER_OWNERSHIP_CHILD,
        TYPED_ERROR_STATUS_DOC_MIRRORS_SOURCE_HELPER_OWNERSHIP_GUARD,
    ),
    (
        "source_helper_status",
        TYPED_ERROR_STATUS_DOC_MIRRORS_SOURCE_HELPER_STATUS_CHILD,
        TYPED_ERROR_STATUS_DOC_MIRRORS_SOURCE_HELPER_STATUS_GUARD,
    ),
    (
        "sources",
        TYPED_ERROR_STATUS_DOC_MIRRORS_SOURCES_CHILD,
        "pub(super) fn typed_error_status_doc_mirror_sources",
    ),
];
