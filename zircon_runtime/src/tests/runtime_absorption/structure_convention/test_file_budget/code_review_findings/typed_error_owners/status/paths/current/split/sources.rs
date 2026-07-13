use super::super::super::super::super::super::super::*;
use super::super::super::root_paths::{
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_BUDGETS_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_CHILD_INVENTORY_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_PARENT_BACKFLOW_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_PARENT_MOUNTS_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_SOURCES_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD,
};
use super::super::super::status_slices::{
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_GUARD,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_GUARD_GUARD,
};

pub(super) const TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "budgets",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_BUDGETS_CHILD,
        "assert_typed_error_status_doc_paths_status_current_children_line_budgets",
    ),
    (
        "child_inventory",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_CHILD_INVENTORY_CHILD,
        "assert_typed_error_status_doc_paths_status_current_split_layout_child_inventory",
    ),
    (
        "parent_backflow",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_PARENT_BACKFLOW_CHILD,
        "assert_typed_error_status_doc_paths_status_current_split_layout_parent_has_no_moved_checks",
    ),
    (
        "parent_mounts",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_PARENT_MOUNTS_CHILD,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_GUARD,
    ),
    (
        "sources",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_SOURCES_CHILD,
        "typed_error_status_doc_paths_status_current_split_layout_child_source_blob",
    ),
    (
        "split_layout",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_GUARD_GUARD,
    ),
    (
        "status_mirrors",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD,
        "assert_typed_error_status_doc_paths_status_current_split_layout_status_is_current",
    ),
];

pub(super) fn typed_error_status_doc_paths_status_current_split_layout_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_CHILDREN
        .iter()
        .map(|(_, path, _)| {
            (
                *path,
                typed_error_status_doc_paths_status_current_split_layout_child_source(path),
            )
        })
        .collect()
}

pub(super) fn typed_error_status_doc_paths_status_current_split_layout_child_source_blob() -> String
{
    let mut blob = String::new();
    for (_, source) in typed_error_status_doc_paths_status_current_split_layout_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}

fn typed_error_status_doc_paths_status_current_split_layout_child_source(path: &str) -> String {
    if path == TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD {
        super::status_mirrors::typed_error_status_doc_paths_status_current_split_layout_status_mirrors_child_source_blob()
    } else {
        read_runtime_src(path)
    }
}
