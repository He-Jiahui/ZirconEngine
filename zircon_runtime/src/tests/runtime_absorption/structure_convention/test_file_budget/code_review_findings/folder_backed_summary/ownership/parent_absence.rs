use super::super::super::super::*;
use super::*;

pub(super) fn assert_folder_backed_summary_parent_keeps_child_details_out(parent: &str) {
    for direct_review_guard in [
        concat!(
            "P0 robustness parent only mounts focused child ",
            "review guard owners"
        ),
        concat!(
            "F8 API convergence parent only mounts focused child ",
            "review guard owners"
        ),
        concat!(
            "render structure child owns F16 render_compiled_scene ",
            "review guard"
        ),
        concat!(
            "F12 dead-code child owns production suppression ",
            "review guard"
        ),
    ] {
        assert!(
            !parent.contains(direct_review_guard),
            "direct review guard `{direct_review_guard}` should stay in {DIRECT_REVIEW_ASSERTIONS_CHILD}"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_folder_backed_summary_child_ownership_guard_is_folder_backed() {
    let child_ownership_parent = read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_CHILD);
    let child_blob = folder_backed_summary_child_ownership_child_source_blob();

    assert_folder_backed_summary_parent_keeps_child_details_out(&read_runtime_src(
        FOLDER_BACKED_SUMMARY_CHILD,
    ));
    direct_assertions::assert_folder_backed_direct_review_assertion_children_are_current();
    source_inventory_checks::assert_folder_backed_source_inventory_child_is_current(
        &read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD),
    );
    budgets::assert_folder_backed_summary_child_ownership_children_line_budgets_are_current();
    for (_, child_path, child_guard) in FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_GUARD_CHILDREN {
        assert!(
            child_ownership_parent.contains(child_path),
            "folder-backed summary child-ownership parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "folder-backed summary child-ownership child source blob should contain child guard {child_guard}"
        );
    }
    assert!(
        !child_ownership_parent.contains("#[test]"),
        "folder_backed_summary/child_ownership.rs should delegate test bodies to focused children"
    );
    assert_contains_all(
        "folder-backed summary child-ownership parent records folder-backed status",
        &child_ownership_parent,
        &[
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_SLICE,
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_STATUS,
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_GUARD,
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_STATUS_GUARD,
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_BUDGET_GUARD,
        ],
    );
}
