pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 production file budget core runtime guard split" => Some(
            "runtime_15_production_file_budget_core_runtime_guard_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 render shader template assembly guard support child-owner split" => Some(
            "runtime_15_render_shader_template_assembly_guard_support_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 render shader template assembly assertion contract child-owner split" => Some(
            "runtime_15_render_shader_template_assembly_assertion_contract_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 mesh pipeline shader source tests child-owner split" => Some(
            "runtime_15_mesh_pipeline_shader_source_tests_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 shader prewarm manifest guard child-owner split" => Some(
            "runtime_15_shader_prewarm_manifest_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 shader prewarm manifest current-child route sync" => Some(
            "runtime_15_shader_prewarm_manifest_current_child_route_sync_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
