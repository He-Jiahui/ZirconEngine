use super::super::super::super::*;
use super::super::*;
use super::*;

pub(super) fn assert_structure_guard_children_line_budgets() {
    let source_paths = [
        STRUCTURE_GUARD_CHILD_OWNER,
        F8_CHILD_OWNER,
        F8_DELEGATION_CHILD_OWNER,
        F8_ROUTE_OWNERSHIP_CHILD_OWNER,
        F8_STATUS_MIRRORS_CHILD_OWNER,
        F8_BUDGETS_CHILD_OWNER,
        LATE_API_CLEANUP_CHILD_OWNER,
        LATE_API_CLEANUP_DELEGATION_CHILD_OWNER,
        LATE_API_CLEANUP_ROUTE_OWNERSHIP_CHILD_OWNER,
        LATE_API_CLEANUP_STATUS_MIRRORS_CHILD_OWNER,
        LATE_API_CLEANUP_BUDGETS_CHILD_OWNER,
        P0_CHILD_OWNER,
        P0_DELEGATION_CHILD_OWNER,
        P0_ROUTE_OWNERSHIP_CHILD_OWNER,
        P0_STATUS_MIRRORS_CHILD_OWNER,
        P0_BUDGETS_CHILD_OWNER,
        P0_NATIVE_FIXTURE_LEAF_OWNER,
        P0_NATIVE_FIXTURE_DELEGATION_CHILD_OWNER,
        P0_NATIVE_FIXTURE_LEAF_OWNERSHIP_CHILD_OWNER,
        P0_NATIVE_FIXTURE_STATUS_MIRRORS_CHILD_OWNER,
        P0_NATIVE_FIXTURE_BUDGETS_CHILD_OWNER,
        PLUGIN_IMPORTER_DX_CHILD_OWNER,
        PLUGIN_IMPORTER_DX_TOP_LEVEL_DELEGATION_CHILD_OWNER,
        PLUGIN_IMPORTER_DX_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER,
        PLUGIN_IMPORTER_DX_TOP_LEVEL_STATUS_MIRRORS_CHILD_OWNER,
        PLUGIN_IMPORTER_DX_TOP_LEVEL_BUDGETS_CHILD_OWNER,
        PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD_OWNER,
        PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD_OWNER,
        PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD_OWNER,
        TYPED_ERROR_CHILD_OWNER,
        TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD_OWNER,
        TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER,
        TYPED_ERROR_TOP_LEVEL_STATUS_MIRRORS_CHILD_OWNER,
        TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD_OWNER,
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_OWNER,
        TYPED_ERROR_STATUS_DOCS_CHILD_OWNER,
        TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER,
        TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD_OWNER,
        TYPED_ERROR_STRUCTURE_DELEGATION_CHILD_OWNER,
        TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD_OWNER,
        TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD_OWNER,
        TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD_OWNER,
        TYPED_ERROR_NATIVE_STRUCTURE_CHILD_OWNER,
        STATUS_DOCS_CHILD_OWNER,
        STATUS_DOCS_SOURCE_ANCHORS_CHILD_OWNER,
        STATUS_DOCS_STATUS_ANCHORS_CHILD_OWNER,
        STRUCTURE_GUARD_CHILDREN_BUDGETS_CHILD_OWNER,
        STRUCTURE_GUARD_CHILDREN_LINE_COUNTS_CHILD_OWNER,
        STRUCTURE_GUARD_CHILDREN_STATUS_MIRRORS_CHILD_OWNER,
    ];
    for path in source_paths {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
    for (path, source) in structure_guard_child_sources()
        .into_iter()
        .chain(structure_guard_children_budget_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the focused structure-guard child budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_structure_guard_children_line_budgets_are_child_owned() {
    assert_structure_guard_children_line_budgets();
}
