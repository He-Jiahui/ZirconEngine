pub(super) fn assert_typed_error_status_doc_paths_status_current_parent_has_no_moved_checks(
    parent: &str,
) {
    for moved_anchor in [
        "fn assert_typed_error_status_doc_paths_status_is_current",
        "fn typed_error_status_doc_paths_child_sources",
        "fn typed_error_status_doc_paths_child_source_blob",
        "typed-error status-doc paths parent mounts child owners",
        "M3 review status map records typed-error status-doc paths split",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc paths status-current `{moved_anchor}` should stay in child files"
        );
    }
}

pub(super) fn assert_typed_error_status_doc_paths_status_current_split_layout_parent_has_no_moved_checks(
    parent: &str,
) {
    for moved_anchor in [
        "fn assert_typed_error_status_doc_paths_status_current_status_is_current",
        "for (_, child_path, anchor) in TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_CHILDREN",
        "typed-error status-doc paths status-current child tree should inventory",
        "Runtime 15 status-doc paths status-current budget",
        "TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_CHILDREN",
        "fn typed_error_status_doc_paths_status_current_split_layout_child_sources",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc paths status-current split-layout `{moved_anchor}` should stay in focused children"
        );
    }
}
