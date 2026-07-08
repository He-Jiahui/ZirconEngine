use super::super::super::super::*;
use super::*;

pub(super) fn assert_typed_error_child_ownership_budgets_are_focused(
    sources: &TypedErrorChildOwnershipSources,
) {
    for (path, source) in [
        (STRUCTURE_GUARD_PARENT, sources.parent.as_str()),
        (
            STRUCTURE_GUARD_TYPED_ERROR_CHILD,
            sources.structure_guard_typed_error_child.as_str(),
        ),
        (TYPED_ERROR_STRUCTURE_CHILD, sources.child.as_str()),
        (
            TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD,
            sources.child_ownership_child.as_str(),
        ),
        (
            TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD,
            sources.structure_assertions_child.as_str(),
        ),
        (
            TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD,
            sources.convergence_mounts_child.as_str(),
        ),
        (
            TYPED_ERROR_STRUCTURE_DELEGATION_CHILD,
            sources.delegation_child.as_str(),
        ),
        (
            TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD,
            sources.child_ownership_structure_child.as_str(),
        ),
        (
            TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD,
            sources.status_mirrors_child.as_str(),
        ),
        (
            TYPED_ERROR_NATIVE_STRUCTURE_CHILD,
            sources.native_plugin_loader_child.as_str(),
        ),
        (
            TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD,
            sources.moved_guard_absence_child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
    for (path, source) in typed_error_child_ownership_child_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 child-ownership child budget; got {line_count} lines"
        );
    }
}
