use super::super::super::super::*;
use super::*;

pub(super) fn assert_typed_error_structure_guard_line_budgets() {
    for (path, source) in [
        (
            STRUCTURE_GUARD_CHILD_OWNER,
            read_runtime_src(STRUCTURE_GUARD_CHILD_OWNER),
        ),
        (
            STRUCTURE_GUARD_TYPED_ERROR_CHILD_OWNER,
            read_runtime_src(STRUCTURE_GUARD_TYPED_ERROR_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_SOURCE_INVENTORY_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_STRUCTURE_DELEGATION_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_STRUCTURE_DELEGATION_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_NATIVE_STRUCTURE_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_NATIVE_STRUCTURE_CHILD_OWNER),
        ),
    ]
    .into_iter()
    .chain(typed_error_structure_guard_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_structure_guard_typed_error_children_line_budgets_are_current() {
    assert_typed_error_structure_guard_line_budgets();
}
