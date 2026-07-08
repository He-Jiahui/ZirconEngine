use super::super::super::super::super::super::super::super::*;
use super::super::super::super::root_paths::{
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_BUDGETS_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_CHILD_INVENTORY_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_PARENT_BACKFLOW_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_PARENT_MOUNTS_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_SOURCES_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD,
};
use super::super::super::super::status_slices::{
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_GUARD,
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_GUARD_GUARD,
};
use super::super::status_mirrors::typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_child_source_blob;

pub(in super::super) const TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_CHILDREN:
    &[(&str, &str, &str)] = &[
    (
        "budgets",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_BUDGETS_CHILD,
        "assert_typed_error_status_doc_paths_child_inventory_split_layout_children_line_budgets",
    ),
    (
        "child_inventory",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_CHILD_INVENTORY_CHILD,
        "assert_typed_error_status_doc_paths_child_inventory_split_layout_child_inventory",
    ),
    (
        "parent_backflow",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_PARENT_BACKFLOW_CHILD,
        "assert_typed_error_status_doc_paths_child_inventory_split_layout_parent_has_no_moved_checks",
    ),
    (
        "parent_mounts",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_PARENT_MOUNTS_CHILD,
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_GUARD,
    ),
    (
        "sources",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_SOURCES_CHILD,
        "pub(super) use split_layout_children::*;",
    ),
    (
        "split_layout",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD,
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_GUARD_GUARD,
    ),
    (
        "status_mirrors",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD,
        "assert_typed_error_status_doc_paths_child_inventory_split_layout_status_is_current",
    ),
];

pub(in super::super) fn typed_error_status_doc_paths_child_inventory_split_layout_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(in super::super) fn typed_error_status_doc_paths_child_inventory_split_layout_child_source_blob(
) -> String {
    let mut blob = String::new();
    for (path, source) in typed_error_status_doc_paths_child_inventory_split_layout_child_sources()
    {
        blob.push_str(&source);
        blob.push('\n');
        if path == TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD {
            blob.push_str(
                &typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_child_source_blob(
                ),
            );
            blob.push('\n');
        }
    }
    blob
}
