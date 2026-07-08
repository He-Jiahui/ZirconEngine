#[path = "route/parent_route.rs"]
mod parent_route;
#[path = "route/split_route.rs"]
mod split_route;
#[path = "route/support_routes.rs"]
mod support_routes;

use super::super::{assert_contains_all, sources::OwnerBudgetSources};

pub(super) fn assert_owner_budget_split_layout(sources: &OwnerBudgetSources) {
    parent_route::assert_owner_budget_parent_route(sources);
    support_routes::assert_owner_budget_support_routes(sources);
    split_route::assert_owner_budget_split_route(sources);
}

pub(super) fn assert_owner_budget_split_layout_route_folder_backed(sources: &OwnerBudgetSources) {
    assert_contains_all(
        "owner-budget split-layout route owner",
        sources.owner_budget_split_layout_route,
        &[
            "#[path = \"route/parent_route.rs\"]",
            "#[path = \"route/split_route.rs\"]",
            "#[path = \"route/support_routes.rs\"]",
            "parent_route::assert_owner_budget_parent_route(sources);",
            "support_routes::assert_owner_budget_support_routes(sources);",
            "split_route::assert_owner_budget_split_route(sources);",
        ],
    );

    let moved_anchors = [
        format!("{}{}", "fn ", "assert_owner_budget_parent_route("),
        format!("{}{}", "fn ", "assert_owner_budget_support_routes("),
        format!("{}{}", "fn ", "assert_owner_budget_split_route("),
        format!("{}{}", "owner-budget child routes ", "support children"),
        format!("{}{}", "owner-budget mirror-docs ", "support children"),
    ];
    for moved_anchor in moved_anchors {
        assert!(
            !sources
                .owner_budget_split_layout_route
                .contains(&moved_anchor),
            "owner_budget/split_layout/route.rs should route instead of owning `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "owner-budget split-layout route children",
        &format!(
            "{}\n{}\n{}",
            sources.owner_budget_split_layout_route_parent_route,
            sources.owner_budget_split_layout_route_split_route,
            sources.owner_budget_split_layout_route_support_routes
        ),
        &[
            "assert_owner_budget_parent_route",
            "assert_owner_budget_support_routes",
            "assert_owner_budget_split_route",
            "runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_split",
        ],
    );
}
