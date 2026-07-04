use super::super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_f12_direct_assertions_are_child_owner() {
    let parent = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    let child = read_runtime_src(F12_DIRECT_ASSERTIONS_CHILD);
    let child_blob = f12_direct_assertion_child_source_blob();
    let sources = super::super::super::source_inventory::code_review_findings_sources();

    assert_contains_all(
        "direct-review assertion child delegates F12 assertions to child owner",
        &parent,
        &[
            "#[path = \"direct_review_assertions/f12.rs\"]",
            "mod f12;",
            "f12::assert_f12_direct_sources_are_folder_backed",
        ],
    );
    for moved_guard in [
        concat!(
            "F12 dead-code child owns production suppression ",
            "review guard"
        ),
        "review_f12_runtime_production_dead_code_suppression_is_globally_gated",
        "runtime_15_f12_dead_code_review_status_sync_static_passed_cargo_deferred",
        "runtime_15_f12_dead_code_runtime_editor_boundary_status_static_passed_cargo_deferred",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "F12 direct assertion `{moved_guard}` should stay in {F12_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    assert_contains_all(
        "F12 direct assertion parent owns child inventory",
        &child,
        &[
            "pub(super) fn assert_f12_direct_sources_are_folder_backed",
            F12_DIRECT_ASSERTIONS_DELEGATION_CHILD,
            F12_DIRECT_ASSERTIONS_REVIEW_GUARD_CHILD,
            F12_DIRECT_ASSERTIONS_BUDGETS_CHILD,
            F12_DIRECT_ASSERTIONS_STATUS_MIRRORS_CHILD,
            "runtime_15_code_review_findings_f12_direct_assertions_are_child_owner",
            F12_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
            F12_DIRECT_ASSERTIONS_STATUS_GUARD,
        ],
    );

    assert_f12_direct_sources_are_folder_backed(&sources);
    for (_, child_path, child_guard) in F12_DIRECT_ASSERTIONS_GUARD_CHILDREN {
        assert!(
            child.contains(child_path),
            "F12 direct assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "F12 direct assertions child source blob should contain child guard {child_guard}"
        );
    }
    budgets::assert_f12_direct_assertions_children_line_budgets_are_current();
}
