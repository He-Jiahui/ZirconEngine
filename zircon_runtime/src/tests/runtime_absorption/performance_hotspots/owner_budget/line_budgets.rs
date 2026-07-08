#[path = "line_budgets/artifact_render_diagnostics.rs"]
mod artifact_render_diagnostics;
#[path = "line_budgets/hotspot_inventory.rs"]
mod hotspot_inventory;
#[path = "line_budgets/owner_budget.rs"]
mod owner_budget;
#[path = "line_budgets/root.rs"]
mod root;
#[path = "line_budgets/scene_project.rs"]
mod scene_project;
#[path = "line_budgets/submit_context.rs"]
mod submit_context;

use super::{assert_contains_all, sources::OwnerBudgetSources};

pub(super) fn assert_performance_hotspot_guard_budgets(sources: &OwnerBudgetSources) {
    root::assert_root_file_budgets(sources);
    artifact_render_diagnostics::assert_artifact_render_diagnostics_budgets(sources);
    hotspot_inventory::assert_hotspot_inventory_budgets(sources);
    owner_budget::assert_owner_budget_budgets(sources);
    scene_project::assert_scene_project_budgets(sources);
    submit_context::assert_submit_context_budgets(sources);
}

pub(super) fn assert_line_budgets_guard_folder_backed(sources: &OwnerBudgetSources) {
    assert_contains_all(
        "owner-budget line-budgets route",
        sources.owner_budget_line_budgets,
        &[
            "#[path = \"line_budgets/artifact_render_diagnostics.rs\"]",
            "#[path = \"line_budgets/hotspot_inventory.rs\"]",
            "#[path = \"line_budgets/owner_budget.rs\"]",
            "#[path = \"line_budgets/root.rs\"]",
            "#[path = \"line_budgets/scene_project.rs\"]",
            "#[path = \"line_budgets/submit_context.rs\"]",
            "root::assert_root_file_budgets(sources);",
            "artifact_render_diagnostics::assert_artifact_render_diagnostics_budgets(sources);",
            "hotspot_inventory::assert_hotspot_inventory_budgets(sources);",
            "owner_budget::assert_owner_budget_budgets(sources);",
            "scene_project::assert_scene_project_budgets(sources);",
            "submit_context::assert_submit_context_budgets(sources);",
        ],
    );

    let moved_anchors = [
        format!("{}{}", "sources.", "artifact_render_diagnostics"),
        format!("{}{}", "sources.", "hotspot_inventory"),
        format!("{}{}", "sources.", "owner_budget_child_routes"),
        format!("{}{}", "sources.", "scene_project_splits"),
        format!("{}{}", "sources.", "submit_context_camera_loop"),
        format!("{}{}", "sources.", "submit_error_paths"),
    ];
    for moved_anchor in moved_anchors {
        assert!(
            !sources.owner_budget_line_budgets.contains(&moved_anchor),
            "owner_budget/line_budgets.rs should route instead of owning budget entry `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "owner-budget line-budget children",
        &format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            sources.owner_budget_line_budgets_artifact_render_diagnostics,
            sources.owner_budget_line_budgets_hotspot_inventory,
            sources.owner_budget_line_budgets_owner_budget,
            sources.owner_budget_line_budgets_root,
            sources.owner_budget_line_budgets_scene_project,
            sources.owner_budget_line_budgets_submit_context
        ),
        &[
            "assert_artifact_render_diagnostics_budgets",
            "assert_hotspot_inventory_budgets",
            "assert_owner_budget_budgets",
            "assert_root_file_budgets",
            "assert_scene_project_budgets",
            "assert_submit_context_budgets",
            "tests/runtime_absorption/performance_hotspots/owner_budget/child_routes/submit_context.rs",
        ],
    );
}

pub(super) fn assert_runtime_15_test_file_budget(path: &str, source: &str) {
    let line_count = source.lines().count();
    assert!(
        line_count < 800,
        "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
    );
}
