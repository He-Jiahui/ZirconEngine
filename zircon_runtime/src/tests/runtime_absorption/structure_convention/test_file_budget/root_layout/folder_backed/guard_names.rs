pub(super) struct GuardNames {
    pub(super) asset_pack_guard: String,
    pub(super) asset_facade_guard: String,
    pub(super) asset_project_zmeta_guard: String,
    pub(super) asset_project_manager_guard: String,
    pub(super) asset_material_guard: String,
    pub(super) asset_gltf_importer_guard: String,
    pub(super) asset_gltf_primitive_fixtures_guard: String,
    pub(super) asset_importer_guard: String,
    pub(super) asset_project_flow_sample_guard: String,
    pub(super) asset_scene_guard: String,
    pub(super) code_review_findings_guard: String,
    pub(super) core_runtime_deactivation_guard: String,
    pub(super) dynamic_scene_absorption_guard: String,
    pub(super) runtime_diagnostics_guard: String,
    pub(super) root_layout_guard: String,
    pub(super) render_products_guard: String,
    pub(super) rhi_command_list_guard: String,
    pub(super) rhi_device_contract_guard: String,
    pub(super) script_vm_tests_guard: String,
    pub(super) gameplay_host_tests_guard: String,
    pub(super) scene_ecs_schedule_guard: String,
    pub(super) scene_ecs_query_guard: String,
    pub(super) scene_ecs_query_structure_guard: String,
    pub(super) scene_derived_state_guard: String,
    pub(super) scene_component_structure_guard: String,
    pub(super) scene_dynamic_scene_root_guard: String,
    pub(super) scene_dynamic_session_guard: String,
    pub(super) scene_ecs_reflect_foundation_guard: String,
    pub(super) scene_ecs_systems_guard: String,
    pub(super) shader_prewarm_manifest_guard: String,
}

pub(super) fn guard_names() -> GuardNames {
    GuardNames {
        asset_pack_guard: format!(
            "{}{}",
            "fn runtime_15_asset", "_pack_tests_are_folder_backed"
        ),
        asset_facade_guard: format!(
            "{}{}",
            "fn runtime_15_asset", "_facade_tests_are_folder_backed"
        ),
        asset_project_zmeta_guard: format!(
            "{}{}",
            "fn runtime_15_asset_project", "_zmeta_tests_are_folder_backed"
        ),
        asset_project_manager_guard: format!(
            "{}{}",
            "fn runtime_15_asset_project", "_manager_tests_are_folder_backed"
        ),
        asset_material_guard: format!(
            "{}{}",
            "fn runtime_15_asset", "_material_tests_are_folder_backed"
        ),
        asset_gltf_importer_guard: format!(
            "{}{}",
            "fn runtime_15_asset_gltf", "_importer_tests_are_folder_backed"
        ),
        asset_gltf_primitive_fixtures_guard: format!(
            "{}{}",
            "fn runtime_15_asset_gltf", "_primitive_fixtures_are_folder_backed"
        ),
        asset_importer_guard: format!(
            "{}{}",
            "fn runtime_15_asset", "_importer_tests_are_folder_backed"
        ),
        asset_project_flow_sample_guard: format!(
            "{}{}",
            "fn runtime_15_asset_project", "_flow_sample_tests_are_folder_backed"
        ),
        asset_scene_guard: format!(
            "{}{}",
            "fn runtime_15_asset", "_scene_tests_are_folder_backed"
        ),
        code_review_findings_guard: format!(
            "{}{}",
            "fn runtime_15_code_review", "_findings_tests_are_folder_backed"
        ),
        core_runtime_deactivation_guard: format!(
            "{}{}",
            "fn runtime_15_core_runtime", "_deactivation_blocked_tests_are_folder_backed"
        ),
        dynamic_scene_absorption_guard: format!(
            "{}{}",
            "fn runtime_15_dynamic_scene", "_absorption_guard_is_folder_backed"
        ),
        runtime_diagnostics_guard: format!(
            "{}{}",
            "fn runtime_15_runtime", "_diagnostics_tests_are_folder_backed"
        ),
        root_layout_guard: format!(
            "{}{}",
            "fn runtime_15_test_file_budget", "_guard_is_folder_backed"
        ),
        render_products_guard: format!(
            "{}{}",
            "fn runtime_15_render_camera", "_target_products_are_folder_backed"
        ),
        rhi_command_list_guard: format!(
            "{}{}",
            "fn runtime_15_rhi", "_command_list_tests_are_folder_backed"
        ),
        rhi_device_contract_guard: format!(
            "{}{}",
            "fn runtime_15_rhi", "_device_contract_tests_are_folder_backed"
        ),
        script_vm_tests_guard: format!(
            "{}{}",
            "fn runtime_15_script", "_vm_tests_are_folder_backed"
        ),
        gameplay_host_tests_guard: format!(
            "{}{}",
            "fn runtime_15_gameplay", "_host_tests_are_folder_backed"
        ),
        scene_ecs_schedule_guard: format!(
            "{}{}",
            "fn runtime_15_scene", "_ecs_schedule_tests_are_folder_backed"
        ),
        scene_ecs_query_guard: format!(
            "{}{}",
            "fn runtime_15_scene", "_ecs_query_tests_are_folder_backed"
        ),
        scene_ecs_query_structure_guard: format!(
            "{}{}",
            "fn runtime_15_scene", "_ecs_query_structure_tests_are_folder_backed"
        ),
        scene_derived_state_guard: format!(
            "{}{}",
            "fn runtime_15_scene", "_derived_state_tests_are_folder_backed"
        ),
        scene_component_structure_guard: format!(
            "{}{}",
            "fn runtime_15_scene", "_component_structure_tests_are_folder_backed"
        ),
        scene_dynamic_scene_root_guard: format!(
            "{}{}",
            "fn runtime_15_dynamic_scene", "_root_tests_are_folder_backed"
        ),
        scene_dynamic_session_guard: format!(
            "{}{}",
            "fn runtime_15_dynamic_scene_session", "_path_management_tests_are_folder_backed"
        ),
        scene_ecs_reflect_foundation_guard: format!(
            "{}{}",
            "fn runtime_15_scene_ecs", "_reflect_foundation_tests_are_folder_backed"
        ),
        scene_ecs_systems_guard: format!(
            "{}{}",
            "fn runtime_15_scene", "_ecs_systems_tests_are_folder_backed"
        ),
        shader_prewarm_manifest_guard: format!(
            "{}{}",
            "fn runtime_15_shader_prewarm", "_manifest_tests_are_folder_backed"
        ),
    }
}
