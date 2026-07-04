use super::super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_f8_direct_assertions_are_child_owner() {
    let parent = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    let child = read_runtime_src(F8_DIRECT_ASSERTIONS_CHILD);
    let child_blob = f8_direct_assertion_child_source_blob();
    let sources = super::super::super::source_inventory::code_review_findings_sources();

    assert_contains_all(
        "direct-review assertion child delegates F8 assertions to child owner",
        &parent,
        &[
            "#[path = \"direct_review_assertions/f8.rs\"]",
            "mod f8;",
            "f8::assert_f8_direct_sources_are_folder_backed",
        ],
    );
    for moved_guard in [
        concat!(
            "F8 API convergence parent only mounts focused child ",
            "review guard owners"
        ),
        concat!(
            "F8 descriptor privacy leaf owners keep private-field ",
            "and constructor review guards"
        ),
        concat!(
            "review_f8_runtime_plugin_descriptor_public_",
            "constructor_is_retired"
        ),
    ] {
        assert!(
            !parent.contains(moved_guard),
            "F8 direct assertion `{moved_guard}` should stay in {F8_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    assert_contains_all(
        "F8 direct assertion parent owns child inventory",
        &child,
        &[
            "pub(super) fn assert_f8_direct_sources_are_folder_backed",
            F8_DIRECT_ASSERTIONS_DELEGATION_CHILD,
            F8_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD,
            F8_DIRECT_ASSERTIONS_REVIEW_CHILDREN_CHILD,
            F8_DIRECT_ASSERTIONS_BUDGETS_CHILD,
            F8_DIRECT_ASSERTIONS_STATUS_MIRRORS_CHILD,
            "runtime_15_code_review_findings_f8_direct_assertions_are_child_owner",
            F8_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
            F8_DIRECT_ASSERTIONS_STATUS_GUARD,
        ],
    );

    assert_f8_direct_sources_are_folder_backed(&sources);
    for (_, child_path, child_guard) in F8_DIRECT_ASSERTIONS_GUARD_CHILDREN {
        assert!(
            child.contains(child_path),
            "F8 direct assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "F8 direct assertions child source blob should contain child guard {child_guard}"
        );
    }
    budgets::assert_f8_direct_assertions_children_line_budgets_are_current();
}
