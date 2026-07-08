use super::super::super::super::super::super::super::*;
use super::super::super::review_guard_paths::TYPED_ERROR_CHILD_OWNER_LINE_BUDGET;
use super::super::super::TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_SLICES_CHILD;
use super::source_tree::{
    typed_error_status_doc_status_slices_child_source_paths,
    typed_error_status_doc_status_slices_folder_backed_guard_blob,
    typed_error_status_doc_status_slices_folder_backed_guard_children,
    typed_error_status_doc_status_slices_source_blob,
    TYPED_ERROR_STATUS_DOCS_STATUS_SLICES_FOLDER_BACKED_CHILD,
};
use super::status_current::{
    assert_status_slices_folder_backed_guard_status_is_current,
    assert_status_slices_folder_backed_status_is_current,
};

#[test]
fn runtime_15_typed_error_status_doc_status_slices_are_folder_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_SLICES_CHILD);
    let child_tree = typed_error_status_doc_status_slices_source_blob();

    assert_contains_all(
        "typed-error status-doc status slices route mounts child owners",
        &parent,
        &[
            "#[path = \"status_slices/core.rs\"]",
            "mod core;",
            "#[path = \"status_slices/paths.rs\"]",
            "mod paths;",
            "#[path = \"status_slices/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"status_slices/status_maps.rs\"]",
            "mod status_maps;",
            "#[path = \"status_slices/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "pub(in super::super) use core::*;",
            "pub(in super::super) use paths::*;",
            "pub(in super::super) use delegation::*;",
            "pub(in super::super) use status_maps::*;",
            "pub(in super::super) use status_mirrors::*;",
        ],
    );

    for moved_anchor in [
        "TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_GUARD_GUARD",
        "TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_GUARD",
        "TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_SPLIT_LAYOUT_GUARD",
        "TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_SPLIT_LAYOUT_GUARD",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc status slices route should not own moved anchor {moved_anchor}"
        );
        assert!(
            child_tree.contains(moved_anchor),
            "typed-error status-doc status slices child tree should own moved anchor {moved_anchor}"
        );
    }

    for path in typed_error_status_doc_status_slices_child_source_paths() {
        assert_line_budget(path);
    }
    assert_line_budget(TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_SLICES_CHILD);

    assert_status_slices_folder_backed_status_is_current();
}

#[test]
fn runtime_15_typed_error_status_doc_status_slices_folder_backed_guard_is_folder_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_STATUS_SLICES_FOLDER_BACKED_CHILD);
    let child_tree = typed_error_status_doc_status_slices_folder_backed_guard_blob();

    assert_contains_all(
        "typed-error status-doc status-slices folder-backed guard route mounts child owners",
        &parent,
        &[
            "#[path = \"folder_backed/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"folder_backed/source_tree.rs\"]",
            "mod source_tree;",
            "#[path = \"folder_backed/status_current.rs\"]",
            "mod status_current;",
        ],
    );

    for moved_anchor in [
        "fn runtime_15_typed_error_status_doc_status_slices_are_folder_backed",
        "fn typed_error_status_doc_status_slices_source_blob",
        "fn assert_status_slices_folder_backed_status_is_current",
        "TYPED_ERROR_STATUS_DOCS_STATUS_SLICE_CHILDREN",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-slices folder-backed route should not own moved anchor {moved_anchor}"
        );
        assert!(
            child_tree.contains(moved_anchor),
            "typed-error status-slices folder-backed child tree should own moved anchor {moved_anchor}"
        );
    }

    assert_status_slices_folder_backed_status_is_current();
    assert_status_slices_folder_backed_guard_status_is_current();

    assert_line_budget(TYPED_ERROR_STATUS_DOCS_STATUS_SLICES_FOLDER_BACKED_CHILD);
    for path in typed_error_status_doc_status_slices_folder_backed_guard_children() {
        assert_line_budget(path);
    }
}

fn assert_line_budget(path: &str) {
    let line_count = read_runtime_src(path).lines().count();
    assert!(
        line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
        "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
    );
}
