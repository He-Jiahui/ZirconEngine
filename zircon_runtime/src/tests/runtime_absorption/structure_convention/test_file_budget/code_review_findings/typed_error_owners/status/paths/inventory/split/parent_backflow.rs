pub(super) fn assert_typed_error_status_doc_paths_child_inventory_parent_has_no_moved_checks(
    parent: &str,
) {
    for moved_anchor in [
        "pub(in super::super) const TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN",
        "pub(in super::super) const TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_CHILDREN",
        "pub(super) const TYPED_ERROR_STATUS_DOCS_PATHS_CHILDREN",
        "pub(super) const TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_CHILDREN",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc paths child-inventory `{moved_anchor}` should stay in child files"
        );
    }
}

pub(super) fn assert_typed_error_status_doc_paths_child_inventory_split_layout_parent_has_no_moved_checks(
    parent: &str,
) {
    for moved_anchor in [
        "typed-error status-doc paths child-inventory parent mounts focused children",
        "for (module_name, child_path, anchor) in TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_CHILDREN",
        "let status_rows = typed_error_status_row_source",
        "M3 review status map records typed-error status-doc paths child-inventory split",
        "Runtime 15 status-doc paths child-inventory budget",
        "const TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_CHILDREN",
        "typed_error_status_doc_paths_child_inventory_split_layout_child_sources",
        "runtime_15_typed_error_status_doc_paths_child_inventory_split_layout_is_folder_backed",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc paths child-inventory split-layout `{moved_anchor}` should stay in focused children"
        );
    }
}
