#[path = "child_routes/artifact_render_diagnostics.rs"]
mod artifact_render_diagnostics;
#[path = "child_routes/hotspot_inventory.rs"]
mod hotspot_inventory;
#[path = "child_routes/owner_budget.rs"]
mod owner_budget_routes;
#[path = "child_routes/scene_project.rs"]
mod scene_project;
#[path = "child_routes/submit_context.rs"]
mod submit_context;

use super::{assert_contains_all, sources::OwnerBudgetSources};

pub(super) fn assert_performance_hotspot_child_routes(sources: &OwnerBudgetSources) {
    submit_context::assert_submit_context_routes(sources);
    hotspot_inventory::assert_hotspot_inventory_routes(sources);
    scene_project::assert_scene_project_routes(sources);
    artifact_render_diagnostics::assert_artifact_render_diagnostics_routes(sources);
    owner_budget_routes::assert_owner_budget_routes(sources);
}

pub(super) fn assert_child_routes_guard_folder_backed(sources: &OwnerBudgetSources) {
    assert_contains_all(
        "owner-budget child-routes route",
        sources.owner_budget_child_routes,
        &[
            "#[path = \"child_routes/artifact_render_diagnostics.rs\"]",
            "#[path = \"child_routes/hotspot_inventory.rs\"]",
            "#[path = \"child_routes/owner_budget.rs\"]",
            "#[path = \"child_routes/scene_project.rs\"]",
            "#[path = \"child_routes/submit_context.rs\"]",
            "submit_context::assert_submit_context_routes(sources);",
            "hotspot_inventory::assert_hotspot_inventory_routes(sources);",
            "scene_project::assert_scene_project_routes(sources);",
            "artifact_render_diagnostics::assert_artifact_render_diagnostics_routes(sources);",
            "owner_budget_routes::assert_owner_budget_routes(sources);",
        ],
    );

    let moved_anchors = [
        format!("{}{}", "fn ", "assert_submit_context_routes("),
        format!("{}{}", "hotspot inventory ", "ECS/extract support children"),
        format!("{}{}", "scene/project ", "support children"),
        format!("{}{}", "artifact/render diagnostics ", "support children"),
        format!("{}{}", "for ", "moved_owner_budget_guard_name in ["),
        format!("{}{}", "owner-budget mirror docs ", "support children"),
    ];
    for moved_anchor in moved_anchors {
        assert!(
            !sources.owner_budget_child_routes.contains(&moved_anchor),
            "owner_budget/child_routes.rs should route instead of owning assertion block `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "owner-budget child-routes support children",
        &format!(
            "{}\n{}\n{}\n{}\n{}",
            sources.owner_budget_child_routes_artifact_render_diagnostics,
            sources.owner_budget_child_routes_hotspot_inventory,
            sources.owner_budget_child_routes_owner_budget,
            sources.owner_budget_child_routes_scene_project,
            sources.owner_budget_child_routes_submit_context
        ),
        &[
            "assert_artifact_render_diagnostics_routes",
            "assert_hotspot_inventory_routes",
            "assert_owner_budget_routes",
            "assert_scene_project_routes",
            "assert_submit_context_routes",
            "runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_split",
        ],
    );
}
