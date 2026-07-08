use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_status_docs_status_anchor_guard_is_folder_backed() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/status_anchor_guard.rs",
    );
    let child_blob = status_anchor_guard_child_source_blob();

    for (_, child_path, child_guard) in STATUS_DOC_STATUS_ANCHOR_GUARD_CHILDREN {
        assert!(
            parent.contains(child_path),
            "status-anchor guard parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "status-anchor guard child source blob should contain child guard {child_guard}"
        );
    }
    for moved_anchor in [
        "fn runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner",
        "fn runtime_15_code_review_findings_status_docs_status_anchors_are_folder_backed",
        "let runtime_15_plan =",
        "status-anchor children own status-doc slice/status/owner/guard anchors",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "status_anchor_guard.rs should delegate implementation anchor `{moved_anchor}` to children"
        );
        assert!(
            child_blob.contains(moved_anchor),
            "status-anchor guard children should own implementation anchor `{moved_anchor}`"
        );
    }
    assert_contains_all(
        "status-anchor guard parent records folder-backed status",
        &parent,
        &[
            STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_SLICE,
            STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_STATUS,
            STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_GUARD,
            STATUS_DOC_STATUS_ANCHOR_GUARD_STATUS_GUARD,
            STATUS_DOC_STATUS_ANCHOR_GUARD_BUDGET_GUARD,
        ],
    );
    budgets::assert_status_anchor_guard_children_line_budgets_are_current();
}
