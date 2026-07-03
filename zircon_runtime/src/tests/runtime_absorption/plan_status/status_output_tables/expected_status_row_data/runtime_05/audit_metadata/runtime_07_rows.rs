use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 status-output Runtime 07 scene asset rows",
        &[
            "Runtime 07 scene asset owner split",
            "Runtime 07 scene asset split-drift repair",
            "hotspot_guard_anchor_count = 20",
            "`scene_asset` / Runtime 07 Cargo gates pending",
        ],
    ),
    (
        "Runtime 05 Runtime 07 scene status 审计元数据",
        &[
            "runtime_07_scene_status_index_anchor_count = 11",
            "runtime_07_scene_status_guard_anchor_count = 10",
            "runtime_07_scene_status_guard_present = true",
            "index 11/11",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 07 owner-budget row",
        &[
            "Runtime 07 owner-budget 0-hotspot current audit sync",
            "large_file_m1_gate_status = classified-and-clear",
            "large_file_hotspot_count = 0",
            "large_file_migration_debt_count = 0",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=0",
        ],
    ),
    (
        "Runtime 05 plan-status owner-budget current mirror fix",
        &[
            "runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs",
            "large_file_hotspot_count = 0",
            "large_file_m1_gate_status = classified-and-clear",
            "Runtime 07 owner-budget 0-hotspot current audit sync",
        ],
    ),
    (
        "Runtime 05 Runtime 07 owner-budget status 审计元数据",
        &[
            "runtime_07_owner_budget_status_index_anchor_count = 9",
            "runtime_07_owner_budget_status_guard_anchor_count = 8",
            "runtime_07_owner_budget_status_guard_present = true",
            "index 9/9",
        ],
    ),
];
