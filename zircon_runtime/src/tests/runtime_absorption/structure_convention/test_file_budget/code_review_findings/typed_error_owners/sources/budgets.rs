use super::*;

pub(super) fn assert_typed_error_line_budgets() {
    for (path, source) in super::reads::typed_error_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the typed-error child-owner budget; got {line_count} lines"
        );
    }
}

pub(super) fn assert_typed_error_source_inventory_children_line_budgets_are_current(
    sources: &TypedErrorSourceInventorySources,
) {
    let mut budget_sources: Vec<(&'static str, String)> = vec![
        (TYPED_ERROR_STRUCTURE_CHILD, sources.structure_child.clone()),
        (
            TYPED_ERROR_SOURCE_INVENTORY_CHILD,
            sources.source_inventory_child.clone(),
        ),
        (
            TYPED_ERROR_SOURCE_INVENTORY_PATHS_CHILD,
            sources.paths_child.clone(),
        ),
        (
            TYPED_ERROR_SOURCE_INVENTORY_READS_CHILD,
            sources.reads_child.clone(),
        ),
        (
            TYPED_ERROR_SOURCE_INVENTORY_BUDGETS_CHILD,
            sources.budgets_child.clone(),
        ),
        (
            TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILD,
            sources.delegation_child.clone(),
        ),
        (
            TYPED_ERROR_SOURCE_INVENTORY_STATUS_MIRRORS_CHILD,
            sources.status_mirrors_child.clone(),
        ),
    ];
    budget_sources.extend(typed_error_source_inventory_delegation_child_sources());
    budget_sources.extend(typed_error_source_inventory_delegation_folder_backed_child_sources());
    budget_sources
        .extend(typed_error_source_inventory_delegation_folder_backed_ownership_child_sources());

    for (path, source) in budget_sources {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_typed_error_source_inventory_children_line_budgets_are_current() {
    let sources = typed_error_source_inventory_sources();

    assert_typed_error_line_budgets();
    assert_typed_error_source_inventory_children_line_budgets_are_current(&sources);
}
