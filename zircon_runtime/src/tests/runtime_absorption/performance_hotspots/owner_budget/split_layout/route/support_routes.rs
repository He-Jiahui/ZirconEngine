use super::super::super::{assert_contains_all, sources::OwnerBudgetSources};

pub(super) fn assert_owner_budget_support_routes(sources: &OwnerBudgetSources) {
    assert_contains_all(
        "owner-budget sources",
        sources.owner_budget_sources,
        &[
            "pub(super) struct OwnerBudgetSources",
            "pub(super) fn load() -> OwnerBudgetSources",
            "#[path = \"sources/load.rs\"]",
            "mod load_sources;",
            "load_sources::load()",
            "pub(super) fn assert_sources_guard_folder_backed",
            "owner_budget_sources_load",
        ],
    );
    assert_contains_all(
        "owner-budget source-loading child",
        sources.owner_budget_sources_load,
        &[
            "pub(super) fn load() -> OwnerBudgetSources",
            "performance_hotspots.rs",
            "ecs_extract_counters/split_layout.rs",
            "mirror_docs/sources.rs",
            "split_layout/route.rs",
            "split_layout/source_inventory.rs",
            "split_layout/status_docs.rs",
            "performance_hotpath_source_inventory.py",
            "runtime07_script_maps.rs",
        ],
    );
    assert_contains_all(
        "owner-budget parent routes child",
        sources.owner_budget_parent_routes,
        &[
            "assert_performance_hotspots_parent_routes",
            "mod artifact_render_diagnostics_splits;",
            "fn runtime_07_submit_context_shares_large_extract_payloads",
        ],
    );
    assert_contains_all(
        "owner-budget child routes child",
        sources.owner_budget_child_routes,
        &[
            "#[path = \"child_routes/artifact_render_diagnostics.rs\"]",
            "#[path = \"child_routes/hotspot_inventory.rs\"]",
            "#[path = \"child_routes/owner_budget.rs\"]",
            "#[path = \"child_routes/scene_project.rs\"]",
            "#[path = \"child_routes/submit_context.rs\"]",
            "assert_performance_hotspot_child_routes",
            "assert_child_routes_guard_folder_backed",
        ],
    );
    assert_contains_all(
        "owner-budget child routes support children",
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
            "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
            "owner_budget/mirror_docs/split_layout.rs",
        ],
    );
    assert_contains_all(
        "owner-budget line-budgets child",
        sources.owner_budget_line_budgets,
        &[
            "#[path = \"line_budgets/artifact_render_diagnostics.rs\"]",
            "#[path = \"line_budgets/hotspot_inventory.rs\"]",
            "#[path = \"line_budgets/owner_budget.rs\"]",
            "#[path = \"line_budgets/root.rs\"]",
            "#[path = \"line_budgets/scene_project.rs\"]",
            "#[path = \"line_budgets/submit_context.rs\"]",
            "assert_performance_hotspot_guard_budgets",
            "assert_line_budgets_guard_folder_backed",
        ],
    );
    assert_contains_all(
        "owner-budget line-budgets support children",
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
        ],
    );
    assert_contains_all(
        "owner-budget mirror-docs route",
        sources.owner_budget_mirror_docs,
        &[
            "#[path = \"mirror_docs/audit_wiring.rs\"]",
            "#[path = \"mirror_docs/doc_mirrors.rs\"]",
            "#[path = \"mirror_docs/performance_guard.rs\"]",
            "#[path = \"mirror_docs/source_inventory.rs\"]",
            "#[path = \"mirror_docs/sources.rs\"]",
            "#[path = \"mirror_docs/split_layout.rs\"]",
        ],
    );
    assert_contains_all(
        "owner-budget mirror-docs support children",
        &format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            sources.owner_budget_mirror_docs_audit_wiring,
            sources.owner_budget_mirror_docs_doc_mirrors,
            sources.owner_budget_mirror_docs_performance_guard,
            sources.owner_budget_mirror_docs_source_inventory,
            sources.owner_budget_mirror_docs_sources,
            sources.owner_budget_mirror_docs_split_layout
        ),
        &[
            "assert_audit_wiring_anchors",
            "assert_runtime_07_mirror_docs",
            "assert_performance_guard_anchors",
            "assert_source_inventory_anchors",
            "#[path = \"sources/assertions.rs\"]",
            "#[path = \"sources/load.rs\"]",
            "#[path = \"sources/views.rs\"]",
            "runtime_15_runtime_07_owner_budget_mirror_docs_guard_folder_backed_split",
            "runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_split",
            "runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_split",
        ],
    );
}
