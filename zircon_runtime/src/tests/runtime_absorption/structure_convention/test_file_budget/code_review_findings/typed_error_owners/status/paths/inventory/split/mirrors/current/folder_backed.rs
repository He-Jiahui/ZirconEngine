use super::super::super::super::super::super::super::super::super::*;
use super::super::super::super::super::review_guard_paths::TYPED_ERROR_CHILD_OWNER_LINE_BUDGET;
use super::source_tree::{
    typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_status_current_child_paths,
    typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_status_current_child_source_blob,
};
use super::{
    assert_typed_error_status_doc_paths_child_inventory_split_layout_guard_status_is_current,
    assert_typed_error_status_doc_paths_child_inventory_split_layout_status_is_current,
    assert_typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_guard_status_is_current,
    assert_typed_error_status_doc_paths_child_inventory_status_is_current,
};

const STATUS_CURRENT_ROUTE: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/inventory/split/mirrors/status_current.rs";

#[test]
fn runtime_15_typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_status_current_is_folder_backed(
) {
    let parent = read_runtime_src(STATUS_CURRENT_ROUTE);
    let child_tree =
        typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_status_current_child_source_blob();

    assert_contains_all(
        "typed-error paths child-inventory split-layout status-mirrors status-current route mounts child owners",
        &parent,
        &[
            "#[path = \"current/current.rs\"]",
            "mod current;",
            "#[path = \"current/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"current/source_tree.rs\"]",
            "mod source_tree;",
            "#[path = \"current/split_layout.rs\"]",
            "mod split_layout;",
            "#[path = \"current/split_layout_guard.rs\"]",
            "mod split_layout_guard;",
            "#[path = \"current/status_mirrors_guard.rs\"]",
            "mod status_mirrors_guard;",
            "#[path = \"current/support.rs\"]",
            "mod support;",
        ],
    );

    for moved_anchor in [
        "let anchors = [",
        "fn assert_documents_and_maps",
        "TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_GUARD_NAME",
        "TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_STATUS_MIRRORS_GUARD_NAME",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error paths child-inventory split-layout status-current route should not own moved anchor {moved_anchor}"
        );
        assert!(
            child_tree.contains(moved_anchor),
            "typed-error paths child-inventory split-layout status-current child tree should own moved anchor {moved_anchor}"
        );
    }

    assert_typed_error_status_doc_paths_child_inventory_status_is_current();
    assert_typed_error_status_doc_paths_child_inventory_split_layout_status_is_current();
    assert_typed_error_status_doc_paths_child_inventory_split_layout_guard_status_is_current();
    assert_typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_guard_status_is_current();

    assert_line_budget(STATUS_CURRENT_ROUTE);
    for path in typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_status_current_child_paths() {
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
