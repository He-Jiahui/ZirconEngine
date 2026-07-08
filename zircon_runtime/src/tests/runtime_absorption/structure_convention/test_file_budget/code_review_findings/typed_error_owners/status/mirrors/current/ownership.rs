use super::super::super::super::super::super::*;
use super::super::super::*;
use super::super::*;

pub(in super::super) fn assert_typed_error_status_mirrors_are_child_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_CHILD);
    let child_tree = super::sources::typed_error_status_mirror_child_source_blob();

    assert_contains_all(
        "typed-error status-doc status mirrors parent mounts child owners",
        &parent,
        &[
            "#[path = \"mirrors/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"mirrors/child_inventory.rs\"]",
            "mod child_inventory;",
            "#[path = \"mirrors/folder_backed_status.rs\"]",
            "mod folder_backed_status;",
            "#[path = \"mirrors/status_current.rs\"]",
            "mod status_current;",
            "pub(super) use child_inventory::*;",
            "runtime_15_typed_error_status_docs_folder_backed_status_is_current",
            TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_OWNERSHIP_GUARD,
        ],
    );
    for moved_anchor in [
        "let status_rows = typed_error_status_row_source",
        "M3 review status map records typed-error status-doc folder-backed split",
        "let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_CHILD)",
        "should stay below the Runtime 15 test-file budget",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc status mirror anchor `{moved_anchor}` should stay in child files"
        );
    }
    for (_, child_path, anchor) in TYPED_ERROR_STATUS_DOCS_STATUS_MIRROR_CHILDREN {
        assert!(
            child_tree.contains(child_path),
            "typed-error status-doc status mirror child tree should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error status-doc status mirror child {child_path} should own anchor {anchor}"
        );
    }

    super::status_sync::assert_typed_error_status_mirrors_status_is_current();

    for (path, source) in [(TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_CHILD, parent)]
        .into_iter()
        .chain(super::sources::typed_error_status_mirror_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 status-doc status-mirror budget; got {line_count} lines"
        );
    }
}
