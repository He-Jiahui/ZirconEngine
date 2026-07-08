use super::super::super::super::super::super::*;
use super::super::super::*;
use super::super::*;

pub(in super::super) fn assert_typed_error_status_doc_paths_are_child_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_PATHS_CHILD);
    let child_tree = super::sources::typed_error_status_doc_paths_child_source_blob();

    assert_contains_all(
        "typed-error status-doc paths parent mounts child owners",
        &parent,
        &[
            "#[path = \"paths/child_inventory.rs\"]",
            "mod child_inventory;",
            "#[path = \"paths/review_guard_paths.rs\"]",
            "mod review_guard_paths;",
            "#[path = \"paths/root_paths.rs\"]",
            "mod root_paths;",
            "#[path = \"paths/status_current.rs\"]",
            "mod status_current;",
            "#[path = \"paths/status_slices.rs\"]",
            "mod status_slices;",
            "pub(super) use child_inventory::*;",
            "pub(super) use review_guard_paths::*;",
            "pub(super) use root_paths::*;",
            "pub(super) use status_slices::*;",
            TYPED_ERROR_STATUS_DOCS_PATHS_OWNERSHIP_GUARD,
        ],
    );
    for moved_anchor in [
        "pub(super) const TYPED_ERROR_STRUCTURE_CHILD",
        "pub(super) const REVIEW_GUARD_STATUS_ROWS_PATH",
        "pub(super) const TYPED_ERROR_STATUS_DOCS_GUARD_SPLIT_NAME",
        "pub(super) const TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN",
        "pub(super) const TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_CHILDREN",
        "pub(super) const TYPED_ERROR_CHILD_OWNER_LINE_BUDGET",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc paths `{moved_anchor}` should stay in child files"
        );
    }
    for (_, child_path, anchor) in TYPED_ERROR_STATUS_DOCS_PATHS_CHILDREN {
        assert!(
            child_tree.contains(child_path),
            "typed-error status-doc paths child tree should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error status-doc paths child {child_path} should own anchor {anchor}"
        );
    }

    super::status_sync::assert_typed_error_status_doc_paths_status_is_current();
    assert_typed_error_status_docs_are_synced();

    for (path, source) in [(TYPED_ERROR_STATUS_DOCS_PATHS_CHILD, parent)]
        .into_iter()
        .chain(super::sources::typed_error_status_doc_paths_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 status-doc paths budget; got {line_count} lines"
        );
    }
}
