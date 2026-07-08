use super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_parent_delegates_status_docs() {
    let typed_error_parent = read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD);

    assert_contains_all(
        "typed-error structure owner delegates status-doc sync to child owner",
        &typed_error_parent,
        &[
            "#[path = \"typed_error_owners/status_docs.rs\"]",
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
}
