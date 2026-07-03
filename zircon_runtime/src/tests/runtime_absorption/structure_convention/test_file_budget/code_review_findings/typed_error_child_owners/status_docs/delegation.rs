use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_status_docs_are_folder_backed() {
    let typed_error_parent = read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD);
    let status_docs_parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_CHILD);
    let status_docs_child_tree = typed_error_status_docs_child_source_blob();
    let sources = typed_error_status_doc_sources();

    assert_contains_all(
        "typed-error structure owner delegates status-doc sync to child owner",
        &typed_error_parent,
        &[
            "#[path = \"typed_error_child_owners/status_docs.rs\"]",
            "mod status_docs;",
            "status_docs::assert_typed_error_status_docs_are_synced",
        ],
    );
    for moved_anchor in [
        "let runtime_15_plan =",
        "let status_rows = format!(",
        "Runtime 15 M3 native live-host typed-error review guard child-owner split",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders/texture.rs",
    ] {
        assert!(
            !typed_error_parent.contains(moved_anchor),
            "typed-error status-doc anchor `{moved_anchor}` should stay in {TYPED_ERROR_STATUS_DOCS_CHILD}"
        );
    }
    assert_contains_all(
        "typed-error status-doc parent delegates focused guard children",
        &status_docs_parent,
        &[
            "#[path = \"status_docs/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"status_docs/doc_mirrors.rs\"]",
            "mod doc_mirrors;",
            "#[path = \"status_docs/status_maps.rs\"]",
            "mod status_maps;",
            "#[path = \"status_docs/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "pub(super) fn typed_error_status_doc_sources",
            "pub(super) fn assert_typed_error_status_docs_are_synced",
            "doc_mirrors::assert_typed_error_status_doc_mirrors_are_synced",
            "status_maps::assert_typed_error_status_maps_are_synced",
            "typed_error_status_docs_child_sources",
            "typed_error_status_docs_child_source_blob",
        ],
    );
    assert_contains_all(
        "typed-error status-doc children own delegated assertions",
        &status_docs_child_tree,
        &[
            "runtime_15_typed_error_status_docs_are_folder_backed",
            "assert_typed_error_status_doc_mirrors_are_synced",
            "assert_typed_error_status_maps_are_synced",
            "runtime_15_typed_error_status_docs_folder_backed_status_is_current",
        ],
    );
    for (_, child_path, anchor) in TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN {
        assert!(
            status_docs_parent.contains(child_path),
            "typed-error status-doc parent should inventory child path {child_path}"
        );
        assert!(
            status_docs_child_tree.contains(anchor),
            "typed-error status-doc child {child_path} should own anchor {anchor}"
        );
    }

    assert_typed_error_status_docs_are_synced();

    for (path, source) in [
        (TYPED_ERROR_STRUCTURE_CHILD, typed_error_parent),
        (TYPED_ERROR_STATUS_DOCS_CHILD, status_docs_parent),
        ("typed-error status row data", sources.status_rows),
    ]
    .into_iter()
    .chain(typed_error_status_docs_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
