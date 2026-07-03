type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 Runtime 07 performance hotspot guard folder split",
        &[
            "runtime_15_runtime_07_performance_hotspots_guard_folder_split_static_passed_cargo_timeout_no_result",
            "tests/runtime_absorption/performance_hotspots.rs",
            "tests/runtime_absorption/performance_hotspots/submit_context.rs",
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory.rs",
            "runtime_15_runtime_07_performance_hotspots_guard_is_folder_backed",
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
