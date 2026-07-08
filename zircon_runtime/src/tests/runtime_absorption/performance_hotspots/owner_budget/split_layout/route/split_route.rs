use super::super::super::{assert_contains_all, sources::OwnerBudgetSources};

pub(super) fn assert_owner_budget_split_route(sources: &OwnerBudgetSources) {
    assert_contains_all(
        "owner-budget split-layout route",
        sources.owner_budget_split_layout,
        &[
            "#[path = \"split_layout/route.rs\"]",
            "#[path = \"split_layout/source_inventory.rs\"]",
            "#[path = \"split_layout/status_docs.rs\"]",
            "runtime_15_runtime_07_owner_budget_guard_folder_backed_split",
            "route::assert_owner_budget_split_layout(&sources);",
            "source_inventory::assert_owner_budget_source_inventory(&sources);",
            "source_inventory::assert_owner_budget_split_budgets(&sources);",
            "status_docs::assert_owner_budget_split_docs(&sources);",
        ],
    );

    for moved_anchor in [
        "fn assert_owner_budget_split_layout(",
        "fn assert_owner_budget_split_docs(",
        "for moved_anchor in [",
        "for (path, source) in [",
        "for (label, source) in [",
    ] {
        assert!(
            !sources.owner_budget_split_layout.contains(moved_anchor),
            "owner_budget/split_layout.rs should route instead of owning assertion block `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "owner-budget split-layout children",
        &format!(
            "{}\n{}\n{}",
            sources.owner_budget_split_layout_route,
            sources.owner_budget_split_layout_source_inventory,
            sources.owner_budget_split_layout_status_docs
        ),
        &[
            "assert_owner_budget_split_layout",
            "assert_owner_budget_source_inventory",
            "assert_owner_budget_split_docs",
            "Runtime 15 M3 Runtime 07 owner-budget split-layout guard folder-backed split",
        ],
    );
}
