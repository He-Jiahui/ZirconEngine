use super::super::super::super::super::*;
use super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_status_doc_mirrors_source_helpers_are_child_backed() {
    let sources = typed_error_status_doc_sources();
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOC_MIRRORS_CHILD);
    let child_tree = typed_error_status_doc_mirror_source_helper_child_source_blob();

    assert_contains_all(
        "typed-error status-doc mirror parent mounts source helper children",
        &parent,
        &[
            "#[path = \"doc_mirrors/child_inventory.rs\"]",
            "mod child_inventory;",
            "#[path = \"doc_mirrors/metadata.rs\"]",
            "mod metadata;",
            "#[path = \"doc_mirrors/source_helper_ownership.rs\"]",
            "mod source_helper_ownership;",
            "#[path = \"doc_mirrors/source_helper_status.rs\"]",
            "mod source_helper_status;",
            "#[path = \"doc_mirrors/sources.rs\"]",
            "mod sources;",
            "pub(super) use child_inventory::*;",
            "pub(super) use metadata::*;",
            "pub(super) use sources::*;",
            "runtime_15_typed_error_status_doc_mirrors_are_folder_backed",
        ],
    );
    for moved_anchor in [
        "pub(super) const TYPED_ERROR_STATUS_DOC_MIRRORS_CHILD",
        "pub(super) const TYPED_ERROR_STATUS_DOC_MIRRORS_FOLDER_BACKED_SLICE",
        "pub(super) const TYPED_ERROR_STATUS_DOC_MIRROR_CHILDREN",
        "pub(super) fn typed_error_status_doc_mirror_sources",
        "pub(super) fn typed_error_status_doc_mirror_child_sources",
        "pub(super) fn typed_error_status_doc_mirror_child_source_blob",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc mirror source helper `{moved_anchor}` should stay in child files"
        );
    }
    for (_, child_path, anchor) in TYPED_ERROR_STATUS_DOC_MIRROR_SOURCE_HELPER_CHILDREN {
        assert!(
            child_tree.contains(child_path),
            "typed-error status-doc mirror source helper tree should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error status-doc mirror source helper child {child_path} should own anchor {anchor}"
        );
    }

    assert_typed_error_status_doc_mirrors_are_synced(&sources);

    for (path, source) in [(TYPED_ERROR_STATUS_DOC_MIRRORS_CHILD, parent)]
        .into_iter()
        .chain(typed_error_status_doc_mirror_source_helper_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 status-doc mirror source helper budget; got {line_count} lines"
        );
    }
}
