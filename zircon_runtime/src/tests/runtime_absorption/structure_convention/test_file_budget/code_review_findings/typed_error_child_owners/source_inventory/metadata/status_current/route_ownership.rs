use super::super::super::super::super::super::*;
use super::super::*;
use super::*;

pub(super) fn assert_typed_error_source_inventory_metadata_status_current_is_child_backed() {
    let metadata_parent = read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_METADATA_CHILD);
    let metadata_child_blob = source_blobs::metadata_child_source_blob();
    let status_current_parent =
        read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_CHILD);
    let status_current_children = source_blobs::metadata_status_current_child_source_blob();

    assert_contains_all(
        "typed-error source inventory metadata parent mounts focused children",
        &metadata_parent,
        &[
            "#[path = \"metadata/child_inventory_paths.rs\"]",
            "#[path = \"metadata/delegation_paths.rs\"]",
            "#[path = \"metadata/review_guard_paths.rs\"]",
            "#[path = \"metadata/root_paths.rs\"]",
            "#[path = \"metadata/status_current.rs\"]",
            "#[path = \"metadata/status_slices.rs\"]",
            "pub(super) use child_inventory_paths::*;",
            "pub(super) use delegation_paths::*;",
            "pub(super) use review_guard_paths::*;",
            "pub(super) use root_paths::*;",
            "pub(super) use status_slices::*;",
            "TYPED_ERROR_SOURCE_INVENTORY_METADATA_CHILDREN",
            "runtime_15_typed_error_source_inventory_metadata_is_child_backed",
        ],
    );
    for moved_anchor in [
        "pub(super) const TYPED_ERROR_STRUCTURE_CHILD",
        "pub(super) const TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_STATUS_CURRENT_ROUTE_CHILD",
        "pub(super) const TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_CHILD_BACKED_GUARD",
        "pub(super) const TYPED_ERROR_CHILD_OWNER_LINE_BUDGET",
        "pub(super) const REVIEW_GUARD_STATUS_ROWS_PATH",
    ] {
        assert!(
            !metadata_parent.contains(moved_anchor),
            "source_inventory/metadata.rs should delegate `{moved_anchor}` to focused children"
        );
    }
    for (label, child_path, anchor) in TYPED_ERROR_SOURCE_INVENTORY_METADATA_CHILDREN {
        assert!(
            metadata_parent.contains(label),
            "typed-error source inventory metadata parent should mount child label {label}"
        );
        assert!(
            metadata_child_blob.contains(child_path),
            "typed-error source inventory metadata child tree should inventory path {child_path}"
        );
        assert!(
            metadata_child_blob.contains(anchor),
            "typed-error source inventory metadata child {child_path} should own anchor {anchor}"
        );
    }
    assert_contains_all(
        "typed-error source inventory metadata status-current parent mounts focused children",
        &status_current_parent,
        &[
            "#[path = \"status_current/budgets.rs\"]",
            "#[path = \"status_current/route_ownership.rs\"]",
            "#[path = \"status_current/source_blobs.rs\"]",
            "#[path = \"status_current/status_mirrors.rs\"]",
            "TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_CHILDREN",
            "route_ownership::assert_typed_error_source_inventory_metadata_status_current_is_child_backed",
            "status_mirrors::assert_typed_error_source_inventory_metadata_status_current_mirrors_are_current",
            "budgets::assert_typed_error_source_inventory_metadata_status_current_child_budgets_are_current",
            "runtime_15_typed_error_source_inventory_metadata_status_current_is_child_backed",
        ],
    );
    for moved_anchor in [
        "let status_rows = read_runtime_src(",
        "TYPED_ERROR_SOURCE_INVENTORY_METADATA_SPLIT",
        "TYPED_ERROR_CHILD_OWNER_LINE_BUDGET",
        "fn metadata_child_source_blob",
        "fn source_blob_from",
    ] {
        assert!(
            !status_current_parent.contains(moved_anchor),
            "metadata/status_current.rs should delegate `{moved_anchor}` to focused children"
        );
    }
    for (_, child_path, anchor) in TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_CHILDREN {
        assert!(
            status_current_children.contains(child_path),
            "typed-error source inventory metadata status-current tree should inventory path {child_path}"
        );
        assert!(
            status_current_children.contains(anchor),
            "typed-error source inventory metadata status-current child {child_path} should own anchor {anchor}"
        );
    }
}
