use super::super::super::super::super::super::*;
use super::super::super::*;

pub(super) fn assert_typed_error_source_inventory_guard_is_folder_backed() {
    let sources = typed_error_source_inventory_sources();
    let child_blob = typed_error_source_inventory_child_source_blob();

    super::super::parent_delegation::assert_typed_error_structure_delegates_source_inventory(
        &sources,
    );
    super::super::source_inventory_mounts::assert_typed_error_source_inventory_parent_mounts_focused_owners(
        &sources,
    );
    super::super::source_ownership::assert_typed_error_source_inventory_paths_and_reads_are_child_owned(
        &sources,
    );
    for (_, child_path, child_guard) in TYPED_ERROR_SOURCE_INVENTORY_CHILDREN {
        assert!(
            child_blob.contains(child_path),
            "typed-error source inventory child tree should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "typed-error source inventory child source blob should contain child guard {child_guard}"
        );
    }
    assert!(
        !sources
            .source_inventory_child
            .contains("const TYPED_ERROR_SOURCE_PATHS:"),
        "source_inventory.rs should delegate source path literals to paths.rs"
    );
    assert!(
        !sources
            .source_inventory_child
            .contains("fn typed_error_sources()"),
        "source_inventory.rs should delegate source reads to reads.rs"
    );
}
