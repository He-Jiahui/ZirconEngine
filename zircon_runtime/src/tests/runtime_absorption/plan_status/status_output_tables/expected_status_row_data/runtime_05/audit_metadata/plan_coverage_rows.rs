use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 plan-status 输出表守卫",
        &[
            "runtime_plan_status_output_tables_cover_index_and_all_subplans",
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "status_table_gaps []",
            "full scene:: Cargo gate 仍 pending",
        ],
    ),
    (
        "Runtime 05 plan-status 审计元数据守卫",
        &[
            "status_output_table_guard_count = 4",
            "missing_status_output_table_guard_anchors = []",
            "all runtime index status rows",
            "full coverage guard",
        ],
    ),
    (
        "Runtime 05 M0 absorption guard coverage sync",
        &[
            "runtime_architecture_review_documents_all_absorption_guards",
            "25 个 mounted",
            "runtime_absorption/ecs_kernel_data.rs",
            "runtime_absorption/script_binding.rs",
        ],
    ),
    (
        "Runtime 05 status-output current anchor fix",
        &[
            "runtime_02_generated_status_guard_anchor_count = 5",
            "runtime_07_owner_budget_status_guard_anchor_count = 5",
            "runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending",
            "missing_runtime_07_owner_budget_status_guard_anchors = []",
        ],
    ),
    (
        "Runtime 05 status-output all-index-row coverage guard",
        &[
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "all runtime index status rows",
            "standalone status-output 2/2",
            "full coverage guard",
        ],
    ),
    (
        "Runtime 05 plan-status Cargo attempt 状态审计",
        &[
            "runtime_plan_status_boundary",
            "cargo_attempt_status_anchor_count = 20",
            "cargo_attempt_status_guard_present = true",
            "Runtime 14 animation Cargo gate 尝试",
        ],
    ),
    (
        "Runtime 05 plan-status Cargo timeout 状态审计",
        &[
            "runtime_plan_status_boundary",
            "cargo_recheck_timeout_no_result",
            "Runtime 14 animation runtime-status focused recheck timeout",
            "runtime_status_reports_player_rig_and_gpu_readiness",
        ],
    ),
    (
        "Runtime 05 status-output audit-metadata owner split",
        &[
            "expected_status_row_data/runtime_05/audit_metadata.rs",
            "runtime_05/audit_metadata/{plan_coverage_rows,runtime_02_03_rows,runtime_07_rows}.rs",
            "Runtime 05 audit rows owner groups separately",
            "plan-status support files 74/74",
        ],
    ),
];
