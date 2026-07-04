use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_typed_error_convergence_mount_budgets_are_focused(
    sources: &TypedErrorConvergenceMountSources,
) {
    for (path, source) in typed_error_convergence_mount_source_files(sources) {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
    for (path, source) in typed_error_convergence_mount_child_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 convergence mounts child budget; got {line_count} lines"
        );
    }
}
