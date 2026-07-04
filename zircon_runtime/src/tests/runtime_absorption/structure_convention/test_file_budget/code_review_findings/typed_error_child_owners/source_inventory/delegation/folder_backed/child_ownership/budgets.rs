use super::super::super::super::super::super::super::*;
use super::super::super::super::*;

pub(super) fn assert_typed_error_source_inventory_delegation_folder_backed_ownership_child_budgets_are_current(
) {
    let ownership_parent = read_runtime_src(
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILD_OWNERSHIP_CHILD,
    );
    for (path, source) in IntoIterator::into_iter([(
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILD_OWNERSHIP_CHILD,
        ownership_parent,
    )]
    )
    .chain(typed_error_source_inventory_delegation_folder_backed_ownership_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
