use super::super::super::super::super::super::super::super::*;
use super::super::super::super::root_paths::{
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_GUARD_CHILDREN_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_PATHS_CHILDREN_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SOURCE_HELPER_CHILDREN_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_STATUS_CURRENT_CHILDREN_CHILD,
};

pub(in super::super) const TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "guard_children",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_GUARD_CHILDREN_CHILD,
        "TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN",
    ),
    (
        "source_helper_children",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SOURCE_HELPER_CHILDREN_CHILD,
        "TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_CHILDREN",
    ),
    (
        "paths_children",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_PATHS_CHILDREN_CHILD,
        "TYPED_ERROR_STATUS_DOCS_PATHS_CHILDREN",
    ),
    (
        "status_current_children",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_STATUS_CURRENT_CHILDREN_CHILD,
        "TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_CHILDREN",
    ),
    (
        "split_layout",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_CHILD,
        "mod split_layout;",
    ),
];

pub(in super::super) fn typed_error_status_doc_paths_child_inventory_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(in super::super) fn typed_error_status_doc_paths_child_inventory_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in typed_error_status_doc_paths_child_inventory_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
