use super::*;

#[test]
fn runtime_15_module_layout_child_summary_owner_budget_guard_child_split_status_is_current() {
    let status_rows = read_runtime_src(PRODUCTION_GUARD_SUPPORT_MODULE_LAYOUT_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    let status_anchors = [
        OWNER_BUDGET_CHILD_SPLIT_STATUS_NAME,
        OWNER_BUDGET_CHILD_SPLIT_STATUS_ID,
        "structure_convention/test_file_budget/row_data/module_layout_child_summaries/owner_budgets.rs",
        "structure_convention/test_file_budget/row_data/module_layout_child_summaries/owner_budgets/route_children.rs",
        "structure_convention/test_file_budget/row_data/module_layout_child_summaries/owner_budgets/nested_children.rs",
        "structure_convention/test_file_budget/row_data/module_layout_child_summaries/owner_budgets/surrounding_owners.rs",
        "structure_convention/test_file_budget/row_data/module_layout_child_summaries/owner_budgets/status_mirrors.rs",
        OWNER_BUDGET_CHILD_SPLIT_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "production guard module-layout rows record child-summary owner-budget split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "M3 status-support map records child-summary owner-budget split",
        &status_map,
        &[
            OWNER_BUDGET_CHILD_SPLIT_STATUS_NAME,
            OWNER_BUDGET_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 status-support date map records child-summary owner-budget split",
        &date_map,
        &[OWNER_BUDGET_CHILD_SPLIT_STATUS_NAME, "2026-07-04"],
    );

    for (label, path) in [
        (
            "Runtime 15 plan",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        ),
        (
            "Runtime index",
            "docs/plans/zircon_runtime/runtime/index.md",
        ),
        (
            "review findings",
            "docs/plans/engine-code-review-findings-2026-06.md",
        ),
        (
            "structure convention",
            "docs/plans/engine-code-structure-convention.md",
        ),
        (
            "module convention doc",
            "docs/zircon_runtime/structure/module-convention.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &status_anchors);
    }
    assert_contains_all(
        "module-layout child-summary owner-budget child source blob reaches every child",
        &owner_budget_child_source_blob(),
        &[
            "assert_module_layout_child_summary_route_owner_budgets_are_current",
            "assert_module_layout_child_summary_nested_budgets_are_current",
            "assert_module_layout_child_summary_surrounding_owner_budgets_are_current",
            "runtime_15_module_layout_child_summary_owner_budget_guard_child_split_status_is_current",
        ],
    );
}
