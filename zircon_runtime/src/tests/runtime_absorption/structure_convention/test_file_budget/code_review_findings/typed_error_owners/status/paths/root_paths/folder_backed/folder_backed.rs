use super::super::super::super::super::super::super::*;
use super::super::super::review_guard_paths::TYPED_ERROR_CHILD_OWNER_LINE_BUDGET;
use super::super::path_children::TYPED_ERROR_STATUS_DOCS_PATHS_ROOT_PATHS_CHILD;
use super::super::source_tree::{
    typed_error_status_doc_root_paths_child_source_paths,
    typed_error_status_doc_root_paths_source_blob,
};
use super::source_tree::{
    typed_error_status_doc_root_paths_folder_backed_guard_blob,
    typed_error_status_doc_root_paths_folder_backed_guard_children,
    TYPED_ERROR_STATUS_DOCS_ROOT_PATHS_FOLDER_BACKED_CHILD,
};
use super::status_current::{
    assert_root_paths_folder_backed_guard_status_is_current,
    assert_root_paths_folder_backed_status_is_current,
};

#[test]
fn runtime_15_typed_error_status_doc_root_paths_are_folder_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_PATHS_ROOT_PATHS_CHILD);
    let child_tree = typed_error_status_doc_root_paths_source_blob();

    assert_contains_all(
        "typed-error status-doc root paths route mounts child owners",
        &parent,
        &[
            "#[path = \"root_paths/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"root_paths/path_children.rs\"]",
            "mod path_children;",
            "#[path = \"root_paths/source_tree.rs\"]",
            "mod source_tree;",
            "#[path = \"root_paths/status_doc_core.rs\"]",
            "mod status_doc_core;",
            "#[path = \"root_paths/status_maps.rs\"]",
            "mod status_maps;",
            "#[path = \"root_paths/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "pub(in super::super) use delegation::*;",
            "pub(in super::super) use path_children::*;",
            "pub(in super::super) use status_maps::*;",
            "pub(in super::super) use status_mirrors::*;",
        ],
    );

    for moved_anchor in [
        "TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD",
        "TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD",
        "TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_STATUS_CURRENT_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD",
        "TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc root paths route should not own moved anchor {moved_anchor}"
        );
        assert!(
            child_tree.contains(moved_anchor),
            "typed-error status-doc root paths child tree should own moved anchor {moved_anchor}"
        );
    }

    for path in typed_error_status_doc_root_paths_child_source_paths() {
        assert_line_budget(path);
    }
    assert_line_budget(TYPED_ERROR_STATUS_DOCS_PATHS_ROOT_PATHS_CHILD);

    assert_root_paths_folder_backed_status_is_current();
}

#[test]
fn runtime_15_typed_error_status_doc_root_paths_folder_backed_guard_is_folder_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_ROOT_PATHS_FOLDER_BACKED_CHILD);
    let child_tree = typed_error_status_doc_root_paths_folder_backed_guard_blob();

    assert_contains_all(
        "typed-error status-doc root paths folder-backed guard route mounts child owners",
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
        "fn runtime_15_typed_error_status_doc_root_paths_are_folder_backed",
        "fn assert_root_paths_folder_backed_status_is_current",
        "typed_error_status_doc_root_paths_source_blob",
        "TYPED_ERROR_STATUS_DOC_PATHS_STATUS_MAP_PATH",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error root paths folder-backed route should not own moved anchor {moved_anchor}"
        );
        assert!(
            child_tree.contains(moved_anchor),
            "typed-error root paths folder-backed child tree should own moved anchor {moved_anchor}"
        );
    }

    assert_root_paths_folder_backed_status_is_current();
    assert_root_paths_folder_backed_guard_status_is_current();

    assert_line_budget(TYPED_ERROR_STATUS_DOCS_ROOT_PATHS_FOLDER_BACKED_CHILD);
    for path in typed_error_status_doc_root_paths_folder_backed_guard_children() {
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
