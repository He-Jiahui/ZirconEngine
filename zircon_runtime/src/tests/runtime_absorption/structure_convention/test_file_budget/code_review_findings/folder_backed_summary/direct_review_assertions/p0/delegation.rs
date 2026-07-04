use super::super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_p0_direct_assertions_are_child_owner() {
    let parent = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    let child = read_runtime_src(P0_DIRECT_ASSERTIONS_CHILD);
    let child_blob = p0_direct_assertion_child_source_blob();
    let sources = super::super::super::source_inventory::code_review_findings_sources();

    assert_contains_all(
        "direct-review assertion child delegates P0 assertions to child owner",
        &parent,
        &[
            "#[path = \"direct_review_assertions/p0.rs\"]",
            "mod p0;",
            "p0::assert_p0_direct_sources_are_folder_backed",
        ],
    );
    for moved_guard in [
        concat!(
            "P0 robustness parent only mounts focused child ",
            "review guard owners"
        ),
        concat!(
            "P0 native host callback child owns F1 panic-boundary ",
            "review guard"
        ),
        concat!(
            "P0 native fixture leaf owners keep D-S8/D3/D13 fixture ",
            "review guards"
        ),
        concat!(
            "review_priority_recommendation_",
            "tracks_current_remaining_work"
        ),
    ] {
        assert!(
            !parent.contains(moved_guard),
            "P0 direct assertion `{moved_guard}` should stay in {P0_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    assert_contains_all(
        "P0 direct assertion parent owns child inventory",
        &child,
        &[
            "pub(super) fn assert_p0_direct_sources_are_folder_backed",
            P0_DIRECT_ASSERTIONS_DELEGATION_CHILD,
            P0_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD,
            P0_DIRECT_ASSERTIONS_REVIEW_CHILDREN_CHILD,
            P0_DIRECT_ASSERTIONS_BUDGETS_CHILD,
            P0_DIRECT_ASSERTIONS_STATUS_MIRRORS_CHILD,
            "runtime_15_code_review_findings_p0_direct_assertions_are_child_owner",
            P0_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
            P0_DIRECT_ASSERTIONS_STATUS_GUARD,
        ],
    );

    assert_p0_direct_sources_are_folder_backed(&sources);
    for (_, child_path, child_guard) in P0_DIRECT_ASSERTIONS_GUARD_CHILDREN {
        assert!(
            child.contains(child_path),
            "P0 direct assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "P0 direct assertions child source blob should contain child guard {child_guard}"
        );
    }
    budgets::assert_p0_direct_assertions_children_line_budgets_are_current();
}
