use super::super::super::super::super::super::*;
use super::super::*;
use super::*;

pub(in super::super) fn assert_typed_error_source_inventory_metadata_status_current_child_budgets_are_current(
) {
    let mut budget_sources: Vec<(&'static str, String)> = vec![(
        TYPED_ERROR_SOURCE_INVENTORY_METADATA_CHILD,
        read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_METADATA_CHILD),
    )];

    for (path, source) in budget_sources {
        let line_count = source.lines().count();
        assert!(
            line_count < super::super::TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
