use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_status_doc_source_helpers_are_child_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_CHILD);
    let child_tree = typed_error_status_doc_source_helper_child_source_blob();

    assert_contains_all(
        "typed-error status-doc parent mounts source helper children",
        &parent,
        &[
            "#[path = \"status/child_sources.rs\"]",
            "mod child_sources;",
            "#[path = \"status/paths.rs\"]",
            "mod paths;",
            "#[path = \"status/source_helper_ownership.rs\"]",
            "mod source_helper_ownership;",
            "#[path = \"status/source_helper_status.rs\"]",
            "mod source_helper_status;",
            "#[path = \"status/sources.rs\"]",
            "mod sources;",
            "pub(super) use child_sources::*;",
            "pub(super) use paths::*;",
            "pub(super) use sources::*;",
        ],
    );
    for moved_anchor in [
        "pub(super) struct TypedErrorStatusDocSources",
        "pub(super) fn typed_error_status_doc_sources",
        "pub(super) fn typed_error_status_row_source",
        "pub(super) fn typed_error_status_docs_child_sources",
        "pub(super) fn typed_error_status_docs_child_source_blob",
        "TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN",
        "REVIEW_GUARD_STATUS_ROW_CHILD_PATHS",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc source helper `{moved_anchor}` should stay in child files"
        );
    }
    for (_, child_path, anchor) in TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_CHILDREN {
        assert!(
            child_tree.contains(child_path),
            "typed-error status-doc source helper tree should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error status-doc source helper child {child_path} should own anchor {anchor}"
        );
    }

    assert_typed_error_status_docs_are_synced();

    for (path, source) in [(TYPED_ERROR_STATUS_DOCS_CHILD, parent)]
        .into_iter()
        .chain(typed_error_status_doc_source_helper_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
