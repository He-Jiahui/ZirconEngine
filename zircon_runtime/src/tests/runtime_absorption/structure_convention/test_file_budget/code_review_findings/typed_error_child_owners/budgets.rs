use super::*;

#[test]
fn runtime_15_typed_error_structure_guard_budgets_are_focused() {
    let sources = [
        (
            TYPED_ERROR_STRUCTURE_CHILD,
            read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD),
        ),
        (
            TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD,
            read_runtime_src(TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD),
        ),
        (
            TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD,
            read_runtime_src(TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD),
        ),
        (
            TYPED_ERROR_TOP_LEVEL_STATUS_MIRRORS_CHILD,
            read_runtime_src(TYPED_ERROR_TOP_LEVEL_STATUS_MIRRORS_CHILD),
        ),
        (
            TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD,
            read_runtime_src(TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD),
        ),
        (
            TYPED_ERROR_SOURCE_INVENTORY_CHILD,
            read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_CHILD),
        ),
        (
            TYPED_ERROR_STATUS_DOCS_CHILD,
            read_runtime_src(TYPED_ERROR_STATUS_DOCS_CHILD),
        ),
        (
            TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD,
            read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD),
        ),
        (
            TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD,
            read_runtime_src(TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD),
        ),
        (
            TYPED_ERROR_STRUCTURE_DELEGATION_CHILD,
            read_runtime_src(TYPED_ERROR_STRUCTURE_DELEGATION_CHILD),
        ),
        (
            TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD,
            read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD),
        ),
        (
            TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD,
            read_runtime_src(TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD),
        ),
        (
            TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD,
            read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD),
        ),
        (
            TYPED_ERROR_NATIVE_STRUCTURE_CHILD,
            read_runtime_src(TYPED_ERROR_NATIVE_STRUCTURE_CHILD),
        ),
    ];

    for (path, source) in sources {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
    for (path, source) in folder_backed_child_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < 260,
            "{path} should stay below the focused typed-error structure child budget; got {line_count} lines"
        );
    }
}
