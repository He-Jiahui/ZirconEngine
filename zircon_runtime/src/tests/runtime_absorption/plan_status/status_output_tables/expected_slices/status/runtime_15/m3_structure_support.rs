#[path = "m3_structure_support/naming_guard_maps.rs"]
mod naming_guard_maps;
#[path = "m3_structure_support/review_guard_maps.rs"]
mod review_guard_maps;
#[path = "m3_structure_support/status_support_maps.rs"]
mod status_support_maps;

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(status) = review_guard_maps::expected_status_for_slice(slice)
        .or_else(|| naming_guard_maps::expected_status_for_slice(slice))
        .or_else(|| status_support_maps::expected_status_for_slice(slice))
    {
        return Some(status);
    }

    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/dead_code_surface.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/runtime_structure_tests.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/plugin_importer_review.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/plugin_importer_migrations.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/runtime_absorption_followups.rs
    // runtime_15_foundation_guards_row_data_owner_is_child_backed
    if slice == "Runtime 15 M3 foundation-guards row-data owner child split" {
        Some("runtime_15_foundation_guards_row_data_owner_child_split_static_passed_cargo_deferred")
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/script_vm_runtime.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/plugin_extension_tests.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/script_vm_gameplay_shader.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/scene_ecs_tests.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/scene_asset_world.rs
    // runtime_15_scene_script_row_data_owner_is_child_backed
    } else if slice == "Runtime 15 M3 scene-script row-data owner child split" {
        Some("runtime_15_scene_script_row_data_owner_child_split_static_passed_cargo_deferred")
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/status_rows.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/policy_guards.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/core_runtime_recovery.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/runtime_services_recovery.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/resource_render_input_recovery.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/script_vm_recovery.rs
    // runtime_15_lock_poison_status_row_data_owner_is_child_backed
    } else if slice == "Runtime 15 M3 lock-poison status row-data owner child split" {
        Some(
            "runtime_15_lock_poison_status_row_data_owner_child_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 graphics dead-code guard module split" {
        Some("runtime_15_graphics_dead_code_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 graphics dead-code guard child-owner split" {
        Some("runtime_15_graphics_dead_code_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 graphics dead-code guard forbidden attribute literal cleanup"
    {
        Some("runtime_15_graphics_dead_code_guard_literal_cleanup_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 provider boilerplate guard module split" {
        Some("runtime_15_provider_boilerplate_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 provider boilerplate guard child-owner split" {
        Some("runtime_15_provider_boilerplate_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 facade surface guard module split" {
        Some("runtime_15_facade_surface_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 runtime dead-code guard module split" {
        Some("runtime_15_runtime_dead_code_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 runtime dead-code guard forbidden attribute literal cleanup" {
        Some("runtime_15_runtime_dead_code_guard_literal_cleanup_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 runtime dead-code guard child-owner split" {
        Some("runtime_15_runtime_dead_code_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 runtime dead-code documentation anchor cleanup" {
        Some(
            "runtime_15_runtime_dead_code_documentation_anchor_cleanup_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 runtime dead-code module-gate status wording cleanup" {
        Some("runtime_15_runtime_dead_code_module_gate_status_wording_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 runtime dead-code production-gate status wording cleanup" {
        Some(
            "runtime_15_runtime_dead_code_production_gate_status_wording_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 diagnostics guard module split" {
        Some("runtime_15_diagnostics_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 core framework test folder split" {
        Some("runtime_15_core_framework_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 picking test folder split" {
        Some("runtime_15_picking_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 core runtime deactivation blocked test folder split" {
        Some(
            "runtime_15_core_runtime_deactivation_blocked_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 lock-poison status row-data child-owner split" {
        Some(
            "runtime_15_lock_poison_status_row_data_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 asset/render/input lock-poison guard child-owner split" {
        Some(
            "runtime_15_asset_render_input_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 runtime services lock-poison guard child-owner split" {
        Some(
            "runtime_15_runtime_services_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 module-convention status row-data child-owner split" {
        Some(
            "runtime_15_module_convention_status_row_data_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 module convention module-doc frontmatter uniqueness guard" {
        Some(
            "runtime_15_module_convention_module_doc_frontmatter_uniqueness_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 module convention gate output contract" {
        Some("runtime_15_module_convention_gate_output_contract_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 module convention non-render debt guard" {
        Some("runtime_15_module_convention_non_render_debt_guard_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 render-scoped migration debt handoff gate" {
        Some("runtime_15_render_scoped_migration_debt_handoff_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 hard-cutover allowed Hyper policy risk cleanup" {
        Some(
            "runtime_15_hard_cutover_allowed_hyper_policy_risk_cleanup_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 module convention gate audit-clear status mirror" {
        Some(
            "runtime_15_module_convention_gate_audit_clear_status_mirror_core_min_cargo_passed_full_sweep_pending",
        )
    } else if slice == "Runtime 15 M3 module convention zero-debt revalidation" {
        Some(
            "runtime_15_module_convention_zero_debt_revalidation_static_passed_cargo_timeout_no_result",
        )
    } else if slice == "Runtime 15 M3 module convention audit script family naming cleanup" {
        Some(
            "runtime_15_module_convention_audit_script_family_naming_core_min_cargo_passed_full_sweep_pending",
        )
    } else if slice == "Runtime 15 M3 dynamic scene absorption guard folder split" {
        Some("runtime_15_dynamic_scene_absorption_guard_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 input manager test folder split" {
        Some("runtime_15_input_manager_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI architecture test folder split" {
        Some("runtime_15_ui_architecture_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI v2 asset test folder split" {
        Some("runtime_15_ui_v2_asset_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 UI shared core test folder split" {
        Some("runtime_15_ui_shared_core_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 UI shared core guard child-owner split" {
        Some("runtime_15_ui_shared_core_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI shared core input visibility child folder split" {
        Some(
            "runtime_15_ui_shared_core_input_visibility_child_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI shared core scroll mutation child folder split" {
        Some(
            "runtime_15_ui_shared_core_scroll_mutation_child_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI shared core layout surface child folder split" {
        Some(
            "runtime_15_ui_shared_core_layout_surface_child_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI accessibility test folder split" {
        Some("runtime_15_ui_accessibility_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI accessibility widget actions test folder split" {
        Some(
            "runtime_15_ui_accessibility_widget_actions_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI layout slots test folder split" {
        Some("runtime_15_ui_layout_slots_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI surface-frame authority test folder split" {
        Some(
            "runtime_15_ui_surface_frame_authority_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI surface dirty domains test folder split" {
        Some("runtime_15_ui_surface_dirty_domains_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI material layout test folder split" {
        Some("runtime_15_ui_material_layout_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI template test folder split" {
        Some("runtime_15_ui_template_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI component catalog test folder split" {
        Some("runtime_15_ui_component_catalog_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI boundary test folder split" {
        Some("runtime_15_ui_boundary_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI boundary ZUI surface projection guard sync" {
        Some(
            "runtime_15_ui_boundary_zui_surface_projection_guard_sync_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI component state test folder split" {
        Some(
            "runtime_15_ui_component_catalog_component_state_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI component state keyboard test folder split" {
        Some(
            "runtime_15_ui_component_catalog_component_state_keyboard_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI Material foundation test folder split" {
        Some(
            "runtime_15_ui_component_catalog_material_foundation_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI event routing test folder split" {
        Some("runtime_15_ui_event_routing_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime input reply routes test folder split" {
        Some(
            "runtime_15_ui_runtime_input_reply_routes_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI runtime input reply route child folder split" {
        Some(
            "runtime_15_ui_runtime_input_reply_route_children_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI runtime input reply table pointer route folder split" {
        Some(
            "runtime_15_ui_runtime_input_reply_table_pointer_routes_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI runtime input reply route guard child-owner split" {
        Some(
            "runtime_15_ui_runtime_input_reply_route_guard_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 runtime diagnostics test folder split" {
        Some("runtime_15_runtime_diagnostics_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 RHI command list test folder split" {
        Some("runtime_15_rhi_command_list_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 RHI device contract test folder split" {
        Some("runtime_15_rhi_device_contract_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset pack test folder split" {
        Some("runtime_15_asset_pack_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset facade test folder split" {
        Some("runtime_15_asset_facade_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset project zmeta test folder split" {
        Some("runtime_15_asset_project_zmeta_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset project zmeta current 12-test guard sync" {
        Some(
            "runtime_15_asset_project_zmeta_current_12_test_guard_sync_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 asset project manager test folder split" {
        Some("runtime_15_asset_project_manager_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset project manager current 11-test guard sync" {
        Some(
            "runtime_15_asset_project_manager_current_11_test_guard_sync_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 asset project flow sample test folder split" {
        Some(
            "runtime_15_asset_project_flow_sample_tests_folder_split_static_passed_cargo_lock_blocked",
        )
    } else if slice == "Runtime 15 M3 asset project example vampire test folder split" {
        Some(
            "runtime_15_asset_project_example_vampire_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 asset artifact store test folder split" {
        Some("runtime_15_asset_artifact_store_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset material test folder split" {
        Some("runtime_15_asset_material_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset mesh test root split" {
        Some("runtime_15_asset_mesh_tests_root_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset glTF importer test folder split" {
        Some("runtime_15_asset_gltf_importer_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset glTF primitive fixture folder split" {
        Some(
            "runtime_15_asset_gltf_primitive_fixtures_folder_split_static_passed_cargo_lock_blocked",
        )
    } else if slice == "Runtime 15 M3 asset importer test folder split" {
        Some("runtime_15_asset_importer_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset scene test folder split" {
        Some("runtime_15_asset_scene_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset UI test folder split" {
        Some("runtime_15_asset_ui_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset pipeline manager test folder split" {
        Some("runtime_15_asset_pipeline_manager_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 test file budget guard folder split" {
        Some("runtime_15_test_file_budget_guard_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 test file budget guard root mod cutover" {
        Some("runtime_15_test_file_budget_guard_root_mod_cutover_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 no oversized test files global gate" {
        Some("runtime_15_no_oversized_test_files_global_gate_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 render product mesh-cache morph tests child-owner split" {
        Some(
            "runtime_15_render_product_mesh_cache_morph_tests_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI text layout folder-backed owner split" {
        Some("runtime_15_ui_text_layout_folder_backed_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 Runtime 07 performance hotspot guard folder split" {
        Some(
            "runtime_15_runtime_07_performance_hotspots_guard_folder_split_static_passed_cargo_timeout_no_result",
        )
    } else if slice
        == "Runtime 15 M3 Runtime 07 owner-budget virtual-geometry guard child-owner split"
    {
        Some(
            "runtime_15_runtime_07_owner_budget_virtual_geometry_guard_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 Runtime 07 owner-budget large-file gate child-owner split" {
        Some(
            "runtime_15_runtime_07_owner_budget_large_file_gate_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 Runtime 07 owner-budget mirror-docs child-owner split" {
        Some(
            "runtime_15_runtime_07_owner_budget_mirror_docs_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 script VM test folder split" {
        Some("runtime_15_script_vm_tests_folder_split_static_passed_cargo_timeout_no_result")
    } else if slice == "Runtime 15 M3 script VM primary guard child-owner split" {
        Some("runtime_15_script_vm_primary_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 script VM hot-reload coordinator test folder split" {
        Some(
            "runtime_15_script_vm_hot_reload_coordinator_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 script VM hot-reload guard child-owner split" {
        Some("runtime_15_script_vm_hot_reload_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 native live-host tests folder split" {
        Some("runtime_15_native_live_host_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 native plugin loader real fixture test folder split" {
        Some(
            "runtime_15_native_plugin_loader_real_fixture_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 extension registry bridge test folder split" {
        Some("runtime_15_extension_registry_bridge_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 manifest contributions test folder split" {
        Some("runtime_15_manifest_contributions_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 manifest contributions runtime-family test child-owner split"
    {
        Some(
            "runtime_15_manifest_contributions_runtime_family_tests_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 runtime plugin package manifest test folder split" {
        Some(
            "runtime_15_runtime_plugin_package_manifest_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M3 runtime plugin package manifest capability-status test child-owner split"
    {
        Some(
            "runtime_15_runtime_plugin_package_manifest_capability_status_tests_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M3 runtime plugin catalog feature-dependency report test child-owner split"
    {
        Some(
            "runtime_15_runtime_plugin_catalog_features_dependency_report_tests_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 runtime plugin lifecycle fixture child-owner split" {
        Some(
            "runtime_15_runtime_plugin_lifecycle_fixture_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 export build plan test folder split" {
        Some("runtime_15_export_build_plan_tests_folder_split_static_passed_cargo_deferred")
    } else if slice
        == "Runtime 15 M3 export build plan profile feature matrix test child-owner split"
    {
        Some(
            "runtime_15_export_build_plan_profile_feature_matrix_tests_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 export build plan platform test folder split" {
        Some(
            "runtime_15_export_build_plan_platform_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M3 export build plan platform release-adapter test child-owner split"
    {
        Some(
            "runtime_15_export_build_plan_platform_release_adapter_tests_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 gameplay host test folder split" {
        Some("runtime_15_gameplay_host_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 script VM gameplay host guard child-owner split" {
        Some(
            "runtime_15_script_vm_gameplay_host_guard_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 shader prewarm manifest test folder split" {
        Some("runtime_15_shader_prewarm_manifest_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS schedule test folder split" {
        Some("runtime_15_scene_ecs_schedule_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS schedule conflict graph child folder split" {
        Some(
            "runtime_15_scene_ecs_schedule_conflict_graph_child_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 scene ECS systems test folder split" {
        Some("runtime_15_scene_ecs_systems_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS query test folder split" {
        Some("runtime_15_scene_ecs_query_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS query structure test folder split" {
        Some("runtime_15_scene_ecs_query_structure_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene derived-state test folder split" {
        Some("runtime_15_scene_derived_state_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 dynamic scene session path-management test folder split" {
        Some(
            "runtime_15_dynamic_scene_session_path_management_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 scene component-structure test folder split" {
        Some("runtime_15_scene_component_structure_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS reflect foundation test folder split" {
        Some(
            "runtime_15_scene_ecs_reflect_foundation_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 dynamic scene root test folder split" {
        Some("runtime_15_dynamic_scene_root_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene render extract test folder split" {
        Some("runtime_15_scene_render_extract_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene asset integration test folder split" {
        Some("runtime_15_scene_asset_integration_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene world basics test folder split" {
        Some("runtime_15_scene_world_basics_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene property paths test folder split" {
        Some("runtime_15_scene_property_paths_tests_folder_split_static_passed_cargo_deferred")
    } else {
        None
    }
}
