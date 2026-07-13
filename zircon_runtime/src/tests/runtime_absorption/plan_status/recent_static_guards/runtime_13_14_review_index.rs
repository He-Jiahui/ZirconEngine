use super::super::support::assert_contains_all;
use super::document_sources::RecentStaticGuardSources;

pub(super) fn assert_runtime_13_14_review_index_anchors(sources: &RecentStaticGuardSources) {
    let runtime_13_anchors = [
        "host_function_registry_matches_documented_ledger",
        "host_capability_representatives_are_declared_on_registered_modules",
        "host_function_without_required_capability_is_rejected_with_explicit_error",
        "script_held_entity_handle_reports_invalid_after_despawn",
        "script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi",
    ];
    let runtime_14_plan_anchors = [
        "runtime_14_module_family_root_seats_match_documented_judgements",
        "runtime_navigation_boundary_file_set_requires_doc_update",
        "runtime_animation_backlog_boundary_requires_doc_update",
        "diagnostic_log_snapshot_bridge_stays_single_owner",
        "engine_module_declared_layer_does_not_own_runtime_lifecycle",
    ];
    let runtime_14_doc_anchors = [
        "runtime_14_module_family_root_seats_match_documented_judgements",
        "runtime_navigation_boundary_file_set_requires_doc_update",
        "runtime_animation_backlog_boundary_requires_doc_update",
        "diagnostic_log_snapshot_bridge_stays_single_owner",
        "engine_module_declared_layer_does_not_own_runtime_lifecycle",
    ];

    assert_contains_all("Runtime 13 subplan", &sources.archives, &runtime_13_anchors);
    assert_contains_all(
        "Runtime 13 mirror doc",
        sources.runtime_13_doc,
        &runtime_13_anchors,
    );
    assert_contains_all(
        "Runtime 14 subplan",
        &sources.archives,
        &runtime_14_plan_anchors,
    );
    assert_contains_all(
        "Runtime 14 animation doc",
        sources.runtime_14_animation_doc,
        &runtime_14_doc_anchors[2..3],
    );
    assert_contains_all(
        "Runtime 14 navigation doc",
        sources.runtime_14_navigation_doc,
        &runtime_14_doc_anchors[1..2],
    );
    assert_contains_all(
        "Runtime 14 diagnostic log doc",
        sources.runtime_14_diagnostic_doc,
        &runtime_14_doc_anchors[3..4],
    );
    assert_contains_all(
        "Runtime 14 engine module doc",
        sources.runtime_14_engine_module_doc,
        &runtime_14_doc_anchors[4..],
    );
    assert_contains_all(
        "Runtime architecture review",
        sources.review,
        &[
            "runtime_absorption/performance_hotspots.rs",
            "runtime_absorption/dynamic_api_session.rs",
            "runtime_absorption/input_stack.rs",
            "runtime_absorption/plan_status.rs",
            "runtime_absorption/ui_architecture.rs",
            "runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs",
            "runtime_01_tech_stack_cargo_gate_records_completed_dependency_validation",
            "core_spine_root_generated_boundary",
            "runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts",
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "runtime_03_schedule_frame_loop_cargo_gate_records_completed_schedule_validation",
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
            "runtime_05_closeout_status_records_completed_scene_cargo_gate",
            "runtime_02_generated_status_guard_present = true",
            "runtime_07_owner_budget_status_guard_present = true",
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
            "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
            "runtime_frame_schedule_stage.<SystemStage>",
            "runtime_09_ui_architecture_doc_records_current_boundaries",
            "runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces",
            "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
            "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
            "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending",
            "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending",
            "Runtime 10 UI Contract M2 Gate",
            "ui_v2_contract_sync_anchors = 9/9",
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "runtime_12_input_stack_contracts_stay_documented_and_exported",
            "runtime_absorption/script_host_ledger.rs",
            "host_function_registry_matches_documented_ledger",
            "runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass",
        ],
    );
    assert_contains_all(
        "Runtime plan index",
        &sources.archives,
        &[
            "runtime_01_tech_stack_cargo_gate_records_completed_dependency_validation",
            "runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
            "runtime_frame_schedule_stage.<SystemStage>",
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts",
            "runtime_03_schedule_frame_loop_cargo_gate_records_completed_schedule_validation",
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
            "runtime_05_scene_1642_structure_1304_review_298_pmrem_parity_passed_closeout_acceptance_complete",
            "runtime_02_generated_status_guard_present = true",
            "runtime_07_owner_budget_status_guard_present = true",
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
            "runtime_absorption::ui_architecture",
            "runtime_absorption::input_stack",
            "`01-tech-stack-and-dependency-governance.md`",
            "`02-core-spine-and-root-surface.md`",
            "`03-schedule-and-frame-loop-alignment.md`",
            "`04-asset-pipeline-alignment.md`",
            "`05-scene-editor-boundary-closeout.md`",
            "`06-plugin-surface-and-lifecycle.md`",
            "`07-runtime-performance-hotpath.md`",
            "`08-ecs-kernel-data-alignment.md`",
            "`09-ui-subsystem-architecture.md`",
            "`10-dynamic-api-and-interface-convergence.md`",
            "`11-job-system-task-model.md`",
            "`12-input-stack-and-action-mapping.md`",
            "`13-script-binding-and-reflection.md`",
            "`14-runtime-module-family-closeout.md`",
            "runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces",
            "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
            "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
            "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending",
            "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending",
            "UI 镜像契约 M2 owner/Cargo gate",
            "ui_v2_contract_sync_anchors = 9/9",
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "host_function_registry_matches_documented_ledger",
            "runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass",
        ],
    );
}
