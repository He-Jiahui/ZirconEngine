use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 status-output Runtime 07 scene asset rows",
        [
            "Runtime 07 scene asset owner split",
            "Runtime 07 scene asset split-drift repair",
            "hotspot_guard_anchor_count = 20",
            "`scene_asset` / Runtime 07 Cargo gates pending",
        ],
    ),
    (
        "Runtime 05 Runtime 07 scene status 审计元数据",
        [
            "runtime_07_scene_status_index_anchor_count = 11",
            "runtime_07_scene_status_guard_anchor_count = 10",
            "runtime_07_scene_status_guard_present = true",
            "index 11/11",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 07 owner-budget row",
        [
            "Runtime 07 owner-budget 30-hotspot current audit sync",
            "large_file_hotspot_count = 30",
            "runtime-other=13",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=30",
        ],
    ),
    (
        "Runtime 05 plan-status owner-budget current mirror fix",
        [
            "runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs",
            "large_file_hotspot_count = 30",
            "runtime-other=13",
            "Runtime 07 owner-budget 30-hotspot current audit sync",
        ],
    ),
    (
        "Runtime 05 Runtime 07 owner-budget status 审计元数据",
        [
            "runtime_07_owner_budget_status_index_anchor_count = 7",
            "runtime_07_owner_budget_status_guard_anchor_count = 6",
            "runtime_07_owner_budget_status_guard_present = true",
            "index 7/7",
        ],
    ),
];
