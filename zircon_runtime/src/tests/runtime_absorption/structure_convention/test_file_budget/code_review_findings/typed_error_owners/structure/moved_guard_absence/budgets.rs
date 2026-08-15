use super::super::super::super::super::*;
use super::super::TYPED_ERROR_CHILD_OWNER_LINE_BUDGET;
use super::*;

pub(super) fn assert_typed_error_moved_guard_absence_line_budgets() {
    for (path, source) in [
        (
            TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD,
            read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD),
        ),
        (
            TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD,
            read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD),
        ),
    ]
    .into_iter()
    .chain(moved_guard_absence_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_typed_error_moved_guard_absence_children_line_budgets_are_current() {
    assert_typed_error_moved_guard_absence_line_budgets();
}
