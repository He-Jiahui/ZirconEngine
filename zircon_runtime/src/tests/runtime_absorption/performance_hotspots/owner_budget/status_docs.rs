use super::{assert_contains_all, sources::OwnerBudgetSources};

pub(super) fn assert_performance_hotspot_status_docs(sources: &OwnerBudgetSources) {
    for (label, source) in [
        ("Runtime 15 plan", sources.runtime_15_plan),
        ("Runtime index", sources.runtime_index),
        ("review findings", sources.review_findings),
        ("structure convention", sources.structure_convention),
        ("module convention doc", sources.module_doc),
        ("status-output row data", sources.status_rows),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 Runtime 07 performance hotspot guard folder split",
                "runtime_15_runtime_07_performance_hotspots_guard_folder_split_static_passed_cargo_timeout_no_result",
                "tests/runtime_absorption/performance_hotspots.rs",
                "tests/runtime_absorption/performance_hotspots/submit_context.rs",
                "runtime_15_runtime_07_performance_hotspots_guard_is_folder_backed",
                "Runtime 15 M3 Runtime 07 owner-budget mirror-docs sources guard folder-backed split",
                "runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_static_passed_cargo_deferred",
                "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/sources.rs",
                "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/sources/load.rs",
                "runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_split",
                "Runtime 15 M3 Runtime 07 owner-budget sources guard folder-backed split",
                "runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_static_passed_cargo_deferred",
                "tests/runtime_absorption/performance_hotspots/owner_budget/sources/load.rs",
                "runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_split",
                "Runtime 15 M3 Runtime 07 owner-budget child-routes guard folder-backed split",
                "runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_static_passed_cargo_deferred",
                "tests/runtime_absorption/performance_hotspots/owner_budget/child_routes/submit_context.rs",
                "runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_split",
                "Runtime 15 M3 Runtime 07 owner-budget line-budgets guard folder-backed split",
                "runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_static_passed_cargo_deferred",
                "tests/runtime_absorption/performance_hotspots/owner_budget/line_budgets/owner_budget.rs",
                "runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_split",
                "Runtime 15 M3 Runtime 07 owner-budget split-layout route guard folder-backed split",
                "runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_static_passed_cargo_deferred",
                "tests/runtime_absorption/performance_hotspots/owner_budget/split_layout/route/support_routes.rs",
                "runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_split",
            ],
        );
    }

    for (label, source) in [
        ("Runtime 07 plan", sources.runtime_07_plan),
        ("Runtime index", sources.runtime_index),
        ("hotspot inventory doc", sources.hotspot_doc),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "expected_test_file_count = 91",
                "Runtime 15 M3 Runtime 07 owner-budget child-routes guard folder-backed split",
                "runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_static_passed_cargo_deferred",
                "tests/runtime_absorption/performance_hotspots/owner_budget/child_routes/submit_context.rs",
                "runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_split",
                "Runtime 15 M3 Runtime 07 owner-budget line-budgets guard folder-backed split",
                "runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_static_passed_cargo_deferred",
                "tests/runtime_absorption/performance_hotspots/owner_budget/line_budgets/{root,artifact_render_diagnostics,hotspot_inventory,owner_budget,scene_project,submit_context}.rs",
                "runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_split",
                "Runtime 15 M3 Runtime 07 owner-budget split-layout route guard folder-backed split",
                "runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_static_passed_cargo_deferred",
                "tests/runtime_absorption/performance_hotspots/owner_budget/split_layout/route/{parent_route,split_route,support_routes}.rs",
                "runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_split",
                "performance_hotspots/owner_budget/{large_file_gate,mirror_docs,virtual_geometry_debug_snapshot}.rs",
                "performance_hotspots/owner_budget/mirror_docs/sources/{assertions,load,views}.rs",
                "performance_hotspots/owner_budget/sources/load.rs",
            ],
        );
    }

    let status_and_date_slices = format!("{}\n{}", sources.status_slice, sources.date_slice);
    assert_contains_all(
        "status-output slices",
        &status_and_date_slices,
        &[
            "runtime_15_runtime_07_performance_hotspots_guard_folder_split_static_passed_cargo_timeout_no_result",
            "runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_static_passed_cargo_deferred",
            "2026-06-23",
            "2026-07-06",
        ],
    );
}
