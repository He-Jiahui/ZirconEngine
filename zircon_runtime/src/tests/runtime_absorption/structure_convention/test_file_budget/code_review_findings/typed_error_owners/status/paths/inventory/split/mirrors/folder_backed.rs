use super::super::super::super::super::super::super::super::*;
use super::super::super::super::review_guard_paths::TYPED_ERROR_CHILD_OWNER_LINE_BUDGET;
use super::super::super::super::root_paths::TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD;
use super::source_tree::{
    typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_child_paths,
    typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_child_source_blob,
};
use super::status_current::{
    assert_typed_error_status_doc_paths_child_inventory_split_layout_guard_status_is_current,
    assert_typed_error_status_doc_paths_child_inventory_split_layout_status_is_current,
    assert_typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_guard_status_is_current,
    assert_typed_error_status_doc_paths_child_inventory_status_is_current,
};

#[test]
fn runtime_15_typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_guard_is_folder_backed(
) {
    let parent = read_runtime_src(
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD,
    );
    let child_tree =
        typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_child_source_blob(
        );

    assert_contains_all(
        "typed-error status-doc paths child-inventory split-layout status-mirrors route mounts child owners",
        &parent,
        &[
            "#[path = \"mirrors/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"mirrors/source_tree.rs\"]",
            "mod source_tree;",
            "#[path = \"mirrors/status_current.rs\"]",
            "mod status_current;",
            "#[path = \"mirrors/status_documents.rs\"]",
            "mod status_documents;",
            "#[path = \"mirrors/status_maps.rs\"]",
            "mod status_maps;",
        ],
    );

    for moved_anchor in [
        "fn assert_typed_error_status_doc_paths_child_inventory_status_is_current",
        "fn assert_typed_error_status_doc_paths_child_inventory_split_layout_status_is_current",
        "fn assert_typed_error_status_doc_paths_child_inventory_split_layout_guard_status_is_current",
        "fn assert_status_documents_contain",
        "REVIEW_GUARD_STATUS_MAP_PATH",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error paths child-inventory split-layout status-mirrors route should not own moved anchor {moved_anchor}"
        );
        assert!(
            child_tree.contains(moved_anchor),
            "typed-error paths child-inventory split-layout status-mirrors child tree should own moved anchor {moved_anchor}"
        );
    }

    assert_typed_error_status_doc_paths_child_inventory_status_is_current();
    assert_typed_error_status_doc_paths_child_inventory_split_layout_status_is_current();
    assert_typed_error_status_doc_paths_child_inventory_split_layout_guard_status_is_current();
    assert_typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_guard_status_is_current();

    assert_line_budget(
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD,
    );
    for path in
        typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_child_paths()
    {
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
