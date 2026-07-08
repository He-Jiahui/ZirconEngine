use super::super::super::super::super::super::*;
use super::super::super::*;
use super::super::*;

pub(in super::super) fn assert_typed_error_status_doc_delegation_is_child_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_DELEGATION_CHILD);
    let child_tree = super::sources::typed_error_status_doc_delegation_child_source_blob();

    assert_contains_all(
        "typed-error status-doc delegation parent mounts child owners",
        &parent,
        &[
            "#[path = \"delegation/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"delegation/child_inventory.rs\"]",
            "mod child_inventory;",
            "#[path = \"delegation/child_tree.rs\"]",
            "mod child_tree;",
            "#[path = \"delegation/status_current.rs\"]",
            "mod status_current;",
            "#[path = \"delegation/status_doc_parent.rs\"]",
            "mod status_doc_parent;",
            "#[path = \"delegation/typed_error_parent.rs\"]",
            "mod typed_error_parent;",
            "pub(super) use child_inventory::*;",
            "runtime_15_typed_error_status_docs_are_folder_backed",
            TYPED_ERROR_STATUS_DOCS_DELEGATION_OWNERSHIP_GUARD,
        ],
    );
    for moved_anchor in [
        "let typed_error_parent = read_runtime_src",
        "let status_docs_parent = read_runtime_src",
        "let status_docs_child_tree = typed_error_status_docs_child_source_blob",
        "typed-error status-doc parent delegates focused guard children",
        "typed-error status-doc children own delegated assertions",
        "typed-error status row data",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc delegation `{moved_anchor}` should stay in child files"
        );
    }
    for (_, child_path, anchor) in TYPED_ERROR_STATUS_DOCS_DELEGATION_CHILDREN {
        assert!(
            child_tree.contains(child_path),
            "typed-error status-doc delegation child tree should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error status-doc delegation child {child_path} should own anchor {anchor}"
        );
    }

    super::status_sync::assert_typed_error_status_doc_delegation_status_is_current();

    for (path, source) in [(TYPED_ERROR_STATUS_DOCS_DELEGATION_CHILD, parent)]
        .into_iter()
        .chain(super::sources::typed_error_status_doc_delegation_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 status-doc delegation budget; got {line_count} lines"
        );
    }
}
