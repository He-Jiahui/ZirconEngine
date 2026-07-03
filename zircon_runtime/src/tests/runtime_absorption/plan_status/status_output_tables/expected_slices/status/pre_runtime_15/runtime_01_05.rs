pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 05 plan-status Cargo attempt 状态审计" {
        Some("cargo_attempt_status_static_passed_cargo_pending")
    } else if slice == "Runtime 05 plan-status Cargo timeout 状态审计" {
        Some("cargo_attempt_timeout_status_static_passed_cargo_pending")
    } else if slice == "Runtime 01 export build-plan directory materialization boundary" {
        Some("export_materialize_owner_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 01 NativeDynamic materialization symlink boundary" {
        Some("export_materialize_symlink_boundary_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 01 export materialization dry-run preview" {
        Some("export_materialize_preview_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 01 export materialization fatal preflight gate" {
        Some("export_materialize_fatal_gate_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 01 editor native-aware fatal export early exit" {
        Some("editor_native_aware_export_fatal_gate_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 01 editor native-aware discovery reuse" {
        Some("editor_native_aware_export_discovery_reuse_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 01 export ZIP archive materialization" {
        Some("export_archive_zip_materialization_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 01 Tech-stack current audit recheck" {
        Some("tech_stack_current_audit_static_passed_cargo_pending")
    } else if slice == "Runtime 01 Tech-stack 2026-07-01 current audit recheck" {
        Some("tech_stack_20260701_current_audit_static_passed_cargo_deferred")
    } else if slice == "Runtime 01 Tech-stack inventory split" {
        Some("tech_stack_inventory_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 01 Tech-stack Markdown renderer split" {
        Some("tech_stack_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 01 Tech-stack SharedTextService 锚点同步" {
        Some("tech_stack_shared_text_service_anchor_sync_static_passed_cargo_deferred")
    } else if slice == "Runtime 05 full scene closeout failed evidence" {
        Some("cargo_recheck_failed_full_scene_gate")
    } else if slice == "Runtime 05 full scene compile-pass graphics-scene blocker" {
        Some("runtime_05_full_scene_gate_compile_passed_graphics_scene_tests_failed_scene_gate_pending")
    } else if slice == "Runtime 05 full scene closeout no-result recheck" {
        Some("cargo_recheck_no_result_external_editor_lane")
    } else if slice == "Runtime 05 scene:: failure support-first triage" {
        Some("support_first_triage_static_passed_cargo_pending")
    } else if slice == "Runtime 05 scene:: lower-layer diagnostic matrix" {
        Some("support_first_matrix_static_passed_cargo_pending")
    } else if slice == "Runtime 05 scene:: diagnostic matrix source anchors" {
        Some("support_first_matrix_source_anchors_static_passed_cargo_pending")
    } else if slice == "Runtime 05 render product streamer 2026-06-21 no-result diagnostic" {
        Some("runtime_05_render_product_streamer_20260621_no_result_residual_stopped")
    } else if slice == "Runtime 05 scene_asset 2026-06-21 no-result diagnostic" {
        Some("runtime_05_scene_asset_20260621_no_result_residual_stopped")
    } else if slice == "Runtime 05 ecs_query 2026-06-21 no-result diagnostic" {
        Some("runtime_05_ecs_query_20260621_no_result_residual_stopped")
    } else if slice == "Runtime 05 serialization source folder-split guard sync" {
        Some("source_guard_static_passed_cargo_deferred_active_lanes")
    } else if slice == "Runtime 05 scene/project serialization Markdown renderer split" {
        Some("scene_project_serialization_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 05 scene/editor surface Markdown renderer split" {
        Some("runtime_scene_editor_surface_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 05 non-network server Markdown renderer split" {
        Some("non_network_server_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 05 runtime naming Markdown renderer split" {
        Some("runtime_naming_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 05 hard-cutover migration-smell Markdown renderer split" {
        Some("hard_cutover_migration_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime M0 entry static dependencies Markdown renderer split" {
        Some("entry_static_dependencies_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime M0 legacy standalone references Markdown renderer split" {
        Some("legacy_standalone_references_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime M0 module inventory Markdown renderer split" {
        Some("module_inventory_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime M0 plugin runtime gaps Markdown renderer split" {
        Some("plugin_runtime_gaps_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime M0 large-file ownership Markdown renderer split" {
        Some("large_file_ownership_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 05 editor_projection residual guard verdict" {
        Some("editor_projection_residual_guard_static_passed_cargo_pending")
    } else if slice == "Runtime 05 dynamic scene session owner-tree guard" {
        Some("dynamic_scene_session_owner_tree_guard_static_passed_cargo_pending")
    } else if slice == "Runtime 05 dynamic scene root owner-tree guard" {
        Some("dynamic_scene_root_owner_tree_guard_static_passed_cargo_pending")
    } else if slice == "Runtime 05 dynamic scene root scene owner split" {
        Some("dynamic_scene_scene_owner_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 05 dynamic scene spawn task owner split" {
        Some("dynamic_scene_spawn_task_owner_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 05 dynamic scene value conversion owner split" {
        Some("dynamic_scene_value_owner_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 05 dynamic scene entity declaration owner split" {
        Some("dynamic_scene_entity_owner_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 05 dynamic scene scene-asset bridge owner split" {
        Some("dynamic_scene_scene_asset_bridge_owner_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 05 dynamic scene document serialization owner split" {
        Some("dynamic_scene_document_owner_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 05 dynamic scene patch preview API" {
        Some("dynamic_scene_patch_preview_api_static_passed_cargo_timeout_no_result_tests_deferred")
    } else if slice == "Runtime 05 dynamic scene patch preview status guard" {
        Some("dynamic_scene_patch_preview_status_guard_static_passed_cargo_pending")
    } else if slice
        == "Runtime 05 dynamic scene patch preview resource preflight details status guard"
    {
        Some("dynamic_scene_patch_preview_resource_preflight_details_status_guard_static_passed_cargo_pending")
    } else if slice
        == "Runtime 05 dynamic scene patch preview resource ensure creation status guard"
    {
        Some("dynamic_scene_patch_preview_resource_ensure_creation_status_guard_static_passed_cargo_pending")
    } else if slice
        == "Runtime 05 dynamic scene patch preview component type install details status guard"
    {
        Some("dynamic_scene_patch_preview_component_type_install_details_status_guard_static_passed_cargo_pending")
    } else if slice
        == "Runtime 05 dynamic scene patch preview component type install counts status guard"
    {
        Some("dynamic_scene_patch_preview_component_type_install_counts_status_guard_static_passed_cargo_pending")
    } else if slice == "Runtime 05 dynamic scene patch preview reflection preflight status guard" {
        Some("dynamic_scene_patch_preview_reflection_preflight_status_guard_static_passed_cargo_pending")
    } else if slice == "Runtime 05 dynamic scene patch preview component workload status guard" {
        Some("dynamic_scene_patch_preview_component_workload_status_guard_static_passed_cargo_pending")
    } else if slice == "Runtime 05 dynamic scene patch preview remap status guard" {
        Some("dynamic_scene_patch_preview_remap_status_guard_static_passed_cargo_pending")
    } else if slice == "Runtime 03 world bootstrap fixed-loop stage guard sync" {
        Some("guard_sync_static_passed_cargo_pending")
    } else if slice == "Runtime 03 Schedule/frame-loop current audit recheck" {
        Some("schedule_frame_loop_current_audit_static_passed_cargo_pending")
    } else if slice == "Runtime 03 Schedule/frame-loop 2026-07-01 current audit recheck" {
        Some("schedule_frame_loop_20260701_current_audit_static_passed_cargo_deferred")
    } else if slice == "Runtime 03 Schedule/frame-loop inventory split" {
        Some("schedule_frame_loop_inventory_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 03 Schedule/frame-loop markdown renderer split" {
        Some("schedule_frame_loop_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 03 Schedule/frame-loop session profile owner audit sync" {
        Some("schedule_frame_loop_session_profile_owner_audit_sync_static_passed_cargo_deferred")
    } else if slice == "Runtime 04 worker-pool manager frame sampler entry"
        || slice == "Runtime 07 asset worker manager sampler entry"
    {
        Some("asset_worker_manager_sampler_static_passed_cargo_deferred")
    } else if slice == "Runtime 04 asset worker request entry hard-cutover" {
        Some("asset_worker_request_sender_hard_cutover_static_passed_cargo_deferred")
    } else if slice == "Runtime 04 Asset pipeline current audit recheck" {
        Some("asset_pipeline_current_audit_static_passed_cargo_pending")
    } else if slice == "Runtime 04 Asset pipeline 2026-07-01 current audit recheck" {
        Some("asset_pipeline_20260701_current_audit_static_passed_cargo_deferred")
    } else if slice == "Runtime 04 artifact-store child owner audit sync" {
        Some("runtime_04_artifact_store_child_owner_audit_sync_static_passed_cargo_deferred")
    } else if slice == "Runtime 04 Asset pipeline inventory split" {
        Some("asset_pipeline_inventory_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 04 Asset pipeline Markdown renderer split" {
        Some("asset_pipeline_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 04 F7 asset artifact/importer typed errors" {
        Some("asset_artifact_importer_typed_errors_coremin_passed")
    } else if slice == "Runtime 04 F8 texture import settings apply API" {
        Some("texture_import_settings_apply_api_coremin_check_passed")
    } else if slice == "Runtime 02 generated template count 审计同步" {
        Some("structure_audit_static_passed_cargo_pending")
    } else if slice == "Runtime 02 root graphics alias block removal" {
        Some("graphics_alias_block_removed_static_passed_cargo_pending")
    } else if slice == "Runtime 02 rhi_wgpu root backend private cutover" {
        Some("rhi_wgpu_root_backend_private_static_passed_cargo_pending")
    } else if slice == "Runtime 02 builtin root facade cutover" {
        Some("builtin_root_facade_removed_static_passed_cargo_pending")
    } else if slice == "Runtime 02 core/root/generated current audit recheck" {
        Some("core_root_generated_current_audit_static_passed_cargo_pending")
    } else if slice == "Runtime 02 core/root/generated 2026-07-01 current audit recheck" {
        Some("core_root_generated_20260701_current_audit_static_passed_cargo_deferred")
    } else if slice == "Runtime 02 core/root/generated Markdown renderer split" {
        Some("core_root_generated_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 02 root-surface Markdown renderer split" {
        Some("runtime_root_surface_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 02 F6 core resource registry typed errors" {
        Some("core_resource_registry_typed_errors_coremin_check_passed")
    } else if slice == "Runtime 02 generated-code Markdown renderer split" {
        Some("generated_code_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if matches!(
        slice,
        Some("Runtime 05 recent-static Runtime 02/07 status metadata guard")
            | "Runtime 05 status-output recent-static metadata row"
            | "Runtime 05 status-output Runtime 12 gamepad event-owner row"
            | "Runtime 05 status-output Runtime 12 behavior-test row"
            | "Runtime 05 status-output Runtime 04 behavior-test row"
            | "Runtime 05 status-output Runtime 08 behavior-test row"
            | "Runtime 05 status-output Runtime 10 behavior-test row"
            | "Runtime 05 plan-status output-anchor module split"
            | "Runtime 05 plan-status output-anchor budget guard"
            | "Runtime 05 status-output status/date helper split"
            | "Runtime 05 status-output expected anchor split"
            | "Runtime 05 plan-status root module split"
            | "Runtime 05 plan-status support inventory split"
            | "Runtime 05 plan-status anchor inventory split"
            | "Runtime 05 plan-status markdown renderer split"
            | "Runtime 05 plan-status markdown direct import hard-cutover"
            | "Runtime 05 plan-status source helper split"
            | "Runtime 05 status-output expected row data split"
            | "Runtime 05 status-output Runtime 05 row-data family split"
            | "Runtime 05 status-output audit-metadata owner split"
            | "Runtime 05 status-output Runtime 14 row-data family split"
            | "Runtime 05 status-output Runtime 07 row-data family split"
            | "Runtime 05 status-output Runtime 09 row-data family split"
            | "Runtime 05 status-output Runtime 10 row-data family split"
            | "Runtime 05 status-output Runtime 12 row-data family split"
            | "Runtime 05 status-output support-structure owner split"
            | "Runtime 05 status-output scene-closeout owner split"
            | "Runtime 05 status-output cargo-gates owner split"
            | "Runtime 05 status-output status/date owner split"
            | "Runtime 05 cargo-gates early Runtime 03 split"
            | "Runtime 05 cargo-gates early Runtime 01 split"
            | "Runtime 05 cargo-gates early Runtime 02 split"
            | "Runtime 05 cargo-gates early Runtime 04 split"
            | "Runtime 05 cargo-gates early Runtime 06 split"
            | "Runtime 05 cargo-gates early Runtime 08 split"
            | "Runtime 05 cargo-gates early Runtime 07 split"
            | "Runtime 05 cargo-gates late Runtime 10 split"
            | "Runtime 05 cargo-gates late Runtime 11 split"
            | "Runtime 05 cargo-gates late Runtime 12 split"
            | "Runtime 05 cargo-gates late Runtime 13 split"
            | "Runtime 05 cargo-gates late Runtime 14 split"
            | "Runtime 05 status-output Runtime 01-04 row-data group split"
            | "Runtime 05 status-output Runtime 06-09 row-data group split"
            | "Runtime 05 status-output Runtime 10-13 row-data group split"
            | "Runtime 05 plan-status 输出表守卫"
            | "Runtime 05 plan-status 审计元数据守卫"
            | "Runtime 05 status-output Runtime 07 scene asset rows"
            | "Runtime 05 Runtime 07 scene status 审计元数据"
            | "Runtime 05 status-output Runtime 02 generated template row"
            | "Runtime 05 Runtime 02 generated status 审计元数据"
            | "Runtime 05 status-output current anchor fix"
            | "Runtime 05 Runtime 02 root_entries count 状态表闭环"
            | "Runtime 05 status-output Runtime 07 owner-budget row"
            | "Runtime 05 plan-status owner-budget current mirror fix"
            | "Runtime 05 Runtime 07 owner-budget status 审计元数据"
            | "Runtime 05 status-output Runtime 03 module-doc row"
            | "Runtime 05 status-output Runtime 03 behavior-test row"
            | "Runtime 05 status-output all-index-row coverage guard"
            | "Runtime 05 status-output non-network server allowlist row"
    ) {
        Some("status_table_static_passed_cargo_pending")
    } else if slice == "Runtime 05 M0 absorption guard coverage sync" {
        Some("static_docs_passed_cargo_pending")
    } else if slice == "Runtime 05 non-network server UI sortingMode allowlist" {
        Some("static_audit_passed_cargo_pending")
    } else if slice == "Runtime 05 naming_boundary non-network server Rust guard" {
        Some("naming_guard_static_passed_cargo_pending")
    } else if slice == "Runtime 05 texture importer DDS caps policy wording" {
        Some("hard_cutover_dds_caps_policy_static_passed_cargo_pending")
    } else if slice == "Runtime 05 status-output row-data group split" {
        Some("status_table_static_passed_cargo_pending")
    } else {
        None
    }
}
