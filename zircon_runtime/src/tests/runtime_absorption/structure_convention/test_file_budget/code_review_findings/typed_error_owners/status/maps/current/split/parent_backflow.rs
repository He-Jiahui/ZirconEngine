pub(super) fn assert_typed_error_status_map_status_current_parent_has_no_moved_checks(
    parent: &str,
) {
    for moved_anchor in [
        "fn assert_typed_error_status_maps_status_is_current",
        "fn typed_error_status_map_child_sources",
        "fn typed_error_status_map_child_source_blob",
        "typed-error status-doc status maps parent mounts child owners",
        "M3 review status map records typed-error status-doc status maps split",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc status maps status-current `{moved_anchor}` should stay in child files"
        );
    }
}

pub(super) fn assert_typed_error_status_map_status_current_split_layout_parent_has_no_moved_checks(
    parent: &str,
) {
    for moved_anchor in [
        "typed-error status-doc status maps status-current parent mounts child owners",
        "for (_, child_path, anchor) in TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_CHILDREN",
        "assert_typed_error_status_map_status_current_status_is_current",
        "M3 review status map records typed-error status-doc status maps status-current split",
        "runtime_15_typed_error_status_doc_status_maps_status_current_is_child_backed",
        "TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_CHILDREN",
        "typed_error_status_map_status_current_split_layout_child_sources",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc status maps status-current split-layout `{moved_anchor}` should stay in focused children"
        );
    }
}
