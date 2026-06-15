use super::support::assert_contains_all;

#[test]
fn runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs() {
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let review =
        include_str!("../../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
    let runtime_01_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md"
    );
    let runtime_01_tech_stack_doc =
        include_str!("../../../../../docs/engine-architecture/runtime-tech-stack.md");
    let runtime_01_text_doc = include_str!("../../../../../docs/zircon_runtime/ui/text.md");
    let runtime_01_physics_doc =
        include_str!("../../../../../docs/zircon_plugins/physics-plugin-options.md");
    let runtime_01_editor_backlog_doc = include_str!(
        "../../../../../docs/editor-and-tooling/runtime-editor-only-dependency-backlog.md"
    );
    let runtime_02_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md"
    );
    let runtime_02_root_doc =
        include_str!("../../../../../docs/zircon_runtime/core/root_surface.md");
    let runtime_02_generated_doc =
        include_str!("../../../../../docs/engine-architecture/generated-code-boundary.md");
    let runtime_03_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md"
    );
    let runtime_03_frame_doc =
        include_str!("../../../../../docs/zircon_runtime/core/frame_schedule.md");
    let runtime_03_parallel_doc =
        include_str!("../../../../../docs/zircon_runtime/scene/ecs/schedule_parallel_executor.md");
    let runtime_04_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let runtime_04_facade_doc = include_str!("../../../../../docs/zircon_runtime/asset/facade.md");
    let runtime_04_worker_doc =
        include_str!("../../../../../docs/zircon_runtime/asset/worker_pool.md");
    let runtime_04_watcher_doc =
        include_str!("../../../../../docs/zircon_runtime/asset/watcher.md");
    let runtime_04_artifact_doc =
        include_str!("../../../../../docs/zircon_runtime/asset/artifact.md");
    let runtime_04_resource_doc =
        include_str!("../../../../../docs/zircon_runtime/core/resource.md");
    let runtime_05_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );
    let runtime_06_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_06_native_doc =
        include_str!("../../../../../docs/engine-architecture/native-plugin-boundary.md");
    let runtime_06_interface_doc =
        include_str!("../../../../../docs/engine-architecture/runtime-interface-convergence.md");
    let runtime_07_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_07_doc =
        include_str!("../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let runtime_08_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
    );
    let runtime_08_doc = include_str!("../../../../../docs/zircon_runtime/scene/ecs.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_09_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_10_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md"
    );
    let runtime_10_doc = include_str!("../../../../../docs/zircon_runtime/dynamic_api/session.md");
    let runtime_10_interface_doc =
        include_str!("../../../../../docs/engine-architecture/runtime-interface-convergence.md");
    let runtime_11_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/11-job-system-task-model.md"
    );
    let runtime_11_doc = include_str!("../../../../../docs/zircon_runtime/core/job_system.md");
    let runtime_12_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md"
    );
    let runtime_12_doc = include_str!("../../../../../docs/zircon_runtime/input/input_state.md");
    let runtime_13_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md"
    );
    let runtime_13_doc =
        include_str!("../../../../../docs/zircon_runtime/script/vm/host/function_ledger.md");
    let runtime_14_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md"
    );
    let runtime_14_animation_doc =
        include_str!("../../../../../docs/zircon_runtime/animation/runtime.md");
    let runtime_14_navigation_doc =
        include_str!("../../../../../docs/zircon_runtime/navigation/runtime.md");
    let runtime_14_diagnostic_doc =
        include_str!("../../../../../docs/zircon_runtime/diagnostic_log/mod.md");
    let runtime_14_engine_module_doc =
        include_str!("../../../../../docs/zircon_runtime/engine_module/relationship.md");

    let runtime_01_plan_anchors = [
        "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
        "runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate",
        "zr_vm_path_dependency_gate_is_documented_with_version_pairing",
        "plugin physics Cargo gates",
    ];
    let runtime_01_doc_anchors = [
        "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
        "runtime_text_doc_records_three_layer_stack_and_cross_reference",
        "physics_backend_option_decision_keeps_jolt_unavailable_and_plugin_owned",
        "export_archive_policy_is_documented_without_manifest_container_dependency",
        "editor_only_dependency_candidates_have_editor_backlog_owner",
    ];
    let runtime_02_plan_anchors = [
        "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
        "core_spine_root_generated_boundary",
        "runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts",
        "pre_m3_type_alias_guard_static_passed_pending_render_owner",
        "core/root/generated/export_build_plan/app/editor/plugin",
    ];
    let runtime_02_doc_anchors = [
        "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
        "pre_m3_type_alias_guard_static_passed_pending_render_owner",
        "generated_code_boundary.m1_gate_status",
        "classified-and-clear",
    ];
    let runtime_03_anchors = [
        "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
        "ecs_schedule/time/session/schedule_parallel",
        "RuntimeTimeAdvance",
        "fixed_step_plan_reports_overstep_fraction_in_unit_range",
        "ScheduleParallelExecutionReport",
    ];
    let runtime_04_anchors = [
        "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
        "AssetWorkerPoolOptions",
        "asset_worker_pool_matches_runtime_04_and_11_decisions",
        "artifact_store_roundtrips_scene_assets_with",
        "watcher",
    ];
    let runtime_05_anchors = [
        "runtime_05_closeout_status_waits_for_full_scene_cargo_gate",
        "pending_full_scene_cargo",
        "cargo test -p zircon_runtime --lib scene:: --locked",
        "runtime_02_generated_status_guard_present = true",
        "runtime_07_owner_budget_status_guard_present = true",
    ];
    let runtime_06_anchors = [
        "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
        "script::vm/vampire_project_session/plugin/native_plugin/app/plugins",
        "native_plugin_public_surface",
        "root_reexport_count = 70",
        "plugin_surface_lifecycle_boundary",
        "runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts",
    ];
    let runtime_07_plan_anchors = [
        "runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
        "frame_spans_static_passed_trace_pending",
        "runtime_frame_schedule_stage.<SystemStage>",
        "SceneScheduleRunner",
    ];
    let runtime_07_doc_anchors = [
        "runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
        "runtime_frame_schedule_stage",
        "SceneScheduleRunner",
        "stage-level span",
    ];
    let runtime_08_anchors = [
        "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
        "despawned_entity_handle_is_rejected_by_world_access",
        "lifecycle_observer_fires_immediately_during_component_mutation",
        "command_queue_on_despawned_entity_target_is_reported_not_silently_dropped",
        "events_require_explicit_update_and_keep_next_queue_hidden",
        "change_tick_comparison_survives_wraparound",
    ];
    let runtime_09_anchors = [
        "runtime_09_ui_architecture_doc_records_current_boundaries",
        "runtime_09_ui_architecture_baselines_match_current_source_scan",
        "runtime_09_v2_verdict_matches_runtime_and_interface_modules",
        "runtime_09_m0_ui_architecture_static_passed",
        "v2-replacement-mainline",
    ];
    let runtime_10_plan_anchors = [
        "runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces",
        "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
        "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
        "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
        "M2 UI 镜像契约 pending gate",
        "Runtime 09/editor UI owner",
        "minimal`/`headless` profile 通过 `uses_render_bridge()` 跳过 render bridge",
        "capture 返回空帧",
        "bind/unbind/present 为 no-op",
    ];
    let runtime_10_doc_anchors = [
        "runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces",
        "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
        "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
        "minimal` and `headless` profiles now skip `RuntimeRenderBridge` creation",
        "frame capture returns an empty encoded frame",
        "surface bind/unbind/present operations are no-ops",
    ];
    let runtime_10_interface_anchors = [
        "Runtime 10 UI Contract M2 Gate",
        "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
        "Runtime 09/editor UI owner",
        "`interface/ui`",
        "`runtime/ui`",
    ];
    let runtime_11_anchors = [
        "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
        "tasks/ecs_schedule/worker_pool/rayon",
        "parallel_frustum.rs",
    ];
    let runtime_12_anchors = [
        "runtime_12_input_stack_contracts_stay_documented_and_exported",
        "runtime_12_action_mapping_keeps_ui_filtered_evaluation_path",
        "runtime_12_gamepad_bridge_keeps_runtime_abi_path",
    ];
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

    assert_contains_all(
        "Runtime 01 subplan",
        runtime_01_plan,
        &runtime_01_plan_anchors,
    );
    assert_contains_all(
        "Runtime 01 tech-stack doc",
        runtime_01_tech_stack_doc,
        &[
            runtime_01_doc_anchors[0],
            runtime_01_doc_anchors[3],
            runtime_01_doc_anchors[4],
        ],
    );
    assert_contains_all(
        "Runtime 01 text doc",
        runtime_01_text_doc,
        &[
            "Backend Responsibility Matrix",
            runtime_01_doc_anchors[1],
            "text_shaper_stack_uses_current_heuristic_backend_until_font_backends_land",
        ],
    );
    assert_contains_all(
        "Runtime 01 physics doc",
        runtime_01_physics_doc,
        &[
            runtime_01_doc_anchors[0],
            runtime_01_doc_anchors[2],
            "only executable V1 backend",
        ],
    );
    assert_contains_all(
        "Runtime 01 editor-only backlog doc",
        runtime_01_editor_backlog_doc,
        &[runtime_01_doc_anchors[4], "rfd", "arboard"],
    );
    assert_contains_all(
        "Runtime 02 subplan",
        runtime_02_plan,
        &runtime_02_plan_anchors,
    );
    assert_contains_all(
        "Runtime 02 root surface doc",
        runtime_02_root_doc,
        &runtime_02_doc_anchors[..2],
    );
    assert_contains_all(
        "Runtime 02 generated boundary doc",
        runtime_02_generated_doc,
        &runtime_02_doc_anchors[0..1],
    );
    assert_contains_all(
        "Runtime 02 generated boundary status doc",
        runtime_02_generated_doc,
        &runtime_02_doc_anchors[2..],
    );
    assert_contains_all("Runtime 03 subplan", runtime_03_plan, &runtime_03_anchors);
    assert_contains_all(
        "Runtime 03 frame schedule doc",
        runtime_03_frame_doc,
        &runtime_03_anchors[2..],
    );
    assert_contains_all(
        "Runtime 03 schedule parallel doc",
        runtime_03_parallel_doc,
        &[
            runtime_03_anchors[4],
            "schedule_parallel_batches_chain_through_job_handles",
        ],
    );
    assert_contains_all("Runtime 04 subplan", runtime_04_plan, &runtime_04_anchors);
    assert_contains_all(
        "Runtime 04 asset facade doc",
        runtime_04_facade_doc,
        &[
            "dangling_handle_queries_report_not_loaded_instead_of_panicking",
            "failed_asset_exposes_failure_reason_through_facade",
        ],
    );
    assert_contains_all(
        "Runtime 04 asset worker doc",
        runtime_04_worker_doc,
        &runtime_04_anchors[1..3],
    );
    assert_contains_all(
        "Runtime 04 asset watcher doc",
        runtime_04_watcher_doc,
        &["Asset Watcher", "Reloading", "watcher"],
    );
    assert_contains_all(
        "Runtime 04 asset artifact doc",
        runtime_04_artifact_doc,
        &[runtime_04_anchors[3], "cache-wire boundary problem"],
    );
    assert_contains_all(
        "Runtime 04 resource doc",
        runtime_04_resource_doc,
        &["ResourceRecord", "failure_reason", "Reloading"],
    );
    assert_contains_all("Runtime 05 subplan", runtime_05_plan, &runtime_05_anchors);
    assert_contains_all("Runtime 06 subplan", runtime_06_plan, &runtime_06_anchors);
    assert_contains_all(
        "Runtime 06 native plugin doc",
        runtime_06_native_doc,
        &runtime_06_anchors,
    );
    assert_contains_all(
        "Runtime 06 interface convergence doc",
        runtime_06_interface_doc,
        &runtime_06_anchors[..3],
    );
    assert_contains_all(
        "Runtime 07 subplan",
        runtime_07_plan,
        &runtime_07_plan_anchors,
    );
    assert_contains_all(
        "Runtime 07 mirror doc",
        runtime_07_doc,
        &runtime_07_doc_anchors,
    );
    assert_contains_all("Runtime 08 subplan", runtime_08_plan, &runtime_08_anchors);
    assert_contains_all("Runtime 08 ECS doc", runtime_08_doc, &runtime_08_anchors);
    assert_contains_all("Runtime 09 subplan", runtime_09_plan, &runtime_09_anchors);
    assert_contains_all("Runtime 09 mirror doc", runtime_09_doc, &runtime_09_anchors);
    assert_contains_all(
        "Runtime 10 subplan",
        runtime_10_plan,
        &runtime_10_plan_anchors,
    );
    assert_contains_all(
        "Runtime 10 mirror doc",
        runtime_10_doc,
        &runtime_10_doc_anchors,
    );
    assert_contains_all(
        "Runtime 10 interface convergence doc",
        runtime_10_interface_doc,
        &runtime_10_interface_anchors,
    );
    assert_contains_all("Runtime 11 subplan", runtime_11_plan, &runtime_11_anchors);
    assert_contains_all("Runtime 11 mirror doc", runtime_11_doc, &runtime_11_anchors);
    assert_contains_all("Runtime 12 subplan", runtime_12_plan, &runtime_12_anchors);
    assert_contains_all("Runtime 12 mirror doc", runtime_12_doc, &runtime_12_anchors);
    assert_contains_all("Runtime 13 subplan", runtime_13_plan, &runtime_13_anchors);
    assert_contains_all("Runtime 13 mirror doc", runtime_13_doc, &runtime_13_anchors);
    assert_contains_all(
        "Runtime 14 subplan",
        runtime_14_plan,
        &runtime_14_plan_anchors,
    );
    assert_contains_all(
        "Runtime 14 animation doc",
        runtime_14_animation_doc,
        &runtime_14_doc_anchors[2..3],
    );
    assert_contains_all(
        "Runtime 14 navigation doc",
        runtime_14_navigation_doc,
        &runtime_14_doc_anchors[1..2],
    );
    assert_contains_all(
        "Runtime 14 diagnostic log doc",
        runtime_14_diagnostic_doc,
        &runtime_14_doc_anchors[3..4],
    );
    assert_contains_all(
        "Runtime 14 engine module doc",
        runtime_14_engine_module_doc,
        &runtime_14_doc_anchors[4..],
    );
    assert_contains_all(
        "Runtime architecture review",
        review,
        &[
            "runtime_absorption/performance_hotspots.rs",
            "runtime_absorption/dynamic_api_session.rs",
            "runtime_absorption/input_stack.rs",
            "runtime_absorption/plan_status.rs",
            "runtime_absorption/ui_architecture.rs",
            "runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs",
            "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
            "core_spine_root_generated_boundary",
            "runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts",
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
            "runtime_05_closeout_status_waits_for_full_scene_cargo_gate",
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
            "Runtime 10 UI Contract M2 Gate",
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "runtime_12_input_stack_contracts_stay_documented_and_exported",
            "runtime_absorption/script_host_ledger.rs",
            "host_function_registry_matches_documented_ledger",
            "runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass",
        ],
    );
    assert_contains_all(
        "Runtime plan index",
        runtime_index,
        &[
            "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
            "runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
            "runtime_frame_schedule_stage.<SystemStage>",
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts",
            "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
            "runtime_05_closeout_status_waits_for_full_scene_cargo_gate",
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
            "UI 镜像契约 M2 owner/Cargo gate",
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "host_function_registry_matches_documented_ledger",
            "runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass",
        ],
    );
}
