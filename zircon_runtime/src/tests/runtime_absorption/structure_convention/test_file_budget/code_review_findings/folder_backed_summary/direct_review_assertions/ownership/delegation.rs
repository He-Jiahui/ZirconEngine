use super::super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_direct_assertions_children_are_child_owned() {
    let sources = super::super::super::source_inventory::code_review_findings_sources();

    parent_absence::assert_direct_review_parent_moved_guards_stay_in_children(&read_runtime_src(
        DIRECT_REVIEW_ASSERTIONS_CHILD,
    ));
    entry_points::assert_direct_review_child_entry_points_are_current();
    assert_code_review_direct_sources_are_folder_backed(&sources);
    budgets::assert_direct_assertions_child_ownership_children_line_budgets_are_current();
}
