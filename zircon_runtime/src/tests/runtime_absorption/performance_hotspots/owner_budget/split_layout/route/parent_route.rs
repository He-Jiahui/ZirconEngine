use super::super::super::{assert_contains_all, sources::OwnerBudgetSources};

pub(super) fn assert_owner_budget_parent_route(sources: &OwnerBudgetSources) {
    assert_contains_all(
        "owner-budget route",
        sources.owner_budget,
        &[
            "#[path = \"owner_budget/child_routes.rs\"]",
            "#[path = \"owner_budget/line_budgets.rs\"]",
            "#[path = \"owner_budget/mirror_docs.rs\"]",
            "#[path = \"owner_budget/parent_routes.rs\"]",
            "#[path = \"owner_budget/source_inventory.rs\"]",
            "#[path = \"owner_budget/sources.rs\"]",
            "#[path = \"owner_budget/split_layout.rs\"]",
            "#[path = \"owner_budget/status_docs.rs\"]",
            "fn runtime_15_runtime_07_performance_hotspots_guard_is_folder_backed()",
            "parent_routes::assert_performance_hotspots_parent_routes(&sources);",
            "child_routes::assert_performance_hotspot_child_routes(&sources);",
            "source_inventory::assert_performance_hotpath_source_inventory(&sources);",
            "line_budgets::assert_performance_hotspot_guard_budgets(&sources);",
            "status_docs::assert_performance_hotspot_status_docs(&sources);",
            "fn runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_split()",
            "sources::assert_sources_guard_folder_backed(&sources);",
            "fn runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_split()",
            "line_budgets::assert_line_budgets_guard_folder_backed(&sources);",
        ],
    );

    for moved_anchor in [
        "let parent = include_str!(\"../performance_hotspots.rs\")",
        "let submit_context_camera_loop = include_str!",
        "for moved_guard in [",
        "for moved_owner_budget_guard_name in [",
        "for (path, source) in [",
        "for (label, source) in [",
    ] {
        assert!(
            !sources.owner_budget.contains(moved_anchor),
            "owner_budget.rs should route instead of owning assertion block `{moved_anchor}`"
        );
    }
}
