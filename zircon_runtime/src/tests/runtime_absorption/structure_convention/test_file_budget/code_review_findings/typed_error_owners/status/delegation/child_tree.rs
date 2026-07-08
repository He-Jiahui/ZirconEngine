use super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_status_doc_children_own_delegated_assertions() {
    let status_docs_child_tree = typed_error_status_docs_child_source_blob();

    assert_contains_all(
        "typed-error status-doc children own delegated assertions",
        &status_docs_child_tree,
        &[
            "runtime_15_typed_error_status_docs_are_folder_backed",
            "assert_typed_error_status_doc_mirrors_are_synced",
            "assert_typed_error_status_maps_are_synced",
            "runtime_15_typed_error_status_docs_folder_backed_status_is_current",
        ],
    );
    for (_, child_path, anchor) in TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN {
        assert!(
            status_docs_child_tree.contains(child_path),
            "typed-error status-doc child tree should inventory child path {child_path}"
        );
        assert!(
            status_docs_child_tree.contains(anchor),
            "typed-error status-doc child {child_path} should own anchor {anchor}"
        );
    }
    for (_, child_path, anchor) in TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_CHILDREN {
        assert!(
            status_docs_child_tree.contains(child_path),
            "typed-error status-doc source helper tree should inventory child path {child_path}"
        );
        assert!(
            status_docs_child_tree.contains(anchor),
            "typed-error status-doc source helper child {child_path} should own anchor {anchor}"
        );
    }
}
