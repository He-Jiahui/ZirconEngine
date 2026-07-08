use super::*;

pub(super) fn assert_module_layout_child_summary_route_owner_budgets_are_current() {
    let child_summary_parent = read_runtime_src(CHILD_SUMMARY_PARENT_PATH);
    assert!(
        child_summary_parent.lines().count() < 120,
        "module_layout_child_summaries.rs should stay below 120 lines as a route/shared-helper owner"
    );

    for (path, source) in child_summary_child_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < 220,
            "{path} should stay below the focused Runtime 15 child-summary guard budget; got {line_count} lines"
        );
    }
}
