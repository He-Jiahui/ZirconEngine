use super::*;

#[test]
fn runtime_15_p0_robustness_structure_guard_budgets_are_focused() {
    let parent = read_runtime_src(STRUCTURE_GUARD_OWNER);
    assert!(
        parent.lines().count() < 220,
        "{STRUCTURE_GUARD_OWNER} should stay below the focused parent budget"
    );

    let review_sources = read_p0_robustness_sources();
    for (path, source) in review_sources.all_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
    for (path, source) in folder_backed_child_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < 220,
            "{path} should stay below the focused P0 robustness structure child budget; got {line_count} lines"
        );
    }
}
