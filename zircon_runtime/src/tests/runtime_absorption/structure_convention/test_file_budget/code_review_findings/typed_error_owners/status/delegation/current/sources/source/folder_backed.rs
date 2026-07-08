use super::super::super::super::super::super::super::super::*;
use super::super::super::super::super::{
    TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SOURCES_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SOURCES_DELEGATION_CHILDREN_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SOURCES_SOURCE_SPLIT_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SOURCES_STATUS_CURRENT_CHILDREN_CHILD,
};
use super::source_tree::{
    typed_error_delegation_status_current_sources_guard_blob,
    typed_error_delegation_status_current_sources_guard_children,
};
use super::status_current::{
    assert_sources_child_split_status_is_current,
    assert_sources_guard_folder_backed_status_is_current,
};

#[test]
fn runtime_15_typed_error_status_doc_delegation_status_current_sources_guard_is_folder_backed() {
    let parent = read_runtime_src(
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SOURCES_SOURCE_SPLIT_CHILD,
    );
    let child_tree = typed_error_delegation_status_current_sources_guard_blob();

    assert_contains_all(
        "typed-error delegation status-current source-split route mounts child owners",
        &parent,
        &[
            "#[path = \"source/source_tree.rs\"]",
            "mod source_tree;",
            "#[path = \"source/status_current.rs\"]",
            "mod status_current;",
            "#[path = \"source/folder_backed.rs\"]",
            "mod folder_backed;",
        ],
    );

    for moved_anchor in [
        "fn runtime_15_typed_error_status_doc_delegation_status_current_sources_are_child_backed",
        "fn assert_sources_child_split_status_is_current",
        "TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_CHILDREN",
        "typed_error_status_doc_delegation_status_current_child_source_blob",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error delegation status-current source-split route should not own moved anchor {moved_anchor}"
        );
        assert!(
            child_tree.contains(moved_anchor),
            "typed-error delegation status-current source-split child tree should own moved anchor {moved_anchor}"
        );
    }

    assert_sources_child_split_status_is_current();
    assert_sources_guard_folder_backed_status_is_current();

    for path in [
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SOURCES_CHILD,
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SOURCES_DELEGATION_CHILDREN_CHILD,
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SOURCES_SOURCE_SPLIT_CHILD,
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SOURCES_STATUS_CURRENT_CHILDREN_CHILD,
    ] {
        assert_line_budget(path);
    }
    for path in typed_error_delegation_status_current_sources_guard_children() {
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
