from __future__ import annotations


CORE_GUARD_ANCHORS = (
    "runtime_index_subplan_map_covers_existing_plan_files_without_stale_rows",
    "runtime_index_problem_rows_reference_existing_subplans",
    "runtime_index_execution_dependencies_reference_existing_subplans",
    "runtime_plan_last_refined_covers_latest_recorded_date",
    "runtime_plan_status_does_not_claim_completed_while_validation_is_pending",
    "runtime_plan_frontmatter_status_uses_known_lifecycle_values",
    "runtime_index_status_map_matches_subplan_frontmatter",
    "runtime_index_in_progress_rows_record_remaining_gate",
    "runtime_known_backlog_gaps_keep_owner_and_trigger_columns",
    "runtime_subplans_keep_status_and_evidence_tables",
    "runtime_subplan_status_records_keep_non_empty_evidence",
    "runtime_plan_status_output_tables_cover_index_and_all_subplans",
    "runtime_index_status_output_records_recent_cross_plan_slices",
    "runtime_05_closeout_status_waits_for_full_scene_cargo_gate",
    "runtime_architecture_review_documents_all_absorption_guards",
)

PENDING_GATE_ANCHORS = (
    "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
    "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
    "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
    "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
    "runtime_05_closeout_status_waits_for_full_scene_cargo_gate",
    "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
    "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
    "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
    "runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation",
    "runtime_10_m1_3_cargo_pending_gate_stays_explicit_until_dynamic_api_validation",
    "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
    "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
    "runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation",
    "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
    "runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass",
)

DOC_ANCHORS = (
    "runtime_plan_status_boundary",
    "pending_full_scene_cargo",
    "cargo test -p zircon_runtime --lib scene:: --locked",
    "runtime_plan_last_refined_covers_latest_recorded_date",
    "runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs",
    "runtime_architecture_review_documents_all_absorption_guards",
    "Runtime 05 closeout",
    "plan-status",
    "status_output_tables",
    "cargo_attempt_status_anchor_count = 20",
    "cargo_attempt_status_guard_present = true",
)

BACKLOG_GAPS = (
    "网络复制 runtime 侧",
    "音频 runtime 服务面",
    "FFI panic 安全",
    "输入录制/回放",
    "脚本调试器/断点面",
    "存档/会话持久化语义",
    "本地化/i18n",
)
