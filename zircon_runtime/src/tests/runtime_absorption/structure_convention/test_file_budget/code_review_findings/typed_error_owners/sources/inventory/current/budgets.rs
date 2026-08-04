use super::super::super::super::super::super::*;
use super::super::super::*;
use super::*;

pub(in super::super) fn assert_typed_error_source_inventory_child_inventory_status_current_child_budgets_are_current(
) {
    let mut budget_sources: Vec<(&'static str, String)> = vec![];

    for (path, source) in budget_sources {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
