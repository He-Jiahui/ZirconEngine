use super::super::{assert_contains_all, sources::OwnerBudgetSources};

pub(super) fn assert_owner_budget_routes(sources: &OwnerBudgetSources) {
    assert_contains_all(
        "owner-budget parent",
        sources.owner_budget,
        &[
            "#[path = \"owner_budget/child_routes.rs\"]",
            "#[path = \"owner_budget/large_file_gate.rs\"]",
            "#[path = \"owner_budget/line_budgets.rs\"]",
            "#[path = \"owner_budget/mirror_docs.rs\"]",
            "#[path = \"owner_budget/parent_routes.rs\"]",
            "#[path = \"owner_budget/source_inventory.rs\"]",
            "#[path = \"owner_budget/sources.rs\"]",
            "#[path = \"owner_budget/split_layout.rs\"]",
            "#[path = \"owner_budget/status_docs.rs\"]",
            "#[path = \"owner_budget/virtual_geometry_debug_snapshot.rs\"]",
            "fn runtime_15_runtime_07_performance_hotspots_guard_is_folder_backed",
            "parent_routes::assert_performance_hotspots_parent_routes(&sources);",
            "child_routes::assert_performance_hotspot_child_routes(&sources);",
            "source_inventory::assert_performance_hotpath_source_inventory(&sources);",
            "line_budgets::assert_performance_hotspot_guard_budgets(&sources);",
            "status_docs::assert_performance_hotspot_status_docs(&sources);",
        ],
    );

    for moved_owner_budget_guard_name in [
        "runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit",
        "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
        "runtime_07_virtual_geometry_debug_snapshot_owner_split_keeps_contracts_folder_backed",
    ] {
        let moved_owner_budget_guard = format!("fn {moved_owner_budget_guard_name}");
        assert!(
            !sources.owner_budget.contains(&moved_owner_budget_guard),
            "performance_hotspots/owner_budget.rs should mount child owners instead of defining `{moved_owner_budget_guard}`"
        );
    }

    let mirror_docs_guard = format!(
        "{}{}",
        "fn ", "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts"
    );
    assert!(
        sources
            .owner_budget_mirror_docs
            .contains(&mirror_docs_guard),
        "owner-budget mirror docs child should retain Runtime 07 audit mirror guard"
    );
    assert_contains_all(
        "owner-budget mirror docs route",
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
    let mirror_docs_children = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        sources.owner_budget_mirror_docs_audit_wiring,
        sources.owner_budget_mirror_docs_doc_mirrors,
        sources.owner_budget_mirror_docs_performance_guard,
        sources.owner_budget_mirror_docs_source_inventory,
        sources.owner_budget_mirror_docs_sources,
        sources.owner_budget_mirror_docs_split_layout
    );
    assert_contains_all(
        "owner-budget mirror docs support children",
        &mirror_docs_children,
        &[
            "EXPECTED_TEST_FILE_COUNT = 91",
            "owner_budget/mirror_docs/split_layout.rs",
            "runtime_15_runtime_07_owner_budget_mirror_docs_guard_folder_backed_split",
            "runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_split",
        ],
    );
}
