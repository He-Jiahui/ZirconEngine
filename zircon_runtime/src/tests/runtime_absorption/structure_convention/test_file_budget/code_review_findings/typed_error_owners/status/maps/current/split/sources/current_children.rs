use super::super::super::super::super::super::super::super::*;
use super::super::super::super::super::{
    TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_CHILD,
    TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_GUARD,
    TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_OWNERSHIP_CHILD,
    TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_SOURCES_CHILD,
    TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_SPLIT_LAYOUT_CHILD,
    TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_STATUS_SYNC_CHILD,
    TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_GUARD,
};

pub(in super::super) const TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "ownership",
        TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_OWNERSHIP_CHILD,
        "assert_typed_error_status_maps_are_child_backed",
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

pub(in super::super) fn typed_error_status_map_status_current_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(in super::super) fn typed_error_status_map_status_current_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in typed_error_status_map_status_current_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
