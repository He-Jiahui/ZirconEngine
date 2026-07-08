type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 Runtime 07 owner-budget mirror-docs guard folder-backed split",
        &[
            "runtime_15_runtime_07_owner_budget_mirror_docs_guard_folder_backed_static_passed_cargo_deferred",
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/sources.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/source_inventory.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/split_layout.rs",
            "runtime_15_runtime_07_owner_budget_mirror_docs_guard_folder_backed_split",
            "expected_test_file_count = 50",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 07 owner-budget mirror-docs sources guard folder-backed split",
        &[
            "runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_static_passed_cargo_deferred",
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/sources.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/sources/load.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/sources/views.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/sources/assertions.rs",
            "runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_split",
            "expected_test_file_count = 76",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 07 owner-budget child-routes guard folder-backed split",
        &[
            "runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_static_passed_cargo_deferred",
            "tests/runtime_absorption/performance_hotspots/owner_budget/child_routes.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/child_routes/submit_context.rs",
            "runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_split",
            "expected_test_file_count = 82",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 07 owner-budget line-budgets guard folder-backed split",
        &[
            "runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_static_passed_cargo_deferred",
            "tests/runtime_absorption/performance_hotspots/owner_budget/line_budgets.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/line_budgets/owner_budget.rs",
            "runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_split",
            "expected_test_file_count = 88",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 07 owner-budget split-layout route guard folder-backed split",
        &[
            "runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_static_passed_cargo_deferred",
            "tests/runtime_absorption/performance_hotspots/owner_budget/split_layout/route.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/split_layout/route/support_routes.rs",
            "runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_split",
            "expected_test_file_count = 91",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 07 owner-budget sources guard folder-backed split",
        &[
            "runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_static_passed_cargo_deferred",
            "tests/runtime_absorption/performance_hotspots/owner_budget/sources.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/sources/load.rs",
            "runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_split",
            "expected_test_file_count = 77",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 07 owner-budget child-source current-route sync",
        &[
            "runtime_15_runtime_07_owner_budget_child_source_current_route_sync_static_passed_cargo_deferred",
            "tests/runtime_absorption/structure_convention/test_file_budget/runtime_07_performance_hotspots_owner_budget.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/runtime_07_performance_hotspots_owner_budget_large_file.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/runtime_07_performance_hotspots_owner_budget_mirror_docs.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/sources/load.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/source_inventory.rs",
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/runtime07_script_maps/runtime07_owner_budget_maps.rs",
            "runtime_15_runtime_07_owner_budget_virtual_geometry_guard_is_child_owner",
            "runtime_15_runtime_07_owner_budget_large_file_gate_is_child_owner",
            "runtime_15_runtime_07_owner_budget_mirror_docs_is_child_owner",
            "expected_test_file_count = 91",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 07 owner-budget virtual-geometry guard child-owner split",
        &[
            "runtime_15_runtime_07_owner_budget_virtual_geometry_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/performance_hotspots/owner_budget.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/virtual_geometry_debug_snapshot.rs",
            "runtime_07_virtual_geometry_debug_snapshot_owner_split_keeps_contracts_folder_backed",
            "runtime_15_runtime_07_owner_budget_virtual_geometry_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 07 owner-budget large-file gate child-owner split",
        &[
            "runtime_15_runtime_07_owner_budget_large_file_gate_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/performance_hotspots/owner_budget.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/large_file_gate.rs",
            "runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit",
            "runtime_15_runtime_07_owner_budget_large_file_gate_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 07 owner-budget mirror-docs child-owner split",
        &[
            "runtime_15_runtime_07_owner_budget_mirror_docs_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/performance_hotspots/owner_budget.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs.rs",
            "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
            "runtime_15_runtime_07_owner_budget_mirror_docs_is_child_owner",
            "expected_test_file_count = 14",
        ],
    ),
];
