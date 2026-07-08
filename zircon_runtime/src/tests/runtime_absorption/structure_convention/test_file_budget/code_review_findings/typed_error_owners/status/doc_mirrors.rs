use super::super::super::super::*;
use super::*;

#[path = "doc_mirrors/child_inventory.rs"]
mod child_inventory;
#[path = "doc_mirrors/guard_anchors.rs"]
mod guard_anchors;
#[path = "doc_mirrors/metadata.rs"]
mod metadata;
#[path = "doc_mirrors/source_helper_ownership.rs"]
mod source_helper_ownership;
#[path = "doc_mirrors/source_helper_status.rs"]
mod source_helper_status;
#[path = "doc_mirrors/source_paths.rs"]
mod source_paths;
#[path = "doc_mirrors/sources.rs"]
mod sources;
#[path = "doc_mirrors/status_current.rs"]
mod status_current;
#[path = "doc_mirrors/status_slices.rs"]
mod status_slices;

pub(super) use child_inventory::*;
pub(super) use metadata::*;
pub(super) use sources::*;

pub(super) fn assert_typed_error_status_doc_mirrors_are_synced(
    sources: &TypedErrorStatusDocSources,
) {
    status_slices::assert_typed_error_status_doc_slice_anchors_are_synced(sources);
    source_paths::assert_typed_error_status_doc_source_paths_are_synced(sources);
    guard_anchors::assert_typed_error_status_doc_guard_anchors_are_synced(sources);
}

#[test]
fn runtime_15_typed_error_status_doc_mirrors_are_folder_backed() {
    let sources = typed_error_status_doc_sources();
    let child_tree = typed_error_status_doc_mirror_child_source_blob();

    for (_, child_path, anchor) in TYPED_ERROR_STATUS_DOC_MIRROR_CHILDREN {
        assert!(
            child_tree.contains(child_path),
            "typed-error status-doc mirror child tree should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error status-doc mirror child {child_path} should own anchor {anchor}"
        );
    }
    assert_typed_error_status_doc_mirrors_are_synced(&sources);
}
