use super::super::super::super::super::super::*;
use super::super::super::*;
use super::*;

pub(in super::super) fn assert_typed_error_source_inventory_child_sources_structure_guard_is_child_backed(
) {
    let parent = read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_CHILD);
    let structure_guard_parent =
        read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_CHILD);
    let structure_guard_children_blob = super::super::source_blobs::source_blob_from(
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_CHILDREN
            .iter()
            .map(|(_, path, _)| (*path, read_runtime_src(path)))
            .collect(),
    );

    assert_contains_all(
        "typed-error source inventory child_sources structure guard mounts focused children",
        &structure_guard_parent,
        &[
            "#[path = \"structure_guard/budgets.rs\"]",
            "#[path = \"structure_guard/route_ownership.rs\"]",
            "#[path = \"structure_guard/status_mirrors.rs\"]",
            "TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_CHILDREN",
            "route_ownership::assert_typed_error_source_inventory_child_sources_structure_guard_is_child_backed",
            "budgets::assert_typed_error_source_inventory_child_sources_structure_guard_budgets_are_current",
        ],
    );
    for moved_anchor in [
        "let status_rows = read_runtime_src(",
        "let status_map = format!",
        "let date_map = format!",
        "TYPED_ERROR_CHILD_OWNER_LINE_BUDGET",
        "for (path, source) in [(",
    ] {
        assert!(
            !structure_guard_parent.contains(moved_anchor),
            "child_sources/structure_guard.rs should delegate `{moved_anchor}` to focused children"
        );
    }
    for (_, child_path, anchor) in
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_CHILDREN
    {
        assert!(
            structure_guard_children_blob.contains(child_path),
            "typed-error source inventory child_sources structure guard should inventory path {child_path}"
        );
        assert!(
            structure_guard_children_blob.contains(anchor),
            "typed-error source inventory child_sources structure guard child {child_path} should own anchor {anchor}"
        );
    }
}
